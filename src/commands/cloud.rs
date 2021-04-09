use crate::utils::cloud_config::get_cloud_config;
use crate::utils::cloud_process_ranges::cloud_process_ranges;
use crate::utils::load_ranges::{fetch_and_load_ranges, load_ranges};
use crate::{
    CloudFilterHelpOptions, CloudGetMergeOptions, CloudGetOptions, CloudGetReadOptions,
    CloudMergeOptions, CloudReadOptions,
};
use anyhow::Error;
use std::io;

pub fn cloud_get_command(options: CloudGetOptions) -> Result<(), Error> {
    let func = get_cloud_config(&options.service)?.fetch_ranges_func;
    let mut response = func()?;
    io::copy(&mut response, &mut io::stdout().lock())?;
    Ok(())
}

pub fn cloud_merge_command(options: CloudMergeOptions) -> Result<(), Error> {
    let ranges = load_ranges(&options.service, options.file.as_ref())?;

    cloud_process_ranges(
        ranges,
        options.filter,
        options.filter_file,
        options.select,
        options.select_file,
        options.min_ipv4_network_size,
        options.min_ipv6_network_size,
        true,
    )?;

    Ok(())
}

pub fn cloud_get_merge_command(options: CloudGetMergeOptions) -> Result<(), Error> {
    let ranges = fetch_and_load_ranges(&options.service)?;

    cloud_process_ranges(
        ranges,
        options.filter,
        options.filter_file,
        options.select,
        options.select_file,
        options.min_ipv4_network_size,
        options.min_ipv6_network_size,
        true,
    )?;

    Ok(())
}

pub fn cloud_read_command(options: CloudReadOptions) -> Result<(), Error> {
    let ranges = load_ranges(&options.service, options.file.as_ref())?;

    cloud_process_ranges(
        ranges,
        options.filter,
        options.filter_file,
        None,
        None,
        None,
        None,
        false,
    )?;

    Ok(())
}

pub fn cloud_get_read_command(options: CloudGetReadOptions) -> Result<(), Error> {
    let ranges = fetch_and_load_ranges(&options.service)?;

    cloud_process_ranges(
        ranges,
        options.filter,
        options.filter_file,
        None,
        None,
        None,
        None,
        false,
    )?;

    Ok(())
}

pub fn cloud_filter_help_command(options: CloudFilterHelpOptions) -> Result<(), Error> {
    println!("{}", get_cloud_config(&options.service)?.filter_help);
    Ok(())
}
