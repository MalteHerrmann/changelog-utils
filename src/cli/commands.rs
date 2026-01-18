use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version)]
pub enum ChangelogCLI {
    #[command(about = "Adds a new entry to the unreleased section of the changelog")]
    Add(AddArgs),
    #[command(about = "Does basic checks for the setup of the tool")]
    Check,
    #[command(
        about = "Checks if a changelog entry was created for a pull request related to the current branch"
    )]
    CheckDiff,
    #[command(
        about = "Creates a PR in the configured target repository and adds the corresponding changelog entry"
    )]
    CreatePR,
    #[command(about = "Applies all possible auto-fixes to the changelog")]
    Fix,
    #[command(
        about = "Gets the contents of a specific version's release notes from the changelog"
    )]
    Get(GetArgs),
    #[command(about = "Checks if the changelog contents adhere to the defined rules")]
    Lint,
    #[command(about = "Initializes the changelog configuration in the current directory")]
    #[command(long_about = r#"
Initializes the changelog configuration in the current directory.
It creates an empty changelog skeleton if no existing changelog is found as well as a default configuration for the tool.
"#)]
    Init,
    #[command(subcommand)]
    #[command(
        about = "Adjust the changelog configuration like allowed categories, change types or other"
    )]
    Config(ConfigSubcommands),
    #[command(about = "Turns the Unreleased section into a new release with the given version")]
    Release(ReleaseArgs),
}

#[derive(Args, Debug)]
pub struct AddArgs {
    pub number: Option<u64>,
    #[arg(short, long)]
    pub yes: bool,
}

#[derive(Args, Debug)]
pub struct GetArgs {
    pub version: String,
}

#[derive(Subcommand, Debug)]
pub enum ConfigSubcommands {
    #[command(about = "Adjust the allowed categories for changelog entries")]
    Category(ConfigArgs),
    #[command(
        about = "Adjust the allowed change types within releases (like 'Bug Fixes', 'Features', etc.)"
    )]
    ChangeType(ChangeTypeArgs),
    #[command(about = "Set or unset the optional legacy version")]
    LegacyVersion(ConditionalArgs),
    #[command(about = "Shows the current configuration")]
    Show,
    #[command(about = "Adjust the expected spellings that should be enforced in the changelog")]
    Spelling(KeyValueArgs),
    #[command(about = "Sets the target repository for the changelog entries")]
    TargetRepo(StringValue),
    #[command(about = "Sets the changelog directory for multi-mode")]
    ChangelogDir(ConditionalArgs),
    #[command(about = "Sets the changelog mode (single or multi)")]
    Mode(StringValue),
    #[command(about = "Sets whether categories are enforced in entries")]
    UseCategories(ConditionalArgs),
    #[command(about = "Migrates the configuration to the current version")]
    Migrate,
}

#[derive(Args, Debug)]
pub struct StringValue {
    pub value: String,
}

#[derive(Args, Debug)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: CategoryOperation,
}

#[derive(Args, Debug)]
pub struct ReleaseArgs {
    pub version: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum CategoryOperation {
    #[command(about = "Adds a new category to the list of allowed ones")]
    Add { value: String },
    #[command(about = "Removes a category if it is set in the configuration")]
    Remove { value: String },
}

#[derive(Args, Debug)]
pub struct ChangeTypeArgs {
    #[command(subcommand)]
    pub command: ChangeTypeConfigOperation,
}

#[derive(Debug, Subcommand)]
pub enum ChangeTypeConfigOperation {
    #[command(about = "Add a new change type configuration entry")]
    Add { long: String, short: String },
    #[command(about = "Remove a change type configuration entry")]
    Remove { short: String },
}

#[derive(Args, Debug)]
pub struct KeyValueArgs {
    #[command(subcommand)]
    pub command: KeyValueOperation,
}

#[derive(Debug, Subcommand)]
pub enum KeyValueOperation {
    #[command(about = "Adds a new key-value pair to the configuration")]
    Add { key: String, value: String },
    #[command(about = "Removes a key if it is found in the hash map")]
    Remove { key: String },
}

#[derive(Args, Debug)]
pub struct ConditionalArgs {
    #[command(subcommand)]
    pub command: OptionalOperation,
}

#[derive(Debug, Subcommand)]
pub enum OptionalOperation {
    #[command(about = "Sets the optional value")]
    Set { value: String },
    #[command(about = "Unsets the optional value")]
    Unset,
}
