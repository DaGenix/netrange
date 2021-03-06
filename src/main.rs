mod commands;
mod sources;
mod utils;

use crate::commands::cloud::{
    cloud_filter_help_command, cloud_get_command, cloud_get_merge_command, cloud_get_read_command,
    cloud_merge_command, cloud_read_command,
};
use crate::commands::merge::merge_command;
use crate::utils::cloud_config::get_cloud_names;
use anyhow::Error;
use std::path::PathBuf;
use structopt::StructOpt;

/// Download the source file that contains the IP ranges that the service uses.
///
/// Many, but not all, cloud services use a JSON formatted file to provide
/// the IP ranges that they use.
#[derive(Debug, StructOpt)]
pub struct CloudGetOptions {
    /// Cloud service
    #[structopt(possible_values = get_cloud_names())]
    pub service: String,
}

/// Load IP ranges for the service, merge adjacent ranges, and output to STDOUT.
///
/// The ranges loaded may be filtered to limit those that are output
/// by attributes provided by the service (eg, "region" for AWS). After
/// filtering, remaining ranges may be selected using those same attributes.
/// Ranges marked as selected will always appear in the output set,
/// while unselected ranges are used to help minimize the output set and
/// may not appear in the output if they do not help minimize it.
///
/// Both selecting and filtering are done with LUA programs that should
/// return either a True or False result for each set of attributes.
/// You may use the "cloud filter-help <service>" command to see what
/// filtering parameters are available by service.
///
/// This subcommand requires that the source IP ranges already have been
/// downloaded such as with the "cloud get <service>" command. You may
/// download and merge in just a single command by using the
/// "cloud get-merge <service>" command instead.
#[derive(Debug, StructOpt)]
pub struct CloudMergeOptions {
    /// Cloud service
    #[structopt(possible_values = get_cloud_names())]
    pub service: String,

    /// File to load the ip ranges from. STDIN is used if
    /// file is "-".
    pub file: PathBuf,

    /// Lua filter program to filter the ranges of interest.
    #[structopt(long, conflicts_with = "filter-file")]
    pub filter: Option<String>,

    /// Path of a file containing a Lua program to filter the ranges of interest.
    #[structopt(long)]
    pub filter_file: Option<PathBuf>,

    /// Lua filter program to select the ranges of interest.
    #[structopt(long, conflicts_with = "select-file")]
    pub select: Option<String>,

    /// Path of a file containing a Lua program to select the ranges of interest.
    #[structopt(long)]
    pub select_file: Option<PathBuf>,

    /// Extra ranges that may be helpful to minimize the set
    ///
    /// A file containing extra ranges to merge with the main set.
    /// The file should contain a single CIDR range per line.
    /// These ranges will be used to minimize the main set in the same way
    /// that non-selected ranges are.
    #[structopt(name = "extra-ranges-file", long)]
    pub extra_ranges_files: Vec<PathBuf>,

    /// A minimum ipv4 network size.
    ///
    /// Any ranges smaller that this size are automatically
    /// increased to this size. This option may help minimize the size of the output
    /// network ranges.
    #[structopt(long)]
    pub min_ipv4_network_size: Option<u8>,

    /// A minimum ipv6 network size.
    ///
    /// Any ranges smaller that this size are automatically
    /// increased to this size. This option may help minimize the size of the output
    /// network ranges.
    #[structopt(long)]
    pub min_ipv6_network_size: Option<u8>,
}

/// download ip ranges for the given service and then try to minimize the set.
///
/// the ranges loaded may be filtered to limit those that are output
/// by attributes provided by the service (eg, "region" for aws). after
/// filtering, remaining ranges may be selected using those same attributes.
/// ranges marked as selected will always appear in the output set,
/// while unselected ranges are used to help minimize the output set and
/// may not appear in the output if they do not help minimize it.
///
/// both selecting and filtering are done with lua programs that should
/// return either a true or false result for each set of attributes.
/// You may use the "cloud filter-help <service>" command to see what
/// filtering parameters are available by service.
///
/// This command will re-download the source IP ranges every time it is
/// invoked. This can be inefficient if you invoke this command multiple
/// times. In such a case, you may want to consider using "cloud get <service>"
/// to download the file once and then use "cloud merge <service>" to process
/// the already downloaded file.
#[derive(Debug, StructOpt)]
pub struct CloudGetMergeOptions {
    /// Cloud service
    #[structopt(possible_values = get_cloud_names())]
    pub service: String,

    /// Lua filter program to filter the ranges of interest.
    #[structopt(long, conflicts_with = "filter-file")]
    pub filter: Option<String>,

    /// Path of a file containing a Lua program to filter the ranges of interest.
    #[structopt(long)]
    pub filter_file: Option<PathBuf>,

    /// Lua filter program to select the ranges of interest.
    #[structopt(long, conflicts_with = "select-file")]
    pub select: Option<String>,

