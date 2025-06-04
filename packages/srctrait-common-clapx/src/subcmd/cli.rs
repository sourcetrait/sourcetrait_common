use clap;

/// Customize command-line interface and environment
#[derive(Debug, clap::Subcommand)]
pub enum CliCommand {
    Alias(CliAliasCommand),
    Config(CliConfigCommand),
}

/// Configure the shell alias used for this command
#[derive(Debug, clap::Parser)]
pub struct CliAliasCommand {
    /// The name to alias this command to
    pub name: String,
}

/// View and edit the configuration for this command
#[derive(Debug, clap::Parser)]
pub struct CliConfigCommand {}

