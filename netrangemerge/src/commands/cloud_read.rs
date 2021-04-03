use crate::sources::{aws, azure, gcp};
use crate::CloudReadOptions;
use anyhow::{bail, Error};
use std::fs::File;
use std::io;
use std::path::Path;
use crate::utils::cloud_process_ranges::cloud_process_ranges;

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
