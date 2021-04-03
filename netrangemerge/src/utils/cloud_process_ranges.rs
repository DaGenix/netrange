use crate::sources::{aws, azure, gcp};
use crate::CloudReadOptions;
use anyhow::{bail, Error};
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use crate::utils::filter::filter;
use std::io::{Read, BufWriter, Write};
use crate::utils::expand_ranges::expand_ranges;
use libnetrangemerge::{Range as _, RangeInterest, IpRange, merge_ranges};
use std::str::FromStr as _;

pub fn cloud_process_ranges(
    service: &str,
    file: Option<PathBuf>,
    filter_program: Option<String>,
    filter_file: Option<PathBuf>,
    ignore_known_ranges: bool,
    min_ipv4_network_size: Option<u8>,
    min_ipv6_network_size: Option<u8>,
    do_merge: bool,
) -> Result<(), Error> {
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

    let mut filtered_ranges = if let Some(filter_program) = filter_program {
        filter(ranges, Some(&filter_program))?
    } else if let Some(filter_file) = filter_file {
        let mut filter_program = String::new();
        File::open(&filter_file)?.read_to_string(&mut filter_program)?;
        filter(ranges, Some(&filter_program))?
    } else {
        filter(ranges, None)?
    };

    expand_ranges(
        filtered_ranges.iter_mut().map(|n| n.range_mut()),
        min_ipv4_network_size,
        min_ipv6_network_size,
    );

    if !ignore_known_ranges {
        for known_range in known_ranges {
            filtered_ranges.push(RangeInterest::new(
                IpRange::from_str(known_range).unwrap(),
                false,
            ));
        }
    }

    if do_merge {
        merge_ranges(&mut filtered_ranges);
    }

    let stdout = io::stdout();
    let mut stdout = BufWriter::new(stdout.lock());
    for network in filtered_ranges {
        if network.is_interesting() {
            writeln!(stdout, "{}", network.range())?;
        }
    }
    stdout.flush()?;

    Ok(())
}
