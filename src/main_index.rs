use std::{fs, path::PathBuf};
use mythos_core::printwarn;
use toml::{map::Map, Value};
use crate::auto_installer::CharonIoError;
use super::InstallationCmd;


pub fn update(cmd: &InstallationCmd, do_dry_run: bool) -> Result<String, CharonIoError> {
    // Keep a master list of all util info, mostly their version and source.
    // This will be used to do system updates.
    let root_path = crate::get_util_index_path(do_dry_run)?;
    let path = root_path.join("index.charon");

    let file = match fs::read_to_string(&path) {
        Ok(file) => file,
        Err(err) => {
            if !path.exists() {
                String::new()
            } else {
                return Err(CharonIoError::GenericIoError(err));
            }
        }

    };

    // Read contents of charon file.
    let mut table = match toml::from_str::<Value>(&file) {
        Ok(Value::Table(table)) => table,
        Ok(other) => {
            let msg = format!("Expected a table, found {other:?}.");
            return Err(CharonIoError::InvalidCharonFile(msg));
        },
        Err(err) => return Err(CharonIoError::TomlDeError(err)),
    };

    let info = get_info_from_cmd(&cmd);
    table.insert(cmd.name.clone(), info);

    // Write output
    let output = match toml::to_string(&table) {
        Ok(val) => val,
        Err(err) => return Err(CharonIoError::TomlSerError(err))
    };

    if do_dry_run {
        let path = root_path.with_file_name("index.dry_run.charon");
        if let Err(err) = fs::write(path, &output) {
            return Err(CharonIoError::GenericIoError(err));
        }
    } else {
        if let Err(err) = fs::write(path, &output) {
            return Err(CharonIoError::GenericIoError(err));
        }
    }
    
    return Ok(output);
}

fn get_info_from_cmd(cmd: &InstallationCmd) -> Value {
    let mut output = Map::new();

    if let Some(version) = &cmd.version {
        output.insert("version".into(), Value::String(version.to_string()));
    }
    if let Some(source) = &cmd.source {
        output.insert("source".into(), Value::String(source.to_string()));
    }

    return Value::Table(output);
}

pub fn update_main_index(utils: Vec<String>) -> Result<String, CharonIoError> {
    // Keep a master list of all util info, mostly their version and source.
    // This will be used to do system updates.
    let root_path = crate::get_util_index_path(false)?;
    let path = root_path.join("index.charon");

    let file = match fs::read_to_string(&path) {
        Ok(file) => file,
        Err(err) => {
            if !path.exists() {
                String::new()
            } else {
                return Err(CharonIoError::GenericIoError(err));
            }
        }

    };

    // Read contents of charon file.
    let mut table = match toml::from_str::<Value>(&file) {
        Ok(Value::Table(table)) => table,
        Ok(other) => {
            let msg = format!("Expected a table, found {other:?}.");
            return Err(CharonIoError::InvalidCharonFile(msg));
        },
        Err(err) => return Err(CharonIoError::TomlDeError(err)),
    };

    for util in utils {
        if let None = table.remove(&util) {
            printwarn!("Did not find {util} in main index.");
        }
    }

    return match toml::to_string(&table) {
        Ok(val) => Ok(val),
        Err(err) => return Err(CharonIoError::TomlSerError(err))
    };
}
