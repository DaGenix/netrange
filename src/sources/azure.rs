use crate::utils::filter_select::RangesWithMetadata;
use anyhow::{bail, Error};
use libnetrangemerge::IpRange;
use serde::Deserialize;
use std::collections::HashMap;
use std::io;
use std::str::FromStr;

pub const FILTER_HELP: &'static str = r###"The Azure service has the following filterable values:
 * is_ipv4
 * is_ipv6
 * name
 * id
 * region
 * regionId
 * platform
 * systemService"###;

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
    let client = reqwest::blocking::Client::new();
    let response = client
        .get("https://www.microsoft.com/en-us/download/confirmation.aspx?id=56519")
        .send()?
        .error_for_status()?;
    let body = response.text()?;
    let regex = regex::Regex::new(r"https://download.*?\.json")?;
    if let Some(m) = regex.find(&body) {
        let url = m.as_str();
        let response = client.get(url).send()?.error_for_status()?;
        Ok(response)
    } else {
        bail!("Unable to find download URL for Azure IP range file");
    }
}

#[allow(non_snake_case)]
pub fn load_ranges(reader: &mut dyn io::Read) -> Result<Vec<RangesWithMetadata>, Error> {
    let mut data = String::new();
    reader.read_to_string(&mut data)?;
    let ranges: AzureRanges = serde_json::from_str(&data)?;
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
                    let network = IpRange::from_str(&prefix)?;
                    Ok(network)
                })
                .collect::<Result<Vec<IpRange>, Error>>()?;

            Ok(RangesWithMetadata::new(metadata, ranges))
        })
        .collect()
}