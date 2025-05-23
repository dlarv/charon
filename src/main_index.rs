use std::fs;
use mythos_core::{printinfo, printwarn};
use toml::{map::Map, Value};
use crate::auto_installer::CharonIoError;
use super::InstallationCmd;

pub enum ListMode { Simple, Verbose, Source }

pub fn update(cmd: &InstallationCmd, do_dry_run: bool) -> Result<String, CharonIoError> {
    // Keep a master list of all util info, mostly their version and source.
    // This will be used to do system updates.
    let mut table = load_main_index(do_dry_run)?;
    let info = get_info_from_cmd(&cmd);
    table.insert(cmd.name.clone(), info);

    // Write output
    let output = match toml::to_string(&table) {
        Ok(val) => val,
        Err(err) => return Err(CharonIoError::TomlSerError(err))
    };

    let root_path = crate::get_util_index_path(do_dry_run)?;
    let path = root_path.join("index.charon");
    if do_dry_run {
        let path = root_path.with_file_name("index.dry_run.charon");
        fs::write(path, &output)?;
    } else {
        fs::write(path, &output)?;
    }
    
    return Ok(output);
}

pub fn update_main_index(utils: Vec<String>) -> Result<String, CharonIoError> {
    let mut table = load_main_index(false)?;

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

pub fn list_main_index(mode: ListMode) {
    let table = match load_main_index(true) {
        Ok(t) => t,
        Err(_) => {
            printinfo!("No utils installed...");
            return;
        }
    };
    let func = match mode {
        ListMode::Verbose => {
            println!("Name\t\tVersion\t\tDescription");
            print_verbose
        },
        ListMode::Simple => print_simple,
        ListMode::Source => {
            println!("Name\t\tSource");
            print_source
        }
    };     

    for (key, val) in table {
        func(key, val);
    }
}

fn print_verbose(key: String, value: Value) {
    let mut msg = format!("{key}\t\t");

    let table = match value {
        Value::Table(t) => t,
        _ => {
            printinfo!("{msg}");
            return;
        }
    };

    if table.contains_key("version") {
        msg += &format!("{}", table.get("version").unwrap().to_string());
        msg += "\t";
    }
    msg += "\t";
    if table.contains_key("description") {
        msg += &format!("{}", table.get("description").unwrap().to_string());
    }


    printinfo!("{msg}");
}

fn print_simple(key: String, _value: Value) {
    printinfo!("{key}");
}

fn print_source(key: String, value: Value) {
    let mut msg = format!("{key}\t\t");

    let table = match value {
        Value::Table(t) => t,
        _ => {
            printinfo!("{msg}");
            return;
        }
    };

    if table.contains_key("source") {
        msg += &format!("{}", table.get("source").unwrap().to_string());
    }
    printinfo!("{msg}");
}

pub fn load_main_index(do_dry_run: bool) -> Result<Map<String, Value>, CharonIoError> {
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
    let table = match toml::from_str::<Value>(&file) {
        Ok(Value::Table(table)) => table,
        Ok(other) => {
            let msg = format!("Expected a table, found {other:?}.");
            return Err(CharonIoError::InvalidCharonFile(msg));
        },
        Err(err) => return Err(CharonIoError::TomlDeError(err)),
    };
    return Ok(table);
}

fn get_info_from_cmd(cmd: &InstallationCmd) -> Value {
    let mut output = Map::new();

    if let Some(version) = &cmd.version {
        output.insert("version".into(), Value::String(version.to_string()));
    }
    if let Some(source) = &cmd.source {
        output.insert("source".into(), Value::String(source.to_string()));
    }
    if let Some(desc) = &cmd.description {
        output.insert("description".into(), Value::String(desc.to_string()));
    }

    return Value::Table(output);
}

