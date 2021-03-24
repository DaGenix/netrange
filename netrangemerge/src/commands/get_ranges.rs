use crate::providers::aws::SELECTED_KNOWN_AMAZON_IP_RANGES;
use crate::providers::azure::SELECTED_KNOWN_AZURE_IP_RANGES;
use crate::providers::{aws, azure, gcp};
use crate::GetRangesOptions;
use anyhow::{bail, Error};
use libnetrangemerge::{merge_networks, IpNetwork, Network, NetworkInterest};
use std::collections::HashMap;
use std::fs::File;
use std::str::FromStr;

pub enum MetadataValue {
    String(String),
    I64(i64),
}

impl From<String> for MetadataValue {
    fn from(val: String) -> MetadataValue {
        MetadataValue::String(val)
    }
}

impl From<i64> for MetadataValue {
    fn from(val: i64) -> MetadataValue {
        MetadataValue::I64(val)
    }
}

pub struct NetworkWithMetadata {
    metadata: HashMap<&'static str, MetadataValue>,
    networks: Vec<IpNetwork>,
}

impl NetworkWithMetadata {
    pub fn new(
        metadata: HashMap<&'static str, MetadataValue>,
        networks: Vec<IpNetwork>,
    ) -> NetworkWithMetadata {
        NetworkWithMetadata { metadata, networks }
    }
}

pub fn get_ranges_command(options: GetRangesOptions) -> Result<(), Error> {
    let (ranges, known_ranges) = match options.service.as_str() {
        "aws" => {
            let ranges = if let Some(path) = options.file {
                let f = File::open(&path)?;
                aws::load_ranges(f)
            } else {
                let r = aws::fetch_ranges()?;
                aws::load_ranges(r)
            }?;
            (ranges, SELECTED_KNOWN_AMAZON_IP_RANGES)
        }
        "azure" => {
            let ranges = if let Some(path) = options.file {
                let f = File::open(&path)?;
                azure::load_ranges(f)
            } else {
                let r = azure::fetch_ranges()?;
                azure::load_ranges(r)
            }?;
            (ranges, SELECTED_KNOWN_AZURE_IP_RANGES)
        }
        "gcp" => {
            let ranges = if let Some(path) = options.file {
                let f = File::open(&path)?;
                gcp::load_ranges(f)
            } else {
                let r = gcp::fetch_ranges()?;
                gcp::load_ranges(r)
            }?;
            let tmp: &'static [&str] = &[];
            (ranges, tmp)
        }
        x => bail!("Invalid service: {}", x),
    };

    let lua = if let Some(filter_function) = options.filter.as_ref() {
        let lua = rlua::Lua::new();
        lua.context(|ctx| -> Result<(), Error> {
            let func = ctx.load(filter_function).into_function()?;
            ctx.globals().set("func", func)?;
            Ok(())
        })?;
        Some(lua)
    } else {
        None
    };

    let mut all_networks: Vec<NetworkInterest<IpNetwork>> = Vec::new();
    for range in ranges {
        let metadata = range.metadata;
        if let Some(lua) = lua.as_ref() {
            lua.context(|ctx| -> Result<(), Error> {
                for (k, v) in metadata {
                    match v {
                        MetadataValue::String(val) => ctx.globals().set(k, val)?,
                        MetadataValue::I64(val) => ctx.globals().set(k, val)?,
                    }
                }
                Ok(())
            })?;
        };
        for network in range.networks {
            let interesting = if let Some(lua) = lua.as_ref() {
                lua.context(|ctx| -> Result<bool, Error> {
                    ctx.globals().set("is_ipv6", network.is_ipv6())?;
                    let func: rlua::Function<'_> = ctx.globals().get("func")?;
                    Ok(func.call(())?)
                })?
            } else {
                true
            };

            all_networks.push(NetworkInterest::new(network, interesting))
        }
    }

    if !options.ignore_known_ranges {
        for known_range in known_ranges {
            all_networks.push(NetworkInterest::new(
                IpNetwork::from_str(known_range).unwrap(),
                false,
            ));
        }
    }

    merge_networks(&mut all_networks);

    for network in all_networks {
        if network.is_interesting() {
            println!("{}", network.network());
        }
    }

    Ok(())
}
