use crate::utils::filter_select::RangesWithMetadata;
use anyhow::{bail, Error};
use csv;
use libnetrangemerge::IpRange;
use std::collections::HashMap;
use std::io;
use std::str::FromStr;

pub const FILTER_HELP: &'static str = r###"
The DigitalOcean service has the following filterable values:
  * is_ipv4 (boolean) - True for IPV4 ranges, False for IPV6 ranges
  * is_ipv6 (boolean) - False for IPV4 ranges, True for IPV6 ranges
  * country_code (string) - 2 letter country code, for example: "US" or "NL"
  * location_code (string) - location code, for example: "US-NJ", "US-SF", or "NL-NH". As witnessed
      by the "US-SF" example, the part after the dash may represent a state or city.
  * city (string) - The name of the city, for example: "San Francisco" or "Amsterdam"
  * postal_code (string) - The postcal code, which varies depenending on the country, for example
      in the US San Francisco is "94124", while Amsterdam is "1105 AT"

NOTE: Most records have all fields available. However, some records are missing every
      field except for the CIDR range. In such cases, the missing attributes will be
      set to the empty string. Take this into account when creating filters!

The source file is linked from: https://docs.digitalocean.com/products/platform/ -
but there appears to be little or no documentation about it. The above field names are a
best guess.
"###;

pub fn fetch_ranges() -> Result<reqwest::blocking::Response, Error> {
    Ok(reqwest::blocking::get("https://digitalocean.com/geo/google.csv")?.error_for_status()?)
}

pub fn load_ranges(reader: &mut dyn io::Read) -> Result<Vec<RangesWithMetadata>, Error> {
    let mut csv_reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(reader);
    csv_reader
        .records()
        .map(|record| {
            let record = record?;

            let cidr_range = if let Some(cidr_range) = record.get(0) {
                IpRange::from_str(cidr_range)?
            } else {
                bail!("Missing CIDR range in CSV is not allowed")
            };

            let mut metadata = HashMap::new();
            metadata.insert(
                "country_code",
                record
                    .get(1)
                    .map(str::to_string)
                    .unwrap_or(String::new())
                    .into(),
            );
            metadata.insert(
                "location_code",
                record
                    .get(2)
                    .map(str::to_string)
                    .unwrap_or(String::new())
                    .into(),
            );
            metadata.insert(
                "city",
                record
                    .get(3)
                    .map(str::to_string)
                    .unwrap_or(String::new())
                    .into(),
            );
            metadata.insert(
                "postal_code",
                record
                    .get(4)
                    .map(str::to_string)
                    .unwrap_or(String::new())
                    .into(),
            );

            Ok(RangesWithMetadata::new(metadata, vec![cidr_range]))
        })
        .collect()
}
