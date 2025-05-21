use std::{fs::{self}, path::PathBuf};
use crate::main_index;
use mythos_core::{dirs::{get_path, MythosDir}, printerror, printinfo, printwarn};
use crate::auto_installer::CharonIoError;

pub fn uninstall_utils(utils: Vec<String>, do_dry_run: bool) {
    //! 1. Find corresponding charon files.
    //! 2. Delete files listed in charon files.
    //! 3. If any directories are completely empty, delete them too.
    //! 4. Remove utils from main index.
    let root_path = match get_path(MythosDir::Data, "charon") {
        Some(mut path) => {
            // Using PathBuf.with_filename removes the last item of path, even if its a directory.
            path.push("filename");
            path
        },
        None => {
            printinfo!("Could not find any installed utilities");
            return;
        }
    };

    // Find all files that should be deleted.
    let mut files: Vec<PathBuf> = Vec::new();
    for util in &utils {
        let mut res = match find_files(
                &util, 
                &root_path.with_file_name(format!("{util}.charon"))) {
            Ok(files) => files,
            Err(err) => {
                printwarn!("{err}");
                continue;
            }
        };
        files.append(&mut res);
    }

    // Remove files and empty dirs.
    remove_files(files, do_dry_run);

    // Remove utils from main index.
    if !do_dry_run {
        let index = match main_index::update_main_index(utils) {
            Ok(index) => index,
            Err(err) => {
                printerror!("{err}");
                return;
            }
        };

        let root_path = match crate::get_util_index_path(do_dry_run) {
            Ok(path) => path,
            Err(err) => {
                printerror!("{err}");
                return;
            }
        };
        let path = root_path.join("index.charon");
        if let Err(err) = fs::write(path, &index) {
            printerror!("{err}");
        }
    }
}

fn find_files(util_name: &str, path: &PathBuf) -> Result<Vec<PathBuf>, CharonIoError> {
    if !path.exists() {
        return Err(CharonIoError::UnknownUtilName(Some(util_name.to_string())));
    }
    let file = match fs::read_to_string(path) {
        Ok(file) => file,
        Err(err) => {
            return Err(CharonIoError::GenericIoError(err));
        },
    };

    let mut output: Vec<PathBuf> = Vec::new();

    // Read contents of charon file.
    for line in file.split("\n") {
        if line.trim().starts_with("#") { continue; }
        else if line.is_empty() { continue; }

        let path = PathBuf::from(line);
        if !path.exists() { 
            printwarn!("{path:?} from {util_name} charon file does not exist.");
            continue;
        }
        output.push(path);
    }

    output.push(path.to_path_buf());
    return Ok(output);
}

fn remove_files(files: Vec<PathBuf>, do_dry_run: bool) -> Vec<PathBuf> {
    let mut output: Vec<PathBuf> = Vec::new();
    for file in files {
        if let Some(f) = remove_file(&file, do_dry_run) {
            output.push(f);
        }

        // Get file stem, in order to check if dir is empty.
        let path = match file.parent() {
            Some(path) => PathBuf::from(path),
            None => continue
        };

        if let Some(f) = remove_dir_if_empty(&path, do_dry_run) {
            output.push(f);
        }
    }

    return output;
}

fn remove_file(path: &PathBuf, do_dry_run: bool) -> Option<PathBuf> {
    if do_dry_run {
        printinfo!("Dry run: Would have removed file: {path:?}");
    } 
    else if let Err(err) = fs::remove_file(&path) {
        printerror!("Could not read file {path:?}. Error = {err}.");
        return None;
    } else {
        printinfo!("Removing file: {path:?}");
    }
    return Some(path.to_path_buf());
}

fn remove_dir_if_empty(path: &PathBuf, do_dry_run: bool) -> Option<PathBuf> {
    // If dir is empty and a mythos dir, delete it.
    if !path.is_dir() { 
        return None; 
    }
    if path.to_str().unwrap_or("").find("mythos").is_none() { return None; }

    // Check whether dir is empty.
    if let Ok(contents) = path.read_dir() {
        let length = contents.count();
        // If dir would be deleted when not doing a dry run.
        if do_dry_run {
            if length == 1 {
                printinfo!("Dry run: Would have removed dir {path:?}.");
                return Some(path.clone());
            }
            return None;
        } else if length == 0 {
            if let Err(err) = fs::remove_dir(&path) {
                printerror!("{err}");
                return None;
            } else {
                printinfo!("Removing dir: {path:?}.");
                return Some(path.clone());
            }
        }

        return Some(path.clone());
    } 
    return None;
}


#[cfg(test)]
mod tests {
    use std::env;

    use serial_test::serial;
    use super::*;

    #[serial]
    #[test]
    fn test_find_charon_files() {
        let files = find_files("a", &PathBuf::from("tests/uninstall/a.charon")).unwrap();
        assert_eq!(files, vec![
            PathBuf::from("/bin"), 
            PathBuf::from("/home"),
            PathBuf::from("tests/uninstall/a.charon"),
        ]);
    }

    #[serial]
    #[test]
    fn test_delete_files() {
        let files = remove_files(vec![
            PathBuf::from("tests/uninstall/data/b"),
            PathBuf::from("tests/uninstall/local_data/b"),
            PathBuf::from("tests/uninstall/config/b"),
        ], true);

        assert_eq!(files, vec![
            PathBuf::from("tests/uninstall/data/b"), 
            PathBuf::from("tests/uninstall/local_data/b"),
            PathBuf::from("tests/uninstall/config/b"),
        ]);
    }

    #[serial]
    #[test]
    fn test_delete_files_2() {
        let files = remove_files(vec![
            PathBuf::from("tests/uninstall/mythos/data/b"),
        ], false);

        assert!(!PathBuf::from("tests/uninstall/mythos/data/").exists());

        fs::create_dir("tests/uninstall/mythos/data/").unwrap();
        fs::write("tests/uninstall/mythos/data/b", " ").unwrap();

        assert_eq!(files, vec![
            PathBuf::from("tests/uninstall/mythos/data/b"), 
            PathBuf::from("tests/uninstall/mythos/data/"), 
        ]);
    }

    #[serial]
    #[test]
    fn test_update_main_index() {
        unsafe {
            env::set_var("MYTHOS_DATA_DIR", "tests/uninstall/");
        }
        let index = main_index::update_main_index(vec!["a".into(), "b".into()]).unwrap();
        assert_eq!(index, "[c]\nversion = \"0.0.3\"\n");
    }
}
