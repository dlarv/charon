mod install_item;
mod installation_cmd;
mod charon_io_error;

use std::{ffi::OsString, fs, path::PathBuf};

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
}


/**
 * A list of all items that must be installed.
 */
#[derive(Debug)]
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
#[derive(Debug, Clone)]
pub struct InstallItem {
    /// Path of item to be installed.
    pub target: PathBuf,
    /// Path item is to be installed to.
    pub dest: PathBuf,
    /// Permission of file in 000
    pub perms: u32,
    /// Remove extension from target file name.
    pub strip_ext: bool,
    /// Optional name of installed file.
    pub alias: Option<PathBuf>,
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

    let parent: PathBuf = path
        .parent()
        .unwrap_or(&std::path::Path::new(""))
        .to_path_buf()
        .canonicalize()
        .unwrap();

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
    for (i, (key, val)) in table.iter().enumerate() {
        let dest = match cmd.add_dir(&key) {
            Some(dest) => dest,
            None => return Err(CharonIoError::InvalidDirKey(key.to_string(), i))
        };

        if let toml::Value::Array(list) = val {
            for item in list {
                cmd.add_item(&parent, &dest, &item);
            }
        } 
        else if let toml::Value::Table(_) = val {
            cmd.set_info(&val);
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

#[cfg(test)]
mod tests {
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

}
