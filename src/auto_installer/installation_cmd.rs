use std::path::PathBuf;

use mythos_core::{dirs, printinfo};
use toml::Value;

use crate::auto_installer::InstallItem;

use super::{CharonIoError, InstallationCmd};

impl InstallationCmd {
    pub fn new() -> InstallationCmd {
        return InstallationCmd {
            items: Vec::new(),
            mkdirs: Vec::new(),
            name: "".into(),
            source: None,
            version: None,
            description: None,
        };
    }
    pub fn set_info(&mut self, val: &Value, charon_path: &PathBuf) -> Result<(), CharonIoError> {
        //! Get version, description, source, etc from info section of a .charon file.
        if let Some(Value::String(val)) = val.get("name") {
            self.name = val.to_string();
        }
        if let Some(Value::String(val)) = val.get("version") {
            self.version = Some(val.to_string());
        }
        if let Some(Value::String(val)) = val.get("description") {
            self.description = Some(val.to_string());
        }
        if let Some(Value::String(val)) = val.get("source") {
            self.source = Some(validate(val, &charon_path)?);
        }
        return Ok(());
    }
    pub fn add_item(&mut self, parent: &PathBuf, dest: &PathBuf, val: &Value, line_num: usize) -> Result<(), CharonIoError>{
        //! Returns Ok if install item was added correctly.
        //! Returns Err if there was a CharonIoError::InvalidInstallItem.
        //! That error is not created in here, b/c this function doesn't know the line number.
        let mut cmd = InstallItem {
            target: parent.into(),
            dest: dest.into(),
            perms: 000,
            strip_ext: false,
            overwrite: true,
            comment: "".into(),
        };
        let table = match val {
            Value::String(v) => {
                cmd.dest = v.into();
                self.items.push(cmd);
                return Ok(());
            },
            Value::Table(table) => {
                table
            },
            _ => return Err(CharonIoError::InvalidInstallItem(val.to_string(), line_num))
        };

        let mut dest = None;
        let mut alias: Option<PathBuf> = None;

        // Resolve target path and ensure it exists.
        if let Some(Value::String(val)) = table.get("target") {
            // cmd.target = val.into();
            cmd.target = parent.join(&val).canonicalize().unwrap_or(val.into());
        } else {
            return Err(CharonIoError::NoTargetProvided(line_num));
        }
        if !cmd.target.exists() {
            return Err(CharonIoError::TargetFileNotFound(cmd.target.into(), line_num));
        }


        if let Some(Value::String(val)) = table.get("dest") {
            dest = Some(PathBuf::from(val));
        }
        if let Some(Value::Integer(val)) = table.get("perms") {
            cmd.perms = val.to_owned() as u32;
        }
        if let Some(Value::Boolean(val)) = table.get("strip_ext") {
            cmd.strip_ext = val.to_owned();
        }
        if let Some(Value::String(val)) = table.get("alias") {
            alias = Some(val.into());
        }
        if let Some(Value::Boolean(val)) = table.get("overwrite") {
            cmd.overwrite = val.to_owned();
        }
        if let Some(Value::String(val)) = table.get("comment") {
            cmd.comment = val.to_owned();
        }
        // alias >> strip_ext >> dest >> target_file_name
        let dest = if let Some(alias) = &alias {
            alias.to_owned()
        } else {
            // Get dest or file_name and remove extension, if applicable.
            let dest: PathBuf = if let Some(dest) = dest {
                dest
            } else {
                cmd.target.file_name().unwrap().into()
            };

            if cmd.strip_ext {
                dest.file_stem().unwrap().into()
            } else {
                dest
            }
        };
        cmd.dest.push(dest);
        printinfo!("Copy {target:#?} --> {dest:#?}", target = cmd.target, dest = cmd.dest);
        self.items.push(cmd);
        return Ok(());
    }
    pub fn add_simple_item(&mut self, target: PathBuf, dest: PathBuf, perms:u32, overwrite: bool, strip_ext: bool) {
        //! Add item without using a toml file.
        let dest = if strip_ext {
            dest.join(PathBuf::from(target.file_stem().unwrap_or_default()).file_name().unwrap_or_default())
        } else {
            dest
        };
        let item = InstallItem {
            target,
            dest,
            perms,
            strip_ext,
            overwrite,
            comment: "".to_string(),
        };
        self.items.push(item);
    }
    pub fn add_dir(&mut self, dir: &str) -> Option<PathBuf> {
        if let Some(path) = dirs::expand_mythos_shortcut(dir, &self.name) {
            if !self.mkdirs.contains(&path) && !path.exists() {
                printinfo!("Create directory: {path:#?}");
                self.mkdirs.push(path.to_owned());
            }
            return Some(path);
        }
        return None;
    }
    pub fn to_toml_str(&self) -> String {
        let mut output = format!("{} = {{", self.name);
        if let Some(val) = &self.version {
            output += &format!("version = \"{val}\", ");
        }
        if let Some(val) = &self.description{
            output += &format!("description = \"{val}\", ");
        }
        let src = if let Some(val) = &self.source {
            val
        } else {
            "charon"
        };
        output += &format!("source = \"{src}\" }}");
        return output;
    }
}

/// If user provided relative file path, expand it.
fn validate(path: &str, charon_path: &PathBuf) -> Result<String, CharonIoError> {
    if path == "." {
        return match charon_path.canonicalize()?.parent() {
            Some(path) => Ok(path.to_string_lossy().to_string()),
            None => Err(CharonIoError::InfoSourceBad(path.into()))
        }
    }

    let p = PathBuf::from(path);
    if p.exists() && p.is_relative() {
        return match p.canonicalize() {
            Ok(path) => Ok(path.to_string_lossy().to_string()),
            Err(_) => Err(CharonIoError::InfoSourceBad(p))
        }
    }
    return Ok(path.to_string());
}


