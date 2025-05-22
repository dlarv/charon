/*!
 * Installer for mythos projects, now fully divorced from mythos-core/plutonian-shores.
 * Reads a toml-style file containing installation instructions.
 */

use std::{env, fs, path::PathBuf};

mod auto_installer;
mod uninstaller;
mod main_index;

use auto_installer::{parse_installation_file, CharonIoError, InstallationCmd};
use mythos_core::{cli::clean_cli_args, dirs, printerror, printinfo};

fn main() {
    let mut do_dry_run = false;
    let mut path = None;
    let mut args = clean_cli_args();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                println!("charon [opts] [path]|[utils...]\nBasic installer util that can use toml files to quickly install programs.\nopts:\n-h | --help\t\tPrint this menu\n-n | --dryrun\t\tRun command without making changes to filesystem\n-r | --remove\t\tDeletes all files installed by mythos utils. The util must have been installed using charon.\n\n-l | --list\t\tShow list of utils installed using charon.\n-L\t\t\tLike -l, but more verbose.\n--src\t\t\tLike -l, but show sources as well.");
                return;
            },
            "-n" | "--dryrun" => do_dry_run = true,
            "-r" | "--remove" => {
                uninstall(args, do_dry_run);
                return;
            },
            "-l" | "--list" => {
                main_index::list_main_index(main_index::ListMode::Simple);
                return
            },
            "-L" => {
                main_index::list_main_index(main_index::ListMode::Verbose);
                return
            },
            "--src" => {
                main_index::list_main_index(main_index::ListMode::Source);
                return
            },
            _ => {
                if arg.starts_with("-") {
                    printerror!("Unknown arg: {arg}.");
                    return;
                }
                path = Some(arg);
            }
        }
    }
    let path = if let Some(path) = path {
        PathBuf::from(path)
    } else {
        match env::current_dir() {
            Ok(path) => path,
            Err(err) => {
                printerror!("Could not get $CWD. Error = {err:?}."); 
                return;
            }
        }
    };
    // Find valid .charon file.
    // Parse .charon file => InstallationCmd.
    let mut cmd = match parse_installation_file(&path) {
        Ok(cmd) => cmd,
        Err(err) => {
            printerror!("{err}");
            return;
        }
    };

    let util_name = cmd.name.clone();

    // Install files.
    printinfo!("\nBeginning installation.");
    let new_charon_index = install(&mut cmd, do_dry_run);

    // Load old charon file, if it exists.
    let old_charon_index = match read_util_index(&util_name, do_dry_run) {
        Ok(file) => file,
        // Fails if fs error occurs.
        // read_index() takes care of logging this error.
        Err(_) => return,
    };

    // Remove orphans.
    process_orphans(old_charon_index, &new_charon_index, do_dry_run);

    // Write (new) index.
    let charon_index_path = if do_dry_run {
        PathBuf::from(format!("{util_name}.dryrun.charon"))
    } else {
        match get_util_index_path(do_dry_run) {
            Ok(mut path) => {
                path.push(util_name + ".charon");
                path
            },
            Err(_) => {
                printerror!("Due to an error while trying to access util index, index file was saved as if this were a dryrun.");
                PathBuf::from(format!("{util_name}.dryrun.charon"))
            }
        }
    };


    println!("\nUpdating util index file: {charon_index_path:?}");
    if let Err(err) = fs::write(&charon_index_path, new_charon_index.join("\n")) {
        printerror!("An error occurred while writing charon file. Error = {err}.");
        return;
    }

    println!("\nUpdating main index file");
    if let Err(err) = main_index::update(&mut cmd, do_dry_run) {
        printerror!("An error occurred while writing charon file. Error = {err}.");
        return;
    }

    printinfo!("Installation complete!");
}


fn install(cmd: &mut InstallationCmd, do_dry_run: bool) -> Vec<String> {
    let mut charon_index: Vec<String> = Vec::new();

    charon_index.push("# Directories".to_string());
    for dir in &cmd.mkdirs {
        charon_index.push(dir.to_string_lossy().to_string());

        if dir.exists() {
            printinfo!("Directory already exists: {dir:?}");
            charon_index.push("# Directory already exists: {dir:?}".into());
            continue;
        } 

        printinfo!("Created directory: {dir:?}");

        if !do_dry_run {
            if let Err(err) = fs::create_dir_all(&dir) {
                printerror!("An error occurred while trying to make directory. Error = {err}.");
            }
        }
    }

    charon_index.push("# Files".to_string());
    for item in &mut cmd.items {
        printinfo!("Installing {:?} --> {:?}", item.target, item.dest);

        if let Err(err) = item.try_install(do_dry_run) {
            printerror!("{err}");
        }
        printinfo!("{}", item.comment);
        charon_index.push(item.print_dest());
        charon_index.push(item.comment.clone());

    }
    return charon_index;
}

