use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};

use super::{CharonInstallError, InstallItem};

impl InstallItem {
    pub fn new() -> InstallItem {
        return InstallItem {
            target: PathBuf::new(),
            dest: PathBuf::new(),
            perms: 0,
            strip_ext: false,
            overwrite: true,
            comment: "".into(),
        };
    }
    pub fn print_dest(&self) -> String {
        return self.dest.to_string_lossy().to_string();
    }

    pub fn try_install(&mut self, do_dry_run: bool) -> Result<(), CharonInstallError> {
        // GenericIoError >> BadPermissions >> NoOverwrite >> DryRun
        let mut comment = vec!["#".to_string()];
        // Init error code.
        let mut err = if do_dry_run {
            Some(CharonInstallError::DryRun)
        } else {
            None
        };

        // If part of comment was declared in charon file, copy it over now.
        if self.comment.len() > 0 {
            comment.push(self.comment.to_string());
        }

        if self.dest.exists() && !self.overwrite {
            comment.push("File exists && !overwrite".into());
            err = Some(CharonInstallError::FileExistsNoOverwrite);

        } else if !do_dry_run {
            match fs::copy(&self.target, &self.dest) {
                Ok(_) => {
                    comment.push("Successfully installed".into());
                    match self.dest.metadata() {
                        Ok(metadata) => metadata.permissions().set_mode(self.perms),
                        Err(msg) => {
                            comment.push(format!("Error changing permissions for {:?}. {msg}.", self.dest));

                            err = Some(CharonInstallError::BadPermissions(msg));
                        }
                    }
                },
                Err(msg) => {
                    comment.push(format!("Could not copy file: {msg}"));
                    err = Some(CharonInstallError::GenericIoError(msg));
                }
            }
        }

        self.comment = comment.join("; ");

        if let Some(err) = err {
            return Err(err);
        }
        return Ok(());
    }
}

