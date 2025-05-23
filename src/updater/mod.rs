use std::{fs, path::PathBuf};

use mythos_core::{cli::get_user_permission, printerror, printinfo, printwarn};
use toml::{map::Map, Value};

use crate::{auto_installer::CharonIoError, main_index};


pub fn update(do_dry_run: bool) -> Result<Vec<String>, CharonIoError> {
    //! Finds all mythos utils installed by charon and checks whether they should be updated.
    //! Returns a list of all utils that are changed.

    // Load index.charon
    // Iterate over utils
    // If util has source, lo
    let index = main_index::load_main_index(do_dry_run)?;

    let mut output: Vec<String> = Vec::new();
    let mut paths: Vec<PathBuf> = Vec::new();
    for (name, val) in index {
        printinfo!("\nChecking updates for {name}...");
        let info= match val.as_table() {
            Some(val) => val,
            None => {
                printwarn!("Could not read entry. Expected table, found {}. Skipping...", val.type_str());
                continue;
            }
        };

        // Load mandatory values: source, version.
        let version = match info.get("version") {
            Some(Value::String(version)) => version,
            Some(val) => {
                printwarn!("Could not parse version. Expected string, found {}. Skipping...", val.type_str());
                continue;
            },
            None => {
                printinfo!("No version number found for {name}. Skipping...");
                continue;
            }
        };
        let source_path = match info.get("source") {
            Some(Value::String(path)) => path,
            Some(val) => {
                printwarn!("Could not parse source path. Expected string, found {}. Skipping...", val.type_str());
                continue;
            },
            None => {
                printinfo!("No source path found for {name}. Skipping...");
                continue;
            }
        };

        let path = format!("{source_path}/{name}.charon");
        let local_charon = match load_local_charon(
            &path) {
            Ok(l) => l,
            Err(err) => {
                printinfo!("Error parsing charon file at {source_path}/{name}.charon. Error = {err}. Skipping...");
                continue;
            }
        };
        printinfo!("Found local source charon file at {path:?}");

        let local_info = match local_charon.get("info") {
            Some(info) => info,
            None => {
                printinfo!("Could not get info section from charon file. Skipping...");
                continue;
            }
        };

        let local_version= match local_info.get("version") {
            Some(Value::String(version)) => version,
            Some(val) => {
                printwarn!("Could not parse local version. Expected string, found {}. Skipping...", val.type_str());
                continue;
            },
            None => {
                printinfo!("No local version number found for {name}. Skipping...");
                continue;
            }
        };
        
        if let Some(val) = compare_versions(version, local_version) {
            if !val {
                printinfo!("No update found for {name}!");
                continue;
            }

            if do_dry_run {
                printinfo!("Updated {name} from v{version} --> v{local_version}");
            }             
            output.push(name);
            paths.push(path.into());
        }
    }
    
    println!("---------------------------------");

    if do_dry_run {
        return Ok(output);
    }

    if paths.len() == 0 {
        printinfo!("No updates found!");
        return Ok(output);
    }

    let msg = output.join("\n");
    if get_user_permission(false, 
        &format!("The following utils will be updated: \n{msg}\n")) {
        for path in paths {
            if let Err(err) = run_update(&path) {
                printerror!("{err}");
            }
        }
        printinfo!("Update completed!");
    } else {
        printinfo!("Update cancelled...");
    }
    return Ok(output);
}


