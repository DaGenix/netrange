mod commands;
mod providers;

use crate::commands::download_sources::download_sources;
use crate::commands::get_ranges::get_ranges_command;
use anyhow::Error;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct GetRangesOptions {
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
pub struct DownloadSources {
    /// Network type to process ranges for ("azure", "aws", or "gcp")
    pub service: String,

    /// The file to write the data to ("-" for STDOUT)
    #[structopt(short, long)]
    pub file: String,
}

#[derive(Debug, StructOpt)]
enum Commands {
    GetRanges {
        #[structopt(flatten)]
        options: GetRangesOptions,
    },
    DownloadSources {
        #[structopt(flatten)]
        options: DownloadSources,
    },
}

fn main() -> Result<(), Error> {
    let opts = Commands::from_args();
    match opts {
        Commands::GetRanges { options } => get_ranges_command(options)?,
        Commands::DownloadSources { options } => download_sources(options)?,
    }
    Ok(())
}
