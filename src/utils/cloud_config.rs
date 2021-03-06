use crate::sources::{aws, azure, cloudflare, digitalocean, fastly, gcp, github, google};
use crate::utils::filter_select::RangesWithMetadata;
use anyhow::{bail, Error};
use once_cell::sync::Lazy;
use std::io::Read;
use std::ops::Deref;

pub struct CloudConfig {
    service_name: &'static str,
    pub fetch_ranges_func: fn() -> Result<reqwest::blocking::Response, Error>,
    pub load_ranges_func: fn(&mut dyn Read) -> Result<Vec<RangesWithMetadata>, Error>,
    pub filter_help: &'static str,
}

const CONFIG: &[CloudConfig] = &[
    CloudConfig {
        service_name: "aws",
        fetch_ranges_func: aws::fetch_ranges,
        load_ranges_func: aws::load_ranges,
        filter_help: aws::FILTER_HELP,
    },
    CloudConfig {
        service_name: "azure",
        fetch_ranges_func: azure::fetch_ranges,
        load_ranges_func: azure::load_ranges,
        filter_help: azure::FILTER_HELP,
    },
    CloudConfig {
        service_name: "cloudflare",
        fetch_ranges_func: cloudflare::fetch_ranges,
        load_ranges_func: cloudflare::load_ranges,
        filter_help: cloudflare::FILTER_HELP,
    },
    CloudConfig {
        service_name: "digitalocean",
        fetch_ranges_func: digitalocean::fetch_ranges,
        load_ranges_func: digitalocean::load_ranges,
        filter_help: digitalocean::FILTER_HELP,
    },
    CloudConfig {
        service_name: "fastly",
        fetch_ranges_func: fastly::fetch_ranges,
        load_ranges_func: fastly::load_ranges,
        filter_help: fastly::FILTER_HELP,
    },
    CloudConfig {
        service_name: "gcp",
        fetch_ranges_func: gcp::fetch_ranges,
        load_ranges_func: gcp::load_ranges,
        filter_help: gcp::FILTER_HELP,
    },
    CloudConfig {
        service_name: "github",
        fetch_ranges_func: github::fetch_ranges,
        load_ranges_func: github::load_ranges,
        filter_help: github::FILTER_HELP,
    },
    CloudConfig {
        service_name: "google",
        fetch_ranges_func: google::fetch_ranges,
        load_ranges_func: google::load_ranges,
        filter_help: google::FILTER_HELP,
    },
];

pub fn get_cloud_names() -> &'static Vec<&'static str> {
    static INSTANCE: Lazy<Vec<&'static str>> =
        Lazy::new(|| CONFIG.iter().map(|cc| cc.service_name).collect());
    INSTANCE.deref()
}

pub fn get_cloud_config(service: &str) -> Result<&'static CloudConfig, Error> {
    let cc = CONFIG.iter().find(|cc| cc.service_name == service);
    if let Some(cc) = cc {
        Ok(cc)
    } else {
        bail!("Invalid service: {}", service)
    }
}