pub fn force_update<T: Iterator<Item = String>>(utils: T, do_dry_run: bool) -> Result<(), CharonIoError> {
    let index = main_index::load_main_index(do_dry_run)?;

    let mut output: Vec<String> = Vec::new();
    let mut paths: Vec<PathBuf> = Vec::new();
    for util in utils {
        let entry = match index.get(&util) {
            Some(Value::Table(e)) => e,
            Some(p) => {
                printwarn!("Could not parse source path for {util}. Expected string, found {}. Skipping...", 
                    p.type_str());
                continue;
            },
            None => {
                printwarn!("Could not find util {util}. Skipping...");
                continue;
            }
        };

        let path = match entry.get("source") {
            Some(Value::String(p)) => p,
            Some(p) => {
                printwarn!("Could not parse source path for {util}. Expected string, found {}. Skipping...", 
                    p.type_str());
                continue;
            },
            None => {
                printwarn!("Could not find source path for {util}. Skipping...");
                continue;
            }
        };

        output.push(util.to_string());
        paths.push(PathBuf::from(path));
    }

    let mut i: isize = -1;
    let msg = format!(
        "Found source paths for the following source paths:\n{}\n\nWould you like to continue?",
        output.iter().map(|x| { 
            i += 1;
            format!("{x}\t\t{:?}", paths[i as usize])
        }).collect::<Vec<String>>().join("\n")
    );

    if !get_user_permission(false, &msg) {
        printinfo!("Installation cancelled...");
        return Ok(());
    }

    for (util, path) in output.iter().zip(paths) {
        if do_dry_run {
            printinfo!("Finished updating {util}!");
            continue;
        }

        match run_update(&path) {
            Ok(_) => printinfo!("Finished updating {util}!"),
            Err(err) => printerror!("Could not update {util}. Error = {err}")
        }
    }
    return Ok(());
}


fn load_local_charon(root_path: &str) -> Result<Map<String, Value>, std::io::Error> {
    let path = PathBuf::from(root_path);
    let contents = fs::read_to_string(&path)?;    
    return match toml::from_str::<Value>(&contents) {
        Ok(Value::Table(table)) => Ok(table),
        _ => return Err(std::io::ErrorKind::InvalidData.into())
    };
}

fn compare_versions(old: &str, new: &str) -> Option<bool> {
    //! Returns true if new > old.
    let v1 = old.split(".");
    let v2 = new.split(".");

    // old = 0.0.1 new = 0.0.1 => false
    // old = 0.0.2 new = 0.0.1 => false
    // old = 0.0.1.1 new = 0.0.1 => false
    // old = 0.0.2 new = 0.0.1.1 => false
    //
    // old = 0.0.1 new = 0.0.2 => true
    // old = 0.0.1 new = 0.0.1.1 => true
    // old = 0.0.1.1 new = 0.0.2 => true

    // for (v1, v2) in (v1.next(), v2.next()){
    //     if v2.parse::<i32>().ok()? > v1.parse::<i32>().ok()?{
    //         return Some(true);
    //     }
    // }

    for (v1, v2) in zip_uneven(v1, v2) {
        if v2.is_none() {
            return Some(false);
        } else if v1.is_none() {
            return Some(true);
        }

        let num1 = v1.unwrap().parse::<i32>().ok()?;
        let num2 = v2.unwrap().parse::<i32>().ok()?;

        if num1 == num2 { 
            continue; 
        } else if num1 > num2 {
            return Some(false);
        } else {
            return Some(true);
        }
    }

    // Two versions are identical.
    return Some(false);
}

fn zip_uneven<T, U> (mut iter_1: U, mut iter_2: U) -> impl Iterator<Item = (Option<T>, Option<T>)> 
where 
    U: Iterator<Item = T> {

    let mut output: Vec<(Option<T>, Option<T>)> = Vec::new();
    loop {
        let v1 = iter_1.next();
        let v2 = iter_2.next();

        if v1.is_none() && v2.is_none() {
            break;
        }
        output.push((v1, v2));

    }

    return output.into_iter();
}

fn run_update(path: &PathBuf) -> Result<(), CharonIoError> {
    crate::install(path, false)?;
    return Ok(());
}

#[cfg(test)]
mod tests {
    use std::env;
    use serial_test::serial;
    use super::update;

    #[serial]
    #[test]
    fn test_update() {
        unsafe {
            env::set_var("MYTHOS_DATA_DIR", "tests/updater");
        }

        let output = update(true).unwrap();
        assert_eq!(output, vec!["a", "b"]);
    }

    #[serial]
    #[test]
    fn test_invalid_update() {
        todo!()
    }

    #[test]
    fn test_compare_versions() {
       todo!() 
    }
}
