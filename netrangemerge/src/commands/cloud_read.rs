use crate::utils::cloud_process_ranges::cloud_process_ranges;
use crate::utils::load_ranges::load_ranges;
use crate::CloudReadOptions;
use anyhow::Error;

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
