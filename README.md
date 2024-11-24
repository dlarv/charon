# Charon
Installer for mythos projects, now fully divorced from mythos-core/plutonian-shores.

# Requirements
1. Reads a toml-style file containing installation instructions.
2. Remove deprecated files when updating utils.
3. Uninstall previously installed utils.
4. Provide a dry-run mode for testing.
5. Backwards compatible with previous charon files.

# Charon Files
Charon files are files ending with the '.charon' file extension. These are specially formatted toml files, with the form:
MYTHOS_DIR = [ SOURCE_ITEM\_1, SOURCE_ITEM_2, ... ]

Where MYTHOS_DIR corresponds with a mythos directory:
- Alias => MYTHOS_ALIAS_DIR
- Bin => MYTHOS_BIN_DIR
- Config => MYTHOS_CONFIG_DIR
- Data => MYTHOS_DATA_DIR
- LocalConfig => MYTHOS_LOCAL_CONFIG_DIR
- LocalData => MYTHOS_LOCAL_DATA_DIR

Each SOURCE_ITEM is a dict with any of the following fields:
- target: This is the local path to the source file being installed. **This field is required.**
- alias: If provided, the installed file will be renamed to this after being copied.
- perms: The permissions of the installed file. These should be in 0xXXX form.
- strip_ext: If true, the extension will be removed from the installed file.
- overwrite: If true, if the file already at the destination path, it will not be overwritten.
- comment: 

[{ target = \"path/to/local\", alias = \"alt_file_name\", perms = 0x544, strip_ext = false, overwrite = false, comment = \"\" }]

Charon files also have an optional info section:
info = { name = "charon", version = "0.2.3", description = "Basic installer utility" }

# Index File
When Charon installs a util, it creates an installation file, which contains a list of all relevant files. After a util is updated, the old and new installation files are compared. Any files found in the old, but not in the new are considered orphans and are deleted. This file is also used to uninstall utils.

