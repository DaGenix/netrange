use crate::sources::{aws, azure, cloudflare, gcp};
use crate::utils::filter::NetworkWithMetadata;
use anyhow::{bail, Error};
use std::io::Read;

pub struct CloudConfig {
    service_name: &'static str,
    pub fetch_ranges_func: fn() -> Result<reqwest::blocking::Response, Error>,
    pub load_ranges_func: fn(&mut dyn Read) -> Result<Vec<NetworkWithMetadata>, Error>,
    pub filter_help: &'static str,
    pub known_ranges: &'static [&'static str],
}

const CONFIG: &[CloudConfig] = &[
    CloudConfig {
        service_name: "aws",
        fetch_ranges_func: aws::fetch_ranges,
        load_ranges_func: aws::load_ranges,
        filter_help: aws::FILTER_HELP,
        known_ranges: aws::SELECTED_KNOWN_AMAZON_IP_RANGES,
    },
    CloudConfig {
        service_name: "azure",
        fetch_ranges_func: azure::fetch_ranges,
        load_ranges_func: azure::load_ranges,
        filter_help: azure::FILTER_HELP,
        known_ranges: azure::SELECTED_KNOWN_AZURE_IP_RANGES,
    },
    CloudConfig {
        service_name: "cloudflare",
        fetch_ranges_func: cloudflare::fetch_ranges,
        load_ranges_func: cloudflare::load_ranges,
        filter_help: cloudflare::FILTER_HELP,
        known_ranges: &[],
    },
    CloudConfig {
        service_name: "gcp",
        fetch_ranges_func: gcp::fetch_ranges,
        load_ranges_func: gcp::load_ranges,
        filter_help: gcp::FILTER_HELP,
        known_ranges: &[],
    },
];

pub fn get_cloud_names() -> Vec<&'static str> {
    CONFIG.iter().map(|cc| cc.service_name).collect()
}

pub fn get_cloud_config(service: &str) -> Result<&'static CloudConfig, Error> {
    let cc = CONFIG.iter().find(|cc| cc.service_name == service);
    if let Some(cc) = cc {
        Ok(cc)
    } else {
        bail!("Invalid service: {}", service)
    }
}
