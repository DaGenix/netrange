mod aws;

use crate::aws::aws_command;
use anyhow::Error;

use std::path::PathBuf;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct AwsOptions {
    /// File to load the AWS ip ranges from. If not specified, the ranges
    /// are fetched from https://ip-ranges.amazonaws.com/ip-ranges.json
    #[structopt(short, long)]
    pub file: Option<PathBuf>,

    /// Lua filter program to select the ranges of interest.
    /// By default, this will select us-east-1 ranges used by
    /// EC2.
    #[structopt(
        long,
        default_value = "return region == 'us-east-1' and service == 'EC2'"
    )]
    pub filter: String,
}

#[derive(Debug, StructOpt)]
enum Commands {
    Aws {
        #[structopt(flatten)]
        options: AwsOptions,
    },
}

fn main() -> Result<(), Error> {
    let opts = Commands::from_args();
    match opts {
        Commands::Aws { options } => aws_command(options)?,
    }
    Ok(())
}
