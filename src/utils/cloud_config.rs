use crate::sources::{aws, azure, cloudflare, digitalocean, fastly, gcp, github, google};
use crate::utils::filter_select::RangesWithMetadata;
use anyhow::{bail, Error};
use clap::ValueEnum;
use std::io::Read;

#[derive(ValueEnum, Clone, Copy, Eq, PartialEq, Debug)]
pub enum CloudName {
    Aws,
    Azure,
    Cloudflare,
    Digitalocean,
    Fastly,
    Gcp,
    Github,
    Google,
}

pub struct CloudConfig {
    service_name: CloudName,
    pub fetch_ranges_func: fn() -> Result<reqwest::blocking::Response, Error>,
    pub load_ranges_func: fn(&mut dyn Read) -> Result<Vec<RangesWithMetadata>, Error>,
    pub filter_help: &'static str,
}

const CONFIG: &[CloudConfig] = &[
    CloudConfig {
        service_name: CloudName::Aws,
        fetch_ranges_func: aws::fetch_ranges,
        load_ranges_func: aws::load_ranges,
        filter_help: aws::FILTER_HELP,
    },
    CloudConfig {
        service_name: CloudName::Azure,
        fetch_ranges_func: azure::fetch_ranges,
        load_ranges_func: azure::load_ranges,
        filter_help: azure::FILTER_HELP,
    },
    CloudConfig {
        service_name: CloudName::Cloudflare,
        fetch_ranges_func: cloudflare::fetch_ranges,
        load_ranges_func: cloudflare::load_ranges,
        filter_help: cloudflare::FILTER_HELP,
    },
    CloudConfig {
        service_name: CloudName::Digitalocean,
        fetch_ranges_func: digitalocean::fetch_ranges,
        load_ranges_func: digitalocean::load_ranges,
        filter_help: digitalocean::FILTER_HELP,
    },
    CloudConfig {
        service_name: CloudName::Fastly,
        fetch_ranges_func: fastly::fetch_ranges,
        load_ranges_func: fastly::load_ranges,
        filter_help: fastly::FILTER_HELP,
    },
    CloudConfig {
        service_name: CloudName::Gcp,
        fetch_ranges_func: gcp::fetch_ranges,
        load_ranges_func: gcp::load_ranges,
        filter_help: gcp::FILTER_HELP,
    },
    CloudConfig {
        service_name: CloudName::Github,
        fetch_ranges_func: github::fetch_ranges,
        load_ranges_func: github::load_ranges,
        filter_help: github::FILTER_HELP,
    },
    CloudConfig {
        service_name: CloudName::Google,
        fetch_ranges_func: google::fetch_ranges,
        load_ranges_func: google::load_ranges,
        filter_help: google::FILTER_HELP,
    },
];

pub fn get_cloud_config(service: CloudName) -> Result<&'static CloudConfig, Error> {
    let cc = CONFIG.iter().find(|cc| cc.service_name == service);
    if let Some(cc) = cc {
        Ok(cc)
    } else {
        bail!("Invalid service: {:?}", service)
    }
}
