use crate::sources::{aws, azure, gcp};
use crate::CloudGetOptions;
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
