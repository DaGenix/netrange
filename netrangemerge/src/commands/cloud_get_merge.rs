use crate::utils::cloud_process_ranges::cloud_process_ranges;
use crate::utils::load_ranges::{load_ranges, fetch_ranges};
use crate::{CloudReadOptions, CloudGetMergeOptions};
use anyhow::Error;

pub fn cloud_get_merge_command(options: CloudGetMergeOptions) -> Result<(), Error> {
    let (ranges, known_ranges) = fetch_ranges(&options.service)?;

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