    /// Path of a file containing a Lua program to select the ranges of interest.
    #[structopt(long)]
    pub select_file: Option<PathBuf>,

    /// Extra ranges that may be helpful to minimize the set
    ///
    /// A file containing extra ranges to merge with the main set.
    /// The file should contain a single CIDR range per line.
    /// These ranges will be used to minimize the main set in the same way
    /// that non-selected ranges are.
    #[structopt(name = "extra-ranges-file", long)]
    pub extra_ranges_files: Vec<PathBuf>,

    /// A minimum ipv4 network size.
    ///
    /// Any ranges smaller that this size are automatically
    /// increased to this size. This option may help minimize the size of the output
    /// network ranges.
    #[structopt(long)]
    pub min_ipv4_network_size: Option<u8>,

    /// A minimum ipv6 network size.
    ///
    /// Any ranges smaller that this size are automatically
    /// increased to this size. This option may help minimize the size of the output
    /// network ranges.
    #[structopt(long)]
    pub min_ipv6_network_size: Option<u8>,
}

/// Load IP ranges for the given service and print them out
///
/// The ranges loaded may be filtered to limit those that are output
/// by attributes provided by the service (eg, "region" for AWS) using
/// a LUA program that should return either a True or False result
/// for each set of attributes. You may use the "cloud filter-help <service>"
/// command to see what filtering parameters are available by service.
///
/// This command requires that the source IP ranges already have been
/// downloaded such as with the "cloud get <service>" command. You may
/// download and merge in just a single command by using the
/// "cloud get-read <service>" command instead.
#[derive(Debug, StructOpt)]
pub struct CloudReadOptions {
    /// Cloud service
    #[structopt(possible_values = get_cloud_names())]
    pub service: String,

    /// File to load the ip ranges from. STDIN is used if
    /// file is "-".
    pub file: PathBuf,

    /// Lua filter program to select the ranges of interest.
    #[structopt(long, conflicts_with = "filter-file")]
    pub filter: Option<String>,

    /// Path of a file containing a Lua filter program to select the ranges of interest.
    #[structopt(long)]
    pub filter_file: Option<PathBuf>,
}

/// Load IP ranges for the given service and print them out
///
/// The ranges loaded may be filtered to limit those that are output
/// by attributes provided by the service (eg, "region" for AWS) using
/// a LUA program that should return either a True or False result
/// for each set of attributes. You may use the "cloud filter-help <service>"
/// command to see what filtering parameters are available by service.
///
/// This command will re-download the source IP ranges every time it is
/// invoked. This can be inefficient if you invoke this command multiple
/// times. In such a case, you may want to consider using "cloud get <service>"
/// to download the file once and then use "cloud merge <service>" to process
/// the already downloaded file.
#[derive(Debug, StructOpt)]
pub struct CloudGetReadOptions {
    /// Cloud service
    #[structopt(possible_values = get_cloud_names())]
    pub service: String,

    /// Lua filter program to select the ranges of interest.
    #[structopt(long, conflicts_with = "filter-file")]
    pub filter: Option<String>,

    /// Path of a file containing a Lua filter program to select the ranges of interest.
    #[structopt(long)]
    pub filter_file: Option<PathBuf>,
}

/// Print information about parameters available to filter ranges
#[derive(Debug, StructOpt)]
pub struct CloudFilterHelpOptions {
    /// Cloud service
    #[structopt(possible_values = get_cloud_names())]
    pub service: String,
}

/// Merge IP ranges to try to minimize the number of ranges
///
/// The source ranges should be provided either in a file or
/// via STDIN with a single range per line.
///
/// The minimized set of ranges will be printed to STDOUT.
#[derive(Debug, StructOpt)]
pub struct MergeOptions {
    /// The file to read ranges from. STDIN is used if
    /// file is "-".
    pub file: PathBuf,

    /// Extra ranges that may be helpful to minimize the set
    ///
    /// A file containing extra ranges to merge with the main set.
    /// The file should contain a single CIDR range per line.
    /// These ranges will be used to minimize the main set in the same way
    /// that non-selected ranges are with the "cloud merge" subcommand.
    #[structopt(name = "extra-ranges-file", long)]
    pub extra_ranges_files: Vec<PathBuf>,

    /// A minimum ipv4 network size.
    ///
    /// Any ranges smaller that this size are automatically
    /// increased to this size. This option may help minimize the size of the output
    /// network ranges.
    #[structopt(long)]
    pub min_ipv4_network_size: Option<u8>,

    /// A minimum ipv6 network size.
    ///
    /// Any ranges smaller that this size are automatically
    /// increased to this size. This option may help minimize the size of the output
    /// network ranges.
    #[structopt(long)]
    pub min_ipv6_network_size: Option<u8>,
}

/// Commands for working with cloud service's IP ranges
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

/// netrangemerge provides a command line interface to retrieve,
/// filter, and merge adjacent IP ranges for various cloud
/// services.
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
