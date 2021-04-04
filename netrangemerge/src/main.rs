mod commands;
mod sources;
mod utils;

use crate::commands::cloud::{
    cloud_filter_help_command, cloud_get_command, cloud_get_merge_command, cloud_get_read_command,
    cloud_merge_command, cloud_read_command,
};
use crate::commands::merge::merge_command;
use anyhow::Error;
use std::path::PathBuf;
use structopt::clap::AppSettings;
use structopt::StructOpt;

const CLOUD_SERVICE_NAMES: &'static [&'static str] = &["azure", "aws", "gcp"];

#[derive(Debug, StructOpt)]
pub struct CloudGetOptions {
    /// Cloud provider
    #[structopt(possible_values = CLOUD_SERVICE_NAMES)]
    pub service: String,
}

/// This is a test!!!
#[derive(Debug, StructOpt)]
pub struct CloudMergeOptions {
    /// Cloud provider
    #[structopt(possible_values = CLOUD_SERVICE_NAMES)]
    pub service: String,

    /// File to load the ip ranges from. STDIN is used if not
    /// specified.
    pub file: Option<PathBuf>,

    /// Lua filter program to select the ranges of interest.
    #[structopt(long, conflicts_with = "filter-file")]
    pub filter: Option<String>,

    /// Path of a file containing a Lua filter program to select the ranges of interest.
    #[structopt(long)]
    pub filter_file: Option<PathBuf>,

    /// By default, we include some (currently) known ranges when
    /// trying to minimize the output. If this option is set, we will
    /// ignore those ranges. This may produce a larger output set but
    /// may be useful in case out know ranges become incorrect in the future.
    #[structopt(long)]
    pub ignore_known_ranges: bool,

    /// A minimum ipv4 network size. Any ranges smaller that this size are automatically
    /// increased to this size. This option may help minimize the size of the output
    /// network ranges.
    #[structopt(long)]
    pub min_ipv4_network_size: Option<u8>,

    /// A minimum ipv6 network size. Any ranges smaller that this size are automatically
    /// increased to this size. This option may help minimize the size of the output
    /// network ranges.
    #[structopt(long)]
    pub min_ipv6_network_size: Option<u8>,
}

#[derive(Debug, StructOpt)]
pub struct CloudGetMergeOptions {
    /// Cloud provider
    #[structopt(possible_values = CLOUD_SERVICE_NAMES)]
    pub service: String,

    /// Lua filter program to select the ranges of interest.
    #[structopt(long, conflicts_with = "filter-file")]
    pub filter: Option<String>,

    /// Path of a file containing a Lua filter program to select the ranges of interest.
    #[structopt(long)]
    pub filter_file: Option<PathBuf>,

    /// By default, we include some (currently) known ranges when
    /// trying to minimize the output. If this option is set, we will
    /// ignore those ranges. This may produce a larger output set but
    /// may be useful in case out know ranges become incorrect in the future.
    #[structopt(long)]
    pub ignore_known_ranges: bool,

    /// A minimum ipv4 network size. Any ranges smaller that this size are automatically
    /// increased to this size. This option may help minimize the size of the output
    /// network ranges.
    #[structopt(long)]
    pub min_ipv4_network_size: Option<u8>,

    /// A minimum ipv6 network size. Any ranges smaller that this size are automatically
    /// increased to this size. This option may help minimize the size of the output
    /// network ranges.
    #[structopt(long)]
    pub min_ipv6_network_size: Option<u8>,
}

#[derive(Debug, StructOpt)]
pub struct CloudReadOptions {
    /// Cloud provider
    #[structopt(possible_values = CLOUD_SERVICE_NAMES)]
    pub service: String,

    /// File to load the ip ranges from. STDIN is used if not
    /// specified.
    pub file: Option<PathBuf>,

    /// Lua filter program to select the ranges of interest.
    #[structopt(long, conflicts_with = "filter-file")]
    pub filter: Option<String>,

    /// Path of a file containing a Lua filter program to select the ranges of interest.
    #[structopt(long)]
    pub filter_file: Option<PathBuf>,
}

#[derive(Debug, StructOpt)]
pub struct CloudGetReadOptions {
    /// Cloud provider
    #[structopt(possible_values = CLOUD_SERVICE_NAMES)]
    pub service: String,

    /// Lua filter program to select the ranges of interest.
    #[structopt(long, conflicts_with = "filter-file")]
    pub filter: Option<String>,

    /// Path of a file containing a Lua filter program to select the ranges of interest.
    #[structopt(long)]
    pub filter_file: Option<PathBuf>,
}

#[derive(Debug, StructOpt)]
pub struct CloudFilterHelpOptions {
    /// Cloud provider
    #[structopt(possible_values = CLOUD_SERVICE_NAMES)]
    pub service: String,
}

#[derive(Debug, StructOpt)]
pub struct MergeOptions {
    /// The file to read ranges from
    pub file: Option<PathBuf>,

    /// A file containing extra ranges to merge with the main set.
    /// These ranges will be used to minimize the main set - but will not
    /// otherwise appear in the output.
    #[structopt(long)]
    pub extra_file: Vec<PathBuf>,

    /// A minimum ipv4 network size. Any ranges smaller that this size are automatically
    /// increased to this size. This option may help minimize the size of the output
    /// network ranges.
    #[structopt(long)]
    pub min_ipv4_network_size: Option<u8>,

    /// A minimum ipv6 network size. Any ranges smaller that this size are automatically
    /// increased to this size. This option may help minimize the size of the output
    /// network ranges.
    #[structopt(long)]
    pub min_ipv6_network_size: Option<u8>,
}

#[derive(Debug, StructOpt)]
enum CloudCommands {
    Get {
        #[structopt(flatten)]
        options: CloudGetOptions,
    },
    Merge {
        #[structopt(flatten)]
        options: CloudMergeOptions,
    },
    GetMerge {
        #[structopt(flatten)]
        options: CloudGetMergeOptions,
    },
    Read {
        #[structopt(flatten)]
        options: CloudReadOptions,
    },
    GetRead {
        #[structopt(flatten)]
        options: CloudGetReadOptions,
    },
    FilterHelp {
        #[structopt(flatten)]
        options: CloudFilterHelpOptions,
    },
}

#[derive(Debug, StructOpt)]
#[structopt(global_setting = AppSettings::DeriveDisplayOrder)]
#[structopt(global_setting = AppSettings::UnifiedHelpMessage)]
#[structopt(global_setting = AppSettings::NextLineHelp)]
#[structopt(global_setting = AppSettings::VersionlessSubcommands)]
enum Commands {
    Cloud {
        #[structopt(flatten)]
        subcommand: CloudCommands,
    },
    Merge {
        #[structopt(flatten)]
        options: MergeOptions,
    },
}

// Merge from STDIN        | nrm cloud merge aws <file.json>
// Download then merge     | nrm cloud get-merge aws
// Download and save:      | nrm cloud read aws <file.json>
// Download and save:      | nrm cloud get-read aws <file.json>

// Help on filter options: | nrm cloud filter-help aws

fn main() -> Result<(), Error> {
    let opts = Commands::from_args();
    match opts {
        Commands::Cloud {
            subcommand: CloudCommands::Get { options },
        } => cloud_get_command(options)?,
        Commands::Cloud {
            subcommand: CloudCommands::Merge { options },
        } => cloud_merge_command(options)?,
        Commands::Cloud {
            subcommand: CloudCommands::GetMerge { options },
        } => cloud_get_merge_command(options)?,
        Commands::Cloud {
            subcommand: CloudCommands::Read { options },
        } => cloud_read_command(options)?,
        Commands::Cloud {
            subcommand: CloudCommands::GetRead { options },
        } => cloud_get_read_command(options)?,
        Commands::Cloud {
            subcommand: CloudCommands::FilterHelp { options },
        } => cloud_filter_help_command(options)?,

        Commands::Merge { options } => merge_command(options)?,
    }
    Ok(())
}
