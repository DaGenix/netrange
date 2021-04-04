use crate::sources::{aws, azure, gcp};
use crate::utils::cloud_process_ranges::cloud_process_ranges;
use crate::utils::load_ranges::{fetch_and_load_ranges, load_ranges};
use crate::{
    CloudFilterHelpOptions, CloudGetMergeOptions, CloudGetOptions, CloudGetReadOptions,
    CloudMergeOptions, CloudReadOptions,
};
use anyhow::{bail, Error};
use std::io;

pub fn cloud_get_command(options: CloudGetOptions) -> Result<(), Error> {
    let mut response = match options.service.as_str() {
        "aws" => aws::fetch_ranges()?,
        "azure" => azure::fetch_ranges()?,
        "gcp" => gcp::fetch_ranges()?,
        x => bail!("Invalid service: {}", x),
    };

    io::copy(&mut response, &mut io::stdout().lock())?;

    Ok(())
}

pub fn cloud_merge_command(options: CloudMergeOptions) -> Result<(), Error> {
    let (ranges, known_ranges) = load_ranges(&options.service, options.file.as_ref())?;

    cloud_process_ranges(
        ranges,
        known_ranges,
        options.filter,
        options.filter_file,
        options.ignore_known_ranges,
        options.min_ipv4_network_size,
        options.min_ipv6_network_size,
        true,
    )?;

    Ok(())
}

pub fn cloud_get_merge_command(options: CloudGetMergeOptions) -> Result<(), Error> {
    let (ranges, known_ranges) = fetch_and_load_ranges(&options.service)?;

    cloud_process_ranges(
        ranges,
        known_ranges,
        options.filter,
        options.filter_file,
        options.ignore_known_ranges,
        options.min_ipv4_network_size,
        options.min_ipv6_network_size,
        true,
    )?;

    Ok(())
}

pub fn cloud_read_command(options: CloudReadOptions) -> Result<(), Error> {
    let (ranges, known_ranges) = load_ranges(&options.service, options.file.as_ref())?;

    cloud_process_ranges(
        ranges,
        known_ranges,
        options.filter,
        options.filter_file,
        true,
        None,
        None,
        false,
    )?;

    Ok(())
}

pub fn cloud_get_read_command(options: CloudGetReadOptions) -> Result<(), Error> {
    let (ranges, known_ranges) = fetch_and_load_ranges(&options.service)?;

    cloud_process_ranges(
        ranges,
        known_ranges,
        options.filter,
        options.filter_file,
        true,
        None,
        None,
        false,
    )?;

    Ok(())
}

pub fn cloud_filter_help_command(options: CloudFilterHelpOptions) -> Result<(), Error> {
    let message = match options.service.as_str() {
        "aws" => aws::FILTER_HELP,
        "azure" => azure::FILTER_HELP,
        "gcp" => gcp::FILTER_HELP,
        x => bail!("Invalid service: {}", x),
    };
    println!("{}", message);
    Ok(())
}
