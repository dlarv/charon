use std::path::PathBuf;

use super::InstallItem;

impl InstallItem {
    pub fn new() -> InstallItem {
        return InstallItem {
            target: PathBuf::new(),
            dest: PathBuf::new(),
            perms: 0,
            strip_ext: false,
            alias: None,
            overwrite: true,
            comment: "".into(),
        };
    }
    pub fn target(&mut self, target: PathBuf) -> &mut InstallItem {
        self.target = target;
        return self;
    }
    pub fn perms(&mut self, val: u32) -> &mut InstallItem {
        self.perms = val;
        return self;
    }
    pub fn strip_ext(&mut self, val: bool) -> &mut InstallItem {
        self.strip_ext = val;
        return self;
    }
    pub fn overwrite(&mut self, val: bool) -> &mut InstallItem {
        self.overwrite = val;
        return self;
    }
    pub fn comment(&mut self, val: String) -> &mut InstallItem {
        self.comment = val;
        return self;
    }
    pub fn print_dest(&self) -> String {
        return self.dest.to_string_lossy().to_string();
    }
    pub fn to_toml_str(&self) -> String {
        let mut output = format!("{{ target = {:?}", self.target);

        if self.perms > 0 {
            output += &format!(", perms = {}", self.perms);
        }
        if self.strip_ext {
            output += ", strip_ext = true";
        }
        if !self.overwrite {
            output += ", overwrite = true";
        }
        if let Some(alias) = &self.alias {
            output += &format!(", alias = {alias:?}");
        }
        if self.comment.len() > 0 {
            output += &format!(", comment = {:?}", self.comment);
        }
        output += "}";
        return output;
    }
}

