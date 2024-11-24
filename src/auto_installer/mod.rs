mod install_item;
mod installation_cmd;
mod charon_io_error;
mod charon_install_error;

use std::{ffi::OsString, fs, path::PathBuf};

use mythos_core::{printerror, printinfo};
use toml::Value;

#[derive(Debug)]
pub enum CharonIoError { 
    GenericIoError(std::io::Error),
    TomlError(toml::de::Error),
    CharonFileNotFound,
    CharonFileEmpty,
    InvalidCharonFile(String),
    // bad_dir: String
    InvalidDirKey(String, usize),
    // bad_item: String
    InvalidInstallItem(String, usize),
    // invalid_target: PathBuf
    TargetFileNotFound(PathBuf, usize),
    NoTargetProvided(usize),
    UnknownUtilName,
}
#[derive(Debug)]
pub enum CharonInstallError {
    GenericIoError(std::io::Error),
    DryRun,
    BadPermissions(std::io::Error),
    FileExistsNoOverwrite,
}

/**
 * A list of all items that must be installed.
 */
#[derive(Debug, PartialEq, Eq)]
pub struct InstallationCmd {
    pub items: Vec<InstallItem>,
    pub mkdirs: Vec<PathBuf>,
    pub name: String,
    /// Location to look for updates.
    pub source: Option<String>,
    /// Package version
    pub version: Option<String>,
    /// Package description
    pub description: Option<String>,
}

/**
 * A single item to be installed.
 */
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallItem {
    /// Path of item to be installed.
    pub target: PathBuf,
    /// Path item is to be installed to.
    pub dest: PathBuf,
    /// Permission of file in 000
    pub perms: u32,
    /// Remove extension from target file name.
    pub strip_ext: bool,
    /// Overwrite file if it already exists?
    pub overwrite: bool,
    /// Comments made during installation process. Used for logging.
    pub comment: String,
}

/// Parses installation file.
/// Assumes path exists.
/// Returns an error if charon file is invalid.
pub fn parse_installation_file(path: &PathBuf) -> Result<InstallationCmd, CharonIoError> {
    let path = find_charon_file(path.to_path_buf())?;
    printinfo!("Reading charon file at {path:?}");

    let parent = path
        .parent()
        .unwrap_or(&std::path::Path::new(""))
        .to_path_buf()
        .canonicalize();
    let parent: PathBuf = match parent {
        Ok(parent) => parent,
        Err(err) => return Err(CharonIoError::GenericIoError(err))
    };

    let file = match fs::read_to_string(&path) {
        Ok(file) => file,
        Err(err) => return Err(CharonIoError::GenericIoError(err)),
    };

    if file.is_empty() {
        return Err(CharonIoError::CharonFileEmpty);
    }

    // Read contents of charon file.
    let table = match toml::from_str::<Value>(&file) {
        Ok(Value::Table(table)) => table,
        Ok(other) => {
            let msg = format!("Expected a table, found {other:?}.");
            return Err(CharonIoError::InvalidCharonFile(msg));
        },
        Err(err) => return Err(CharonIoError::TomlError(err)),
    };

    let mut cmd = InstallationCmd::new();
    cmd.name = match parse_util_name(&path) {
        Some(name) => name,
        None => {
            return Err(CharonIoError::UnknownUtilName);
        }
    };

    // Start actually parsing file.
    for (i, (key, val)) in table.iter().enumerate() {
        let dest = match cmd.add_dir(&key) {
            Some(dest) => dest,
            None => {
                if key.to_lowercase() == "info" {
                    cmd.set_info(&val);
                    continue;
                } 
                return Err(CharonIoError::InvalidDirKey(key.to_string(), i));
            }
        };

        if let toml::Value::Array(list) = val {
            for item in list {
                cmd.add_item(&parent, &dest, &item, i)?;
            }
        } else {
            return Err(CharonIoError::InvalidInstallItem(val.to_string(), i));
        }
    }

    return Ok(cmd);
}

fn find_charon_file(path: PathBuf) -> Result<PathBuf, CharonIoError> {
    //! Path is a file, alleged to be a charon file.
    //! Or path is a directory which must contain a charon file.
    //!     File should must have .charon extension.
    if path.is_file() {
        return Ok(path);
    }

    let mut contents = match path.read_dir() {
        Ok(contents) => contents,
        Err(err) =>  return Err(CharonIoError::GenericIoError(err))
    };

    let res = contents.find(|file| {
        if let Ok(file) = file {
            file.path().extension() == Some(&OsString::from("charon"))
        } else {
            false
        }
    });
    if let Some(entry) = res {
        match entry {
            Ok(entry) => return Ok(entry.path()),
            Err(err) => return Err(CharonIoError::GenericIoError(err))
        }
    }
    // No files matched pattern
    return Err(CharonIoError::CharonFileNotFound);
}

