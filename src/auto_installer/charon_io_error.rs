use std::{error::Error, fmt::Display};

use super::CharonIoError;

impl Error for CharonIoError { }

impl Display for CharonIoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            CharonIoError::CharonFileNotFound => write!(f, "Could not find charon file in $CWD."),
            CharonIoError::CharonFileEmpty => write!(f, "Charon file provided is empty."),
            CharonIoError::TomlError(err) => write!(f, "Error reading toml file. Error = {err:?}."),
            CharonIoError::InvalidDirKey(key, i) => write!(f, "Invalid directory shortcut on line {i}: \"{key}\"."),
            CharonIoError::InvalidInstallItem(item, i) => write!(f, "Invalid install item on line {i}: \"{item}\"."),
            CharonIoError::TargetFileNotFound(path, i) => write!(f, "Could not find target item {path:?}. Item declared on line {i} of Charon file."),
            CharonIoError::GenericIoError(err) => write!(f, "Error reading Charon file. Error = {err:?}"),
            CharonIoError::InvalidCharonFile(msg) => write!(f, "{msg}."),
        };
    }
}