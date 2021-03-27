mod commands;
mod sources;
mod utils;

use crate::commands::cloud_download_source::cloud_download_source_command;
use crate::commands::cloud_get::cloud_get_command;
use crate::commands::merge::merge_command;
use anyhow::Error;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CloudGetOptions {
    /// Network type to process ranges for ("azure", "aws", or "gcp")
    pub service: String,

    /// File to load the ip ranges from. If not specified, the ranges
    /// are fetched from the the service provider.
    #[structopt(short, long)]
    pub file: Option<PathBuf>,

    /// Lua filter program to select the ranges of interest.
    #[structopt(long)]
    pub filter: Option<String>,

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
pub struct CloudDownloadSourceOptions {
    /// Network type to process ranges for ("azure", "aws", or "gcp")
    pub service: String,

    /// The file to write the data to ("-" for STDOUT)
    #[structopt(short, long)]
    pub file: PathBuf,
}

#[derive(Debug, StructOpt)]
pub struct MergeOptions {
    /// The files to read data from ("-" for STDIN)
    pub file: PathBuf,

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
    DownloadSource {
        #[structopt(flatten)]
        options: CloudDownloadSourceOptions,
    },
}

#[derive(Debug, StructOpt)]
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

fn main() -> Result<(), Error> {
    let opts = Commands::from_args();
    match opts {
        Commands::Cloud {
            subcommand: CloudCommands::Get { options },
        } => cloud_get_command(options)?,
        Commands::Cloud {
            subcommand: CloudCommands::DownloadSource { options },
        } => cloud_download_source_command(options)?,
        Commands::Merge { options } => merge_command(options)?,
    }
    Ok(())
}