fn parse_util_name(path: &PathBuf) -> Option<String> {
    if path.extension()? == "charon" {
        return Some(path.file_stem()?.to_string_lossy().to_string());
    } 
    return Some(path.parent()?.file_stem()?.to_string_lossy().to_string());
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    #[test]
    fn find_charon_file_given_dir() {
        let res = find_charon_file(PathBuf::from("tests/find_charon_file")).unwrap();
        assert_eq!(res, PathBuf::from("tests/find_charon_file/empty.charon"));
    }
    #[test]
    fn find_charon_file_dne() {
        let res = find_charon_file(PathBuf::from("tests/find_charon_file_dne")).unwrap_err();
        println!("{res}");
        assert!(matches!(res, CharonIoError::CharonFileNotFound));
    }
    #[test]
    fn file_is_empty() {
        let res = parse_installation_file(&PathBuf::from("tests/find_charon_file/empty.charon")).unwrap_err();
        println!("{res}");
        assert!(matches!(res, CharonIoError::CharonFileEmpty));
    }
    #[test] 
    fn charon_file_not_valid_toml() {
        let res = parse_installation_file(&PathBuf::from("tests/not_toml.charon")).unwrap_err();
        println!("{res}");
        assert!(matches!(res, CharonIoError::TomlError(_)));
    }
    #[test]
    fn charon_file_invalid_dir_key() {
        let res = parse_installation_file(&PathBuf::from("tests/invalid_dir_key.charon")).unwrap_err();
        println!("{res}");
        assert!(matches!(res, CharonIoError::InvalidDirKey(_, _)));
    }
    #[test]
    fn charon_file_invalid_item() {
        let res = parse_installation_file(&PathBuf::from("tests/invalid_install_item.charon")).unwrap_err();
        println!("{res}");
        assert!(matches!(res, CharonIoError::InvalidInstallItem(_, _)));
    }
    #[test]
    fn no_target_provided() {
        let res = parse_installation_file(&PathBuf::from("tests/no_target_provided.charon")).unwrap_err();
        println!("{res}");
        assert!(matches!(res, CharonIoError::NoTargetProvided(_)));
    }
    #[test]
    fn valid_charon_file() {
        unsafe {
            env::set_var("MYTHOS_CONFIG_DIR", "tests/valid/dests/etc");
            env::set_var("MYTHOS_LOCAL_CONFIG_DIR", "tests/valid/dests/config");
            env::set_var("MYTHOS_BIN_DIR", "tests/valid/dests/bin");
            env::set_var("MYTHOS_DATA_DIR", "tests/valid/dests/data");
        }
        let items = vec![
            // config = [ { target = "targets/config.conf" }]
            InstallItem { 
                target: PathBuf::from("tests/valid/targets/config.conf"), 
                dest: PathBuf::from("tests/valid/dests/etc/valid/config.conf"), 
                perms: 0,
                strip_ext: false, 
                overwrite: true, 
                comment: String::new() 
            },
            InstallItem { 
                target: PathBuf::from("tests/valid/targets/config.conf"), 
                dest: PathBuf::from("tests/valid/dests/config/valid/config.conf"), 
                perms: 0,
                strip_ext: false, 
                overwrite: false, 
                comment: String::new() 
            },
            InstallItem { 
                target: PathBuf::from("tests/valid/targets/1.txt"), 
                dest: PathBuf::from("tests/valid/dests/data/valid/one.txt"), 
                perms: 0,
                strip_ext: false, 
                overwrite: true, 
                comment: String::new() 
            },
            InstallItem { 
                target: PathBuf::from("tests/valid/targets/2.txt"), 
                dest: PathBuf::from("tests/valid/dests/data/valid/2.txt"), 
                perms: 0,
                strip_ext: false, 
                overwrite: true, 
                comment: String::new() 
            },
            InstallItem { 
                target: PathBuf::from("tests/valid/targets/executable.bin"), 
                dest: PathBuf::from("tests/valid/dests/bin/executable"), 
                perms: 0x755,
                strip_ext: true, 
                overwrite: true, 
                comment: String::new() 
            },
        ];

        let res = parse_installation_file(&PathBuf::from("tests/valid/valid.charon")).unwrap();
        for item in res.items {
            println!("{item:?}");
            assert!(items.contains(&item));
        }
    }
    #[test]
    fn empty_dir_field() {
        unsafe {
            env::set_var("MYTHOS_DATA_DIR", "tests/valid/dests/data1");
        }
        let res = parse_installation_file(&PathBuf::from("tests/valid/empty_dir_field.charon")).unwrap();
        assert_eq!(res.mkdirs, vec![PathBuf::from("tests/valid/dests/data1/empty_dir_field")])
    }
}
