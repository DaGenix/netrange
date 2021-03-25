use crate::providers::{aws, azure, gcp};
use crate::DownloadSources;
use anyhow::{bail, Error};
use std::fs::File;
use std::io;

pub fn download_sources(options: DownloadSources) -> Result<(), Error> {
    let mut response = match options.service.as_str() {
        "aws" => aws::fetch_ranges()?,
        "azure" => azure::fetch_ranges()?,
        "gcp" => gcp::fetch_ranges()?,
        x => bail!("Invalid service: {}", x),
    };

    if options.file == "-" {
        io::copy(&mut response, &mut io::stdout().lock())?;
    } else {
        io::copy(&mut response, &mut File::create(&options.file)?)?;
    }

    Ok(())
}
