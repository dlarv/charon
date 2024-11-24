/*!
 * Installer for mythos projects, now fully divorced from mythos-core/plutonian-shores.
 * Reads a toml-style file containing installation instructions.
 */

mod auto_installer;

fn main() {
    // Find valid .charon file.
    // Parse .charon file => InstallationCmd.
    // Install files.
    // Check index.
    // Remove orphans.
    // Write (new) index.
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;
    use crate::*;

    /* #[test]
    fn find_charon_file_in_dir() {
        // let cmd = auto_install(PathBuf::from("tests/find_file_test/"));
        // assert!(cmd.is_some());
    }
    #[test]
    fn charon_file_empty() {
        // let cmd = auto_install(PathBuf::from("tests/find_file_test/empty.charon"));
        // assert!(cmd.is_some());
    }
    #[test]
    fn charon_file_dne() {
        // let cmd = auto_install(PathBuf::from("tests/"));
        // assert!(cmd.is_none());
    }
    #[test]
    fn invalid_dir() {
    }
    #[test]
    fn file_exists_and_overwrite() {
    }
    #[test]
    fn file_exists_and_no_overwrite() {
    }
    #[test]
    fn file_dne_and_no_overwrite() {
    }
    #[test]
    fn file_dne_and_overwrite() {
    }
    #[test]
    fn empty_dir_item() {
    }
    #[test]
    fn dir_dne() {
    }
    #[test]
    fn target_missing() {
    } */
}

