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

Normally, this file is saved to $MYTHOS_DATA_DIR/charon/\<util_name>.charon. However, when charon is used with the -n arg (dry run), this file is instead saved to $CWD/\<util_name>.dryrun.charon.

## Charon files have 2 different formats?
You may have noticed that the installation charon files use a different format from index files, despite both using the same file extension. Admittedly, this is an artifact from how charon files used to work. Originally, the index file format was used for both. I'm hoping to someday rectify this, by allowing simple installation instructions to be written in index format.

## Util index files vs Charon Index File
In addition to the index files discussed above (util index files), there is a file call $MYTHOS_DATA_DIR/charon/index.charon. This is a toml file that holds high level information about every util charon has installed. This is where the info field inside installation instructions ends up.

# Util Name
There are 3 methods charon uses to determine the name of the util it is currently installing.
- The info.name field inside the installation file.
- The filestem of the charon file.
- The name of the $CWD.


