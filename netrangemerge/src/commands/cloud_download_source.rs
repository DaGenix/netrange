use crate::sources::{aws, azure, gcp};
use crate::CloudDownloadSourceOptions;
use anyhow::{bail, Error};
use std::fs::File;
use std::io;
use std::path::Path;

pub fn cloud_download_source_command(options: CloudDownloadSourceOptions) -> Result<(), Error> {
    let mut response = match options.service.as_str() {
        "aws" => aws::fetch_ranges()?,
        "azure" => azure::fetch_ranges()?,
        "gcp" => gcp::fetch_ranges()?,
        x => bail!("Invalid service: {}", x),
    };

    if options.file.as_path() == Path::new("-") {
        io::copy(&mut response, &mut io::stdout().lock())?;
    } else {
        io::copy(&mut response, &mut File::create(&options.file)?)?;
    }

    Ok(())
}
