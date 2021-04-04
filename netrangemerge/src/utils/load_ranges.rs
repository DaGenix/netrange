use crate::sources::{aws, azure, gcp};
use crate::utils::expand_ranges::expand_ranges;
use crate::utils::filter::{filter, NetworkWithMetadata};
use anyhow::{bail, Error};
use libnetrangemerge::{merge_ranges, IpRange, RangeInterest};
use std::fs::File;
use std::io;
use std::io::{BufWriter, Read, Write};
use std::path::PathBuf;
use std::str::FromStr as _;

pub fn load_ranges(
    service: &str,
    file: Option<&PathBuf>,
) -> Result<(Vec<NetworkWithMetadata>, &'static [&'static str]), Error> {
    let (ranges, known_ranges) = match service {
        "aws" => {
            let ranges = if let Some(file) = file {
                aws::load_ranges(File::open(&file)?)?
            } else {
                let stdin = io::stdin();
                aws::load_ranges(stdin.lock())?
            };
            (ranges, aws::SELECTED_KNOWN_AMAZON_IP_RANGES)
        }
        "azure" => {
            let ranges = if let Some(file) = file {
                azure::load_ranges(File::open(&file)?)?
            } else {
                let stdin = io::stdin();
                azure::load_ranges(stdin.lock())?
            };
            (ranges, azure::SELECTED_KNOWN_AZURE_IP_RANGES)
        }
        "gcp" => {
            let ranges = if let Some(file) = file {
                gcp::load_ranges(File::open(&file)?)?
            } else {
                let stdin = io::stdin();
                gcp::load_ranges(stdin.lock())?
            };
            let tmp: &'static [&str] = &[];
            (ranges, tmp)
        }
        x => bail!("Invalid service: {}", x),
    };

    Ok((ranges, known_ranges))
}

pub fn fetch_ranges(
    service: &str,
) -> Result<(Vec<NetworkWithMetadata>, &'static [&'static str]), Error> {
    let (ranges, known_ranges) = match service {
        "aws" => {
            let ranges = aws::load_ranges(aws::fetch_ranges()?)?;
            (ranges, aws::SELECTED_KNOWN_AMAZON_IP_RANGES)
        }
        "azure" => {
            let ranges = azure::load_ranges(azure::fetch_ranges()?)?;
            (ranges, azure::SELECTED_KNOWN_AZURE_IP_RANGES)
        }
        "gcp" => {
            let ranges = gcp::load_ranges(gcp::fetch_ranges()?)?;
            let tmp: &'static [&str] = &[];
            (ranges, tmp)
        }
        x => bail!("Invalid service: {}", x),
    };

    Ok((ranges, known_ranges))
}
