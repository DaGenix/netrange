use crate::commands::get_ranges::NetworkWithMetadata;
use anyhow::{bail, Error};
use libnetrangemerge::IpNetwork;
use serde::Deserialize;
use std::collections::HashMap;
use std::io;
use std::str::FromStr;

pub const SELECTED_KNOWN_AZURE_IP_RANGES: &[&'static str] = &[
    "13.64.0.0/11",
    "13.96.0.0/13",
    "13.104.0.0/14",
    "23.96.0.0/13",
    "70.37.0.0/17",
    "70.37.128.0/18",
    "157.54.0.0/15",
    "157.56.0.0/14",
    "157.60.0.0/16",
    "20.33.0.0/16",
    "20.34.0.0/15",
    "20.36.0.0/14",
    "20.40.0.0/13",
    "20.48.0.0/12",
    "20.64.0.0/10",
    "20.128.0.0/16",
    "40.64.0.0/13",
    "20.150.0.0/15",
    "20.152.0.0/15",
    "20.192.0.0/10",
    "40.74.0.0/15",
    "40.76.0.0/14",
    "40.80.0.0/12",
    "40.96.0.0/12",
    "40.112.0.0/13",
    "40.120.0.0/14",
    "40.124.0.0/16",
    "40.125.0.0/17",
    "131.253.12.0/22",
    "131.253.16.0/23",
    "131.253.18.0/24",
    "65.52.0.0/14",
    "102.133.0.0/16",
    "131.253.21.0/24",
    "131.253.22.0/23",
    "131.253.24.0/21",
    "131.253.32.0/20",
    "2a01:110::/31",
    "2603:1000::/24",
    "2620:1EC::/36",
];

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct AzureRanges {
    changeNumber: i64,
    cloud: String,
    values: Vec<AzureRange>,
}

#[derive(Deserialize, Debug)]
struct AzureRange {
    name: String,
    id: String,
    properties: AzureRangeProperties,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct AzureRangeProperties {
    changeNumber: i64,
    region: String,
    regionId: i64,
    platform: String,
    systemService: String,
    addressPrefixes: Vec<String>,
    networkFeatures: Option<Vec<String>>,
}

pub fn fetch_ranges() -> Result<reqwest::blocking::Response, Error> {
    let response = reqwest::blocking::get(
        "https://www.microsoft.com/en-us/download/confirmation.aspx?id=56519",
    )?;
    let body = response.text()?;
    let regex = regex::Regex::new(r"https://download.*?\.json")?;
    if let Some(m) = regex.find(&body) {
        let url = m.as_str();
        let response = reqwest::blocking::get(url)?;
        Ok(response)
    } else {
        bail!("Unable to find download URL for Azure IP range file");
    }
}

#[allow(non_snake_case)]
pub fn load_ranges<R: io::Read>(reader: R) -> Result<Vec<NetworkWithMetadata>, Error> {
    let ranges: AzureRanges = serde_json::from_reader(reader)?;
    ranges
        .values
        .into_iter()
        .map(|range| {
            let AzureRange {
                name,
                id,
                properties,
            } = range;
            let AzureRangeProperties {
                region,
                regionId,
                platform,
                systemService,
                addressPrefixes,
                ..
            } = properties;

            let mut metadata = HashMap::new();

            metadata.insert("name", name.into());
            metadata.insert("id", id.into());

            metadata.insert("region", region.into());
            metadata.insert("regionId", regionId.into());
            metadata.insert("platform", platform.into());
            metadata.insert("systemService", systemService.into());

            let ranges = addressPrefixes
                .into_iter()
                .map(|prefix| {
                    let network = IpNetwork::from_str(&prefix)?;
                    Ok(network)
                })
                .collect::<Result<Vec<IpNetwork>, Error>>()?;

            Ok(NetworkWithMetadata::new(metadata, ranges))
        })
        .collect()
}
