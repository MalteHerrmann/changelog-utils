use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{config::config, errors::ChangeTypeError};

use super::entry::{self, MultiFileEntry};

#[derive(Clone, Debug)]
pub struct ChangeType {
    pub name: String,
    pub fixed: String,
    pub path: PathBuf,
    pub problems: Vec<String>,
    pub entries: Vec<MultiFileEntry>,
}

impl ChangeType {
    // TODO: this returns a string for the eventually generated full changelog. There should be another function for fixing the entries in the individual files!
    pub fn get_fixed_contents(&self) -> String {
        let mut exported_string = String::new();

        exported_string.push_str(&format!("### {}", &self.fixed));
        exported_string.push_str("\n\n");

        self.entries.iter().for_each(|entry| {
            exported_string.push_str(format!("{}\n", entry.fixed).as_str());
        });

        exported_string
    }
}

pub fn parse(config: &config::Config, dir: &Path) -> Result<ChangeType, ChangeTypeError> {
    let base_name = dir
        .file_name()
        .expect("no base name of path found")
        .to_str()
        .expect("failed to unpack base name string");

    let mut problems: Vec<String> = Vec::new();

    if !config
        .change_types
        .iter()
        .any(|ct| ct.long.to_ascii_lowercase().replace(" ", "-").eq(base_name))
    {
        problems.push(format!("invalid change type: {}", base_name));
    }

    let entries: Vec<MultiFileEntry> = fs::read_dir(dir)
        .expect("failed to read dir contents")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .filter_map(|p| match entry::parse(config, p.as_path()) {
            Ok(entry) => Some(entry),
            Err(_) => {
                problems.push(format!("invalid entry found in file: {}", p.display()));
                None
            }
        })
        .collect();

    Ok(ChangeType {
        name: base_name.into(),
        // TODO: when generating the full changelog this should be made uppercase then
        fixed: base_name.into(),
        path: dir.into(),
        problems,
        entries,
    })
}

// TODO: add more tests
#[cfg(test)]
mod tests {
    use crate::config::{unpack_config, Config};

    use super::*;

    fn load_example_config() -> Config {
        unpack_config(include_str!(
            "../../tests/testdata/multi_file/fail/.clconfig.json"
        ))
        .expect("failed to load multi file config")
    }

    #[test]
    fn test_fail() {
        let res = parse(
            &load_example_config(),
            Path::new("tests/testdata/multi_file/fail/.changelog/v9.0.0/features"),
        );
        assert!(res.is_ok());

        let ct = res.unwrap();
        assert_eq!(ct.entries.len(), 6);
    }
}
