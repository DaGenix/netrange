use crate::utils::filter_select::RangesWithMetadata;
use anyhow::Error;
use libnetrangemerge::IpRange;
use serde::Deserialize;
use std::collections::HashMap;
use std::io;
use std::str::FromStr;

pub const FILTER_HELP: &'static str = r###"
The AWS service has the following filterable values:
  * is_ipv4 (boolean) - True for IPV4 ranges, False for IPV6 ranges
  * is_ipv6 (boolean) - False for IPV4 ranges, True for IPV6 ranges
  * region (string) - The region, for example: "us-east-1", "us-gov-west-1", "GLOBAL", etc
  * service (string) - The service, for example: "EC2", "AMAZON", "S3", "API_GATEWAY", etc
  * network_border_group (string) - The network border group

See https://docs.aws.amazon.com/general/latest/gr/aws-ip-ranges.html for more information
"###;

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
    Ok(
        reqwest::blocking::get("https://ip-ranges.amazonaws.com/ip-ranges.json")?
            .error_for_status()?,
    )
}

pub fn load_ranges(reader: &mut dyn io::Read) -> Result<Vec<RangesWithMetadata>, Error> {
    let mut data = String::new();
    reader.read_to_string(&mut data)?;
    let ranges: AwsRanges = serde_json::from_str(&data)?;
    let ipv4_ranges = ranges.prefixes.into_iter().map(|range| {
        let mut metadata = HashMap::new();
        metadata.insert("region", range.region.into());
        metadata.insert("service", range.service.into());
        metadata.insert("network_border_group", range.network_border_group.into());

        let ranges = vec![IpRange::from_str(&range.ip_prefix)?];

        Ok(RangesWithMetadata::new(metadata, ranges))
    });
    let ipv6_ranges = ranges.ipv6_prefixes.into_iter().map(|range| {
        let mut metadata = HashMap::new();
        metadata.insert("region", range.region.into());
        metadata.insert("service", range.service.into());
        metadata.insert("network_border_group", range.network_border_group.into());

        let ranges = vec![IpRange::from_str(&range.ipv6_prefix)?];

        Ok(RangesWithMetadata::new(metadata, ranges))
    });
    ipv4_ranges.chain(ipv6_ranges).collect()
}
