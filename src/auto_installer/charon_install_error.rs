use std::fmt::Display;

use super::CharonInstallError;


impl Display for CharonInstallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            CharonInstallError::GenericIoError(err) => write!(f, "An error occurred while copying file. Error = {err}."),
            CharonInstallError::DryRun => write!(f, "Installation is dryrun, no changes were made."),
            CharonInstallError::BadPermissions(err) => write!(f, "An error occurred while changing file permissions. Error = {err}."),
            CharonInstallError::FileExistsNoOverwrite => write!(f, "File exists and !overwrite."),
        };
    }
}
