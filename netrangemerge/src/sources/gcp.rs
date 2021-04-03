use crate::utils::filter::NetworkWithMetadata;
use anyhow::{bail, Error};
use libnetrangemerge::IpRange;
use serde::Deserialize;
use std::collections::HashMap;
use std::io;
use std::str::FromStr;

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
#[allow(dead_code)]
struct GcpRanges {
    syncToken: String,
    creationTime: String,
    prefixes: Vec<GcpRange>,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
#[allow(dead_code)]
struct GcpRange {
    ipv4Prefix: Option<String>,
    ipv6Prefix: Option<String>,
    service: String,
    scope: String,
}

pub fn fetch_ranges() -> Result<reqwest::blocking::Response, Error> {
    Ok(reqwest::blocking::get(
        "https://www.gstatic.com/ipranges/cloud.json",
    )?)
}

pub fn load_ranges<R: io::Read>(reader: R) -> Result<Vec<NetworkWithMetadata>, Error> {
    let ranges: GcpRanges = serde_json::from_reader(io::BufReader::new(reader))?;
    ranges
        .prefixes
        .into_iter()
        .map(|range| {
            let mut metadata = HashMap::new();
            metadata.insert("service", range.service.into());
            metadata.insert("scope", range.scope.into());

            let ranges = if let Some(r) = range.ipv4Prefix {
                vec![IpRange::from_str(&r)?]
            } else if let Some(r) = range.ipv6Prefix {
                vec![IpRange::from_str(&r)?]
            } else {
                bail!("No ipv4 or ipv6 prefix found in element.")
            };
            Ok(NetworkWithMetadata::new(metadata, ranges))
        })
        .collect()
}
