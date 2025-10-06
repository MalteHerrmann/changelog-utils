use crate::multi_file::changelog::MultiFileChangelog;
use crate::single_file::changelog::SingleFileChangelog;

pub enum Changelog {
    Single(SingleFileChangelog),
    Multi(MultiFileChangelog),
}
