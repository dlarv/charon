/*!
 * Installer for mythos projects, now fully divorced from mythos-core/plutonian-shores.
 * Reads a toml-style file containing installation instructions.
 */

use std::{env, fs, path::PathBuf};

use auto_installer::{parse_installation_file, InstallationCmd};
use mythos_core::{cli::clean_cli_args, dirs, printerror, printinfo};


mod auto_installer;

fn main() {
    let mut do_dry_run = false;
    let mut path = None;
    let mut args = clean_cli_args();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                println!("charon [opts] [path]\nBasic installer util that can use toml files to quickly install programs.\nopts:\n-h | --help\t\tPrint this menu\n-n | --dryrun\t\tRun command without making changes to filesystem\n-U | --uninstall\t\tBegin uninstallation process.\n-c | --create\t\tCreate a basic charon file");
                return;
            },
            "-n" | "--dryrun" => do_dry_run = true,
            "-c" | "--create" => {
            },
            "-U" | "--uninstall" => {
                uninstall(args.next());
                return;
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
    let cmd = match parse_installation_file(&path) {
        Ok(cmd) => cmd,
        Err(err) => {
            printerror!("{err}");
            return;
        }
    };

    let util_name = cmd.name.clone();

    // Install files.
    printinfo!("Beginning installation.");
    let new_charon_index = install(cmd, do_dry_run);

    // Load old charon file, if it exists.
    let old_charon_index = match read_index(&util_name, do_dry_run) {
        Ok(file) => file,
        // Fails if fs error occurs.
        // read_index() takes care of logging this error.
        Err(_) => return,
    };

    // Remove orphans.
    process_orphans(old_charon_index, &new_charon_index, do_dry_run);

    // Write (new) index.
    let charon_index_path = if do_dry_run {
        PathBuf::from(format!("{util_name}"))
    } else {
        match get_util_index_path(do_dry_run) {
            Some(path) => path.with_file_name(util_name).with_extension("charon"),
            None => {
                printerror!("Due to an error while trying to access util index, index file was saved as if this were a dryrun.");
                PathBuf::from(format!("{util_name}.dryrun.charon"))
            }
        }
    };

    if let Err(err) = fs::write(&charon_index_path, new_charon_index.join("\n")) {
        printerror!("An error occurred while writing charon file. Error = {err}.");
        return;
    }

    printinfo!("Installation complete!");
}

fn install(cmd: InstallationCmd, do_dry_run: bool) -> Vec<String> {
    let mut charon_index: Vec<String> = Vec::new();

    for mut item in cmd.items {
        printinfo!("Installing {:?} --> {:?}", item.target, item.dest);

        if let Err(err) = item.try_install(do_dry_run) {
            printerror!("{err}");
        }
        printinfo!("{}", item.comment);
        charon_index.push(item.print_dest());
        charon_index.push(item.comment);

    }
    return charon_index;
}

fn get_util_index_path(do_dry_run: bool) -> Option<PathBuf> {
    let path = dirs::expand_path(dirs::MythosDir::Data, "charon");

    if path.exists() { }
    else if do_dry_run {
        printinfo!("$MYTHOS_DATA_DIR/charon/ does not exists. But since this is a dry run, no changes were made.");
    }
    else {
        printinfo!("$MYTHOS_DATA_DIR/charon/ does not exists, making it now...");
        match dirs::make_dir(dirs::MythosDir::Data, "charon") {
            Ok(path) => return Some(path),
            Err(err) => {
                printerror!("An error occurred while trying to mkdir. Error = {err}.");
                return None;
            }
        }
    }

    return Some(path);
}

fn read_index(util_name: &str, do_dry_run: bool) -> Result<Vec<String>, ()> {
    //! Read file inside $MYTHOS_DATA_DIR/$util_name.charon
    // make_dir works the same as get_path, except it creates the dir if it dne.
    let mut path = match get_util_index_path(do_dry_run) {
        Some(path) => path,
        None => {
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

fn uninstall(util: Option<String>) {
}

#[cfg(test)]
mod tests {
    use std::env;
    use super::*;

    fn setup1() {
        unsafe {
            env::set_var("MYTHOS_CONFIG_DIR", "tests/valid/dests/etc");
            env::set_var("MYTHOS_LOCAL_CONFIG_DIR", "tests/valid/dests/config");
            env::set_var("MYTHOS_BIN_DIR", "tests/valid/dests/bin");
            env::set_var("MYTHOS_DATA_DIR", "tests/valid/dests/data");
        }
    }
    #[test]
    fn overwrite() {
        setup1();
        let cmd = parse_installation_file(&PathBuf::from("tests/main/overwrite.charon")).unwrap();
        let res = install(cmd, false);
    }
    #[test]
    fn no_overwrite() {
    }
    #[test]
    fn invalid_permissions() {
    }
    #[test]
    fn load_old_index() {
    }
    #[test]
    fn read_old_index_dne() {
    }
    #[test]
    fn remove_orphans() {
    }
    #[test]
    fn uninstall() {
    }
}

