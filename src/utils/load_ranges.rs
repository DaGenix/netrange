use crate::sources::{aws, azure, gcp};
use crate::utils::cloud_config::get_cloud_config;
use crate::utils::filter::NetworkWithMetadata;
use anyhow::{bail, Error};
use std::fs::File;
use std::io;
use std::path::PathBuf;

pub fn fetch_and_load_ranges(
    service: &str,
) -> Result<(Vec<NetworkWithMetadata>, &'static [&'static str]), Error> {
    let cc = get_cloud_config(service)?;
    let fetch_func = cc.fetch_ranges_func;
    let load_func = cc.load_ranges_func;
    let ranges = load_func(&mut fetch_func()?)?;
    Ok((ranges, cc.known_ranges))
}

pub fn load_ranges(
    service: &str,
    file: Option<&PathBuf>,
) -> Result<(Vec<NetworkWithMetadata>, &'static [&'static str]), Error> {
    let stdin = io::stdin();
    let cc = get_cloud_config(service)?;
    let load_func = cc.load_ranges_func;
    let ranges = if let Some(file) = file {
        load_func(&mut File::open(&file)?)?
    } else {
        load_func(&mut stdin.lock())?
    };
    Ok((ranges, cc.known_ranges))
}
