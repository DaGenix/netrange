use crate::utils::filter_select::RangesWithMetadata;
use anyhow::Error;
use libnetrangemerge::IpRange;
use serde::Deserialize;
use std::collections::HashMap;
use std::io;
use std::str::FromStr;

pub const FILTER_HELP: &'static str = r###"
GitHub has the following filterable values:
  * is_ipv4 (boolean) - True for IPV4 ranges, False for IPV6 ranges
  * is_ipv6 (boolean) - False for IPV4 ranges, True for IPV6 ranges
  * sevice (string) - The service the range is used for, one of:
    - "hooks"
    - "web"
    - "api"
    - "git"
    - "pages"
    - "importer"
    - "actions"
    - "dependabot"

Documentation is available from:
https://docs.github.com/en/github/authenticating-to-github/about-githubs-ip-addresses
"###;

#[derive(Deserialize, Debug)]
struct GithubRanges {
    hooks: Vec<String>,
    web: Vec<String>,
    api: Vec<String>,
    git: Vec<String>,
    pages: Vec<String>,
    importer: Vec<String>,
    actions: Vec<String>,
    dependabot: Vec<String>,
}

pub fn fetch_ranges() -> Result<reqwest::blocking::Response, Error> {
    // reqwest doesn't seem to send a User-Agent header by default,
    // which causes GitHub to fail our request with a 403. GitHub doesn't
    // seem to care what User-Agent we use, however.
    let client = reqwest::blocking::ClientBuilder::new()
        .user_agent("netrange")
        .build()?;
    Ok(client
        .get("https://api.github.com/meta")
        .send()?
        .error_for_status()?)
}

pub fn load_ranges(reader: &mut dyn io::Read) -> Result<Vec<RangesWithMetadata>, Error> {
    let mut data = String::new();
    reader.read_to_string(&mut data)?;
    let github_ranges: GithubRanges = serde_json::from_str(&data)?;

    fn parse_ranges(
        service: &str,
        unparsed_ranges: Vec<String>,
    ) -> Result<RangesWithMetadata, Error> {
        let ranges = unparsed_ranges
            .into_iter()
            .map(|r| Ok(IpRange::from_str(&r)?))
            .collect::<Result<Vec<IpRange>, Error>>()?;
        let mut metadata = HashMap::new();
        metadata.insert("service", service.to_string().into());
        Ok(RangesWithMetadata::new(metadata, ranges))
    }

    let mut ranges_with_metadata = Vec::new();
    ranges_with_metadata.push(parse_ranges("hooks", github_ranges.hooks)?);
    ranges_with_metadata.push(parse_ranges("web", github_ranges.web)?);
    ranges_with_metadata.push(parse_ranges("api", github_ranges.api)?);
    ranges_with_metadata.push(parse_ranges("git", github_ranges.git)?);
    ranges_with_metadata.push(parse_ranges("pages", github_ranges.pages)?);
    ranges_with_metadata.push(parse_ranges("importer", github_ranges.importer)?);
    ranges_with_metadata.push(parse_ranges("actions", github_ranges.actions)?);
    ranges_with_metadata.push(parse_ranges("dependabot", github_ranges.dependabot)?);

    Ok(ranges_with_metadata)
}
