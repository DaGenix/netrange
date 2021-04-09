use crate::utils::filter_select::RangesWithMetadata;
use anyhow::Error;
use libnetrangemerge::IpRange;
use serde::Deserialize;
use std::collections::HashMap;
use std::io;
use std::str::FromStr;

pub const FILTER_HELP: &'static str = r###"The Cloudflare service has the following filterable values:
 * is_ipv4
 * is_ipv6"###;

#[derive(Deserialize, Debug)]
struct CloudflareRanges {
    result: CloudflareRangesResult,
    success: bool,
}

#[derive(Deserialize, Debug)]
struct CloudflareRangesResult {
    ipv4_cidrs: Vec<String>,
    ipv6_cidrs: Vec<String>,
}

pub fn fetch_ranges() -> Result<reqwest::blocking::Response, Error> {
    Ok(reqwest::blocking::get("https://api.cloudflare.com/client/v4/ips")?.error_for_status()?)
}

pub fn load_ranges(reader: &mut dyn io::Read) -> Result<Vec<RangesWithMetadata>, Error> {
    let mut data = String::new();
    reader.read_to_string(&mut data)?;
    let cf_ranges: CloudflareRanges = serde_json::from_str(&data)?;
    let ip_ranges = cf_ranges
        .result
        .ipv4_cidrs
        .into_iter()
        .chain(cf_ranges.result.ipv6_cidrs.into_iter())
        .map(|range| Ok(IpRange::from_str(&range)?))
        .collect::<Result<Vec<IpRange>, Error>>()?;
    Ok(vec![RangesWithMetadata::new(HashMap::new(), ip_ranges)])
}
