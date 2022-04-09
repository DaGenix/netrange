use crate::utils::filter_select::RangesWithMetadata;
use anyhow::Error;
use libnetrangemerge::IpRange;
use serde::Deserialize;
use std::collections::HashMap;
use std::io;
use std::str::FromStr;

pub const FILTER_HELP: &'static str = r###"
The Fastly service has the following filterable values:
  * is_ipv4 (boolean) - True for IPV4 ranges, False for IPV6 ranges
  * is_ipv6 (boolean) - False for IPV4 ranges, True for IPV6 ranges

Fastly IP ranges are published at: https://api.fastly.com/public-ip-list
"###;

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct FastlyRanges {
    addresses: Vec<String>,
    ipv6_addresses: Vec<String>,
}

pub fn fetch_ranges() -> Result<reqwest::blocking::Response, Error> {
    Ok(reqwest::blocking::get("https://api.fastly.com/public-ip-list")?.error_for_status()?)
}

pub fn load_ranges(reader: &mut dyn io::Read) -> Result<Vec<RangesWithMetadata>, Error> {
    let mut data = String::new();
    reader.read_to_string(&mut data)?;
    let ranges: FastlyRanges = serde_json::from_str(&data)?;
    let ip_ranges = ranges
        .addresses
        .into_iter()
        .chain(ranges.ipv6_addresses.into_iter())
        .map(|range| Ok(IpRange::from_str(&range)?))
        .collect::<Result<Vec<IpRange>, Error>>()?;
    Ok(vec![RangesWithMetadata::new(HashMap::new(), ip_ranges)])
}