fn get_util_index_path(do_dry_run: bool) -> Result<PathBuf, CharonIoError> {
    let path = dirs::expand_path(dirs::MythosDir::Data, "charon");

    if path.exists() { }
    else if do_dry_run {
        printinfo!("$MYTHOS_DATA_DIR/charon/ does not exists. But since this is a dry run, no changes were made.");
    }
    else {
        printinfo!("$MYTHOS_DATA_DIR/charon/ does not exists, making it now...");
        match dirs::make_dir(dirs::MythosDir::Data, "charon") {
            Ok(path) => return Ok(path),
            Err(err) => {
                printerror!("An error occurred while trying to mkdir. Error = {err}.");
                return Err(CharonIoError::GenericIoError(err));
            }
        }
    }

    return Ok(path);
}

fn read_util_index(util_name: &str, do_dry_run: bool) -> Result<Vec<String>, ()> {
    //! Read file inside $MYTHOS_DATA_DIR/charon/$util_name.charon
    // make_dir works the same as get_path, except it creates the dir if it dne.
    let mut path = match get_util_index_path(do_dry_run) {
        Ok(path) => path,
        Err(_) => {
            return Err(());
        }
    };
    path.push(util_name.to_owned() + ".charon");

    if !path.exists() {
        return Ok(vec![]);
    }

    let contents: Vec<String> = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(err) => {
            printerror!("An error occurred while reading old index file at {path:?}. Error = {err}.");
            return Err(());
        }
    }.trim()
        .split("\n")
        .filter(|x| x.len() > 0)
        .map(|x| x.to_string())
        .collect();

    return Ok(contents);
}

fn process_orphans(old_index: Vec<String>, new_index: &Vec<String>, do_dry_run: bool) -> Vec<PathBuf> {
    // Compare files.
    // If file exists in old, but not in new, it is an orphan.
    let mut orphans: Vec<PathBuf> = Vec::new();
    printinfo!("\nProcessing orphans...");

    for old in old_index {
        // Skip comments.
        if old.starts_with("#") { continue; }

        if !new_index.contains(&old) {
            let path = PathBuf::from(old);
            printinfo!("Found orphaned file: {path:?}");

            if !path.exists() {
                printinfo!("But file no longer exists. Skipping...");
            } else if do_dry_run {
                printinfo!("Dry run. Skipping...");
            } else {
                match fs::remove_file(&path) {
                    Ok(_) => printinfo!("File was removed!"),
                    Err(err) => printerror!("An error occurred while removing orphan. Error = {err}.")
                }
            }

            orphans.push(path);
        }
    }

    return orphans;
}


fn uninstall<T: Iterator<Item = String>>(mut utils: T, mut do_dry_run: bool) {
    // Find corresponding charon files.
    // Delete files listed in charon files.
    // If any directories are completely empty, delete them too.
    // Remove utils from main index.
    let mut pkgs: Vec<String> = Vec::new();
    
    while let Some(util) = utils.next() {
        match util.as_str() {
            "-n" | "--dryrun" => do_dry_run = true,
            _ => {
                if util.starts_with("-") {
                    continue;
                }
                pkgs.push(util);
            }
        }
    }
    uninstaller::uninstall_utils(pkgs, do_dry_run);
}

#[cfg(test)]
mod tests {
    use std::env;
    use serial_test::serial;

    use super::*;

    fn setup1() {
        unsafe {
            env::set_var("MYTHOS_CONFIG_DIR", "tests/main/dests/etc");
            env::set_var("MYTHOS_LOCAL_CONFIG_DIR", "tests/main/dests/config");
            env::set_var("MYTHOS_BIN_DIR", "tests/main/dests/bin");
            env::set_var("MYTHOS_DATA_DIR", "tests/main/dests/data");
        }
    }
    #[serial]
    #[test]
    fn overwrite() {
        setup1();
        let mut cmd = parse_installation_file(&PathBuf::from("tests/main/overwrite.charon")).unwrap();
        let res = install(&mut cmd, false);

        let mut counter = 0;
        for item in res {
            if item.contains("File exists && !overwrite") {
                counter += 1;
            }
        }
        assert_eq!(counter, 1);
        fs::remove_file("tests/main/dests/data/overwrite/overwrite2.txt").unwrap();
        assert!(!PathBuf::from("tests/main/dests/data/overwrite/overwrite2.txt").exists());
    }
    #[serial]
    #[test]
    fn load_old_index() {
        setup1();
        let res = read_util_index("util1", true).unwrap();
        assert!(res.len() == 3);
    }
    #[serial]
    #[test]
    fn read_old_index_dne() {
        let res = read_util_index("util2", true).unwrap();
        assert!(res.is_empty());
    }
    #[serial]
    #[test]
    fn remove_orphans() {
        setup1();
        let old_index = read_util_index("orphan_test", true).unwrap();
        let mut cmd = parse_installation_file(&PathBuf::from("tests/main/orphan_test.charon")).unwrap();
        let new_index = install(&mut cmd, true);

        println!("{new_index:?}");
        println!("{old_index:?}");
        let orphans = process_orphans(old_index, &new_index, true);

        println!("{orphans:?}");
        assert!(orphans.contains(&PathBuf::from("tests/main/dests/data/orphan_test/Orphan1")));
        assert!(orphans.contains(&PathBuf::from("tests/main/dests/data/orphan_test/Orphan2")));
        assert!(orphans.contains(&PathBuf::from("tests/main/dests/data/orphan_test/Orphan3/Item1")));
        assert_eq!(orphans.len(), 3);
    }
}

