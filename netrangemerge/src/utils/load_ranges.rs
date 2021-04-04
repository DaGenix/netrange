use crate::sources::{aws, azure, gcp};
use crate::utils::filter::NetworkWithMetadata;
use anyhow::{bail, Error};
use std::fs::File;
use std::io;
use std::path::PathBuf;

pub fn fetch_and_load_ranges(
    service: &str,
) -> Result<(Vec<NetworkWithMetadata>, &'static [&'static str]), Error> {
    let (ranges, known_ranges) = match service {
        "aws" => {
            let ranges = aws::load_ranges(&mut aws::fetch_ranges()?)?;
            (ranges, aws::SELECTED_KNOWN_AMAZON_IP_RANGES)
        }
        "azure" => {
            let ranges = azure::load_ranges(&mut azure::fetch_ranges()?)?;
            (ranges, azure::SELECTED_KNOWN_AZURE_IP_RANGES)
        }
        "gcp" => {
            let ranges = gcp::load_ranges(&mut gcp::fetch_ranges()?)?;
            let tmp: &'static [&str] = &[];
            (ranges, tmp)
        }
        x => bail!("Invalid service: {}", x),
    };

    Ok((ranges, known_ranges))
}

pub fn load_ranges(
    service: &str,
    file: Option<&PathBuf>,
) -> Result<(Vec<NetworkWithMetadata>, &'static [&'static str]), Error> {
    let stdin = io::stdin();
    let (ranges, known_ranges) = match service {
        "aws" => {
            let ranges = if let Some(file) = file {
                aws::load_ranges(&mut File::open(&file)?)?
            } else {
                aws::load_ranges(&mut stdin.lock())?
            };
            (ranges, aws::SELECTED_KNOWN_AMAZON_IP_RANGES)
        }
        "azure" => {
            let ranges = if let Some(file) = file {
                azure::load_ranges(&mut File::open(&file)?)?
            } else {
                azure::load_ranges(&mut stdin.lock())?
            };
            (ranges, azure::SELECTED_KNOWN_AZURE_IP_RANGES)
        }
        "gcp" => {
            let ranges = if let Some(file) = file {
                gcp::load_ranges(&mut File::open(&file)?)?
            } else {
                gcp::load_ranges(&mut stdin.lock())?
            };
            let tmp: &'static [&str] = &[];
            (ranges, tmp)
        }
        x => bail!("Invalid service: {}", x),
    };

    Ok((ranges, known_ranges))
}
