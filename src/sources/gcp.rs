use crate::utils::filter_select::RangesWithMetadata;
use anyhow::{bail, Error};
use libnetrangemerge::IpRange;
use serde::Deserialize;
use std::collections::HashMap;
use std::io;
use std::str::FromStr;

pub const FILTER_HELP: &'static str = r###"
The GCP service has the following filterable values:
  * is_ipv4 (boolean) - True for IPV4 ranges, False for IPV6 ranges
  * is_ipv6 (boolean) - False for IPV4 ranges, True for IPV6 ranges
  * service (string) - The service, always "Google Cloud"
  * scope (string) - The scope, for example: "us-east1" or "europe-central2"

Documentation is available from: https://support.google.com/a/answer/10026322?hl=en
"###;

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
    Ok(
        reqwest::blocking::get("https://www.gstatic.com/ipranges/cloud.json")?
            .error_for_status()?,
    )
}

pub fn load_ranges(reader: &mut dyn io::Read) -> Result<Vec<RangesWithMetadata>, Error> {
    let mut data = String::new();
    reader.read_to_string(&mut data)?;
    let ranges: GcpRanges = serde_json::from_str(&data)?;
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
            Ok(RangesWithMetadata::new(metadata, ranges))
        })
        .collect()
}
