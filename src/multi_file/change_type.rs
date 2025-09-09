use std::{fs, path::Path};

use crate::{config::config, errors::ChangeTypeError};

use super::entry::{self, MultiFileEntry};

#[derive(Clone, Debug)]
pub struct ChangeType {
    pub name: String,
    pub fixed: String,
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
    // TODO: use match for correct error handling here?
    let base_name = dir
        .file_name()
        .expect("no base name of path found")
        .to_str()
        .expect("failed to unpack base name string");

    let mut problems: Vec<String> = Vec::new();

    // TODO: more advanced checking here?
    if !config
        .change_types
        .iter()
        .any(|ct| ct.long.to_ascii_lowercase().eq(base_name))
    {
        problems.push(format!("invalid change type: {}", base_name));
    }

    let entries = fs::read_dir(dir)
        .expect("failed to read dir contents")
        .into_iter()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .filter_map(|p| entry::parse(config, p.as_path()).ok())
        .collect();

    Ok(ChangeType {
        name: base_name.into(),
        // TODO: I guess this should rather be lowercase, but rather only when generating the
        // full changelog based on the individual entries.
        fixed: base_name.into(), // TODO: capitalize
        problems,
        entries,
    })
}
