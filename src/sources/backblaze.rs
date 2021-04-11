use crate::utils::filter_select::RangesWithMetadata;
use anyhow::Error;
use regex::Regex;
use std::collections::HashMap;
use std::io;

pub const FILTER_HELP: &'static str = r###"
The Backblaze service has the following filterable values:
  * is_ipv4 (boolean) - True for IPV4 ranges, False for IPV6 ranges
  * is_ipv6 (boolean) - False for IPV4 ranges, True for IPV6 ranges

Backblaze doesn't publish a machine readable list of IP ranges - instead
we have to scrape them using a regex from:
https://help.backblaze.com/hc/en-us/articles/217664588-Backblaze-IP-Address-List
"###;

pub fn fetch_ranges() -> Result<reqwest::blocking::Response, Error> {
    let response = reqwest::blocking::get(
        "https://help.backblaze.com/hc/en-us/articles/217664588-Backblaze-IP-Address-List",
    )?
    .error_for_status()?;
    Ok(response)
}

pub fn load_ranges(reader: &mut dyn io::Read) -> Result<Vec<RangesWithMetadata>, Error> {
    let mut data = String::new();
    reader.read_to_string(&mut data)?;
    let regex = Regex::new(
        r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}/\d{1,2}\b|\b(?:[0-9a-f]{4}:)+:?/\d{1,3}\b",
    )?;
    let ranges = regex
        .find_iter(&data)
        .map(|m| -> Result<RangesWithMetadata, Error> {
            let range = m.as_str().parse()?;
            Ok(RangesWithMetadata::new(HashMap::new(), vec![range]))
        })
        .collect::<Result<Vec<RangesWithMetadata>, Error>>()?;
    Ok(ranges)
}
