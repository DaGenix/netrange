use crate::utils::cloud_process_ranges::cloud_process_ranges;
use crate::utils::load_ranges::{load_ranges, fetch_ranges};
use crate::{CloudReadOptions, CloudGetMergeOptions, CloudGetReadOptions};
use anyhow::Error;

pub fn cloud_get_read_command(options: CloudGetReadOptions) -> Result<(), Error> {
    let (ranges, known_ranges) = fetch_ranges(&options.service)?;

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
