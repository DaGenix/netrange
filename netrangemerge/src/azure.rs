use crate::AzureOptions;
use anyhow::{bail, Error};
use libnetrangemerge::{merge_networks, IpNetwork, NetworkInterest};
use serde::Deserialize;
use std::fs::File;
use std::str::FromStr;
use std::cmp;
use cidr::{Inet, Cidr};

const SELECTED_KNOWN_AZURE_IP_RANGES: &[&'static str] = &[
    "2a01:110::/31",
    "2603:1000::/24",
    "2620:1EC::/36",

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
];

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct AzureRanges {
    changeNumber: u32,
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
    changeNumber: u32,
    region: String,
    regionId: u32,
    platform: String,
    systemService: String,
    addressPrefixes: Vec<String>,
    networkFeatures: Option<Vec<String>>,
}

fn convert_to_network_interests(
    azure_ip_ranges: AzureRanges,
    filter_function: &str,
) -> Result<Vec<NetworkInterest<IpNetwork>>, Error> {
    azure_ip_ranges.values.into_iter().flat_map(|range| {
        range.properties.addressPrefixes.into_iter().map(|prefix| {
            let c = cidr::IpInet::from_str(&prefix)?;
            let c = cidr::IpInet::new(c.first_address(), cmp::min(8, c.network_length()))?;
            let network = IpNetwork::new(c.network().first_address(), c.network_length())?;
            Ok(NetworkInterest::new(network, true))
        })
    }).collect()
}

pub fn azure_command(azure_options: AzureOptions) -> Result<(), Error> {
    let ranges: AzureRanges = if let Some(file) = azure_options.file {
        let file = File::open(file)?;
        serde_json::from_reader(file)?
    } else {
        let response = reqwest::blocking::get("https://www.microsoft.com/en-us/download/confirmation.aspx?id=56519")?;
        let body = response.text()?;
        let regex = regex::Regex::new(r"https://download.*?\.json")?;
        if let Some(m) = regex.find(&body) {
            let url = m.as_str();
            let response = reqwest::blocking::get(url)?;
            let body = response.text()?;
            serde_json::from_str(&body)?
        } else {
            bail!("Unable to find download URL for Azure IP range file");
        }
    };

    let mut all_networks = convert_to_network_interests(ranges, &azure_options.filter)?;

    // for known_range in SELECTED_KNOWN_AZURE_IP_RANGES {
    //     all_networks.push(NetworkInterest::new(
    //         IpNetwork::from_str(known_range).unwrap(),
    //         false,
    //     ));
    // }

    merge_networks(&mut all_networks);

    for network in all_networks {
        if network.is_interesting() {
            println!("{}", network.network());
        }
    }

    Ok(())
}
