use crate::utils::cloud_process_ranges::cloud_process_ranges;
use crate::CloudReadOptions;
use anyhow::Error;

pub fn cloud_read_command(options: CloudReadOptions) -> Result<(), Error> {
    cloud_process_ranges(
        &options.service,
        options.file,
        options.filter,
        options.filter_file,
        true,
        None,
        None,
        false,
    )?;

    Ok(())
}
