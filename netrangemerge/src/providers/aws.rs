use crate::commands::get_ranges::NetworkWithMetadata;
use anyhow::Error;
use libnetrangemerge::IpRange;
use serde::Deserialize;
use std::collections::HashMap;
use std::io;
use std::str::FromStr;

pub const SELECTED_KNOWN_AMAZON_IP_RANGES: &[&'static str] = &[
    "15.177.0.0/16",
    "15.190.0.0/15",
    "15.192.0.0/15",
    "18.128.0.0/9",
    "18.32.0.0/11",
    "18.64.0.0/10",
    "3.0.0.0/8",
    "35.152.0.0/13",
    "35.160.0.0/12",
    "35.176.0.0/13",
    "52.0.0.0/10",
    "52.64.0.0/12",
    "52.84.0.0/14",
    "52.88.0.0/13",
    "54.144.0.0/12",
    "54.160.0.0/11",
    "54.192.0.0/12",
    "54.208.0.0/13",
    "54.216.0.0/14",
    "54.220.0.0/15",
    "54.224.0.0/11",
    "99.150.0.0/17",
    "99.77.128.0/17",
    "99.78.0.0/18",
    "2400:6500::/32",
    "2400:6700::/32",
    "2400:7fc0::/32",
    "2403:b300::/32",
    "2404:c2c0::/32",
    "2406:da00::/24",
    "240f:8000::/24",
    "2600:1F00::/24",
    "2600:9000::/28",
    "2606:F40::/32",
    "2620:107:3000::/44",
    "2620:107:4000::/44",
    "2804:800::/32",
    "2a01:578::/32",
    "2a05:d000::/25",
];

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct AwsRanges {
    #[allow(dead_code)]
    syncToken: String,
    #[allow(dead_code)]
    createDate: String,
    prefixes: Vec<AwsIpv4Range>,
    ipv6_prefixes: Vec<AwsIpv6Range>,
}

#[derive(Deserialize, Debug)]
struct AwsIpv4Range {
    ip_prefix: String,
    region: String,
    service: String,
    network_border_group: String,
}

#[derive(Deserialize, Debug)]
struct AwsIpv6Range {
    ipv6_prefix: String,
    region: String,
    service: String,
    network_border_group: String,
}

pub fn fetch_ranges() -> Result<reqwest::blocking::Response, Error> {
    Ok(reqwest::blocking::get(
        "https://ip-ranges.amazonaws.com/ip-ranges.json",
    )?)
}

pub fn load_ranges<R: io::Read>(reader: R) -> Result<Vec<NetworkWithMetadata>, Error> {
    let ranges: AwsRanges = serde_json::from_reader(reader)?;
    let ipv4_ranges = ranges.prefixes.into_iter().map(|range| {
        let mut metadata = HashMap::new();
        metadata.insert("region", range.region.into());
        metadata.insert("service", range.service.into());
        metadata.insert("network_border_group", range.network_border_group.into());

        let ranges = vec![IpRange::from_str(&range.ip_prefix)?];

        Ok(NetworkWithMetadata::new(metadata, ranges))
    });
    let ipv6_ranges = ranges.ipv6_prefixes.into_iter().map(|range| {
        let mut metadata = HashMap::new();
        metadata.insert("region", range.region.into());
        metadata.insert("service", range.service.into());
        metadata.insert("network_border_group", range.network_border_group.into());

        let ranges = vec![IpRange::from_str(&range.ipv6_prefix)?];

        Ok(NetworkWithMetadata::new(metadata, ranges))
    });
    ipv4_ranges.chain(ipv6_ranges).collect()
}
