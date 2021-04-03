use anyhow::Error;
use libnetrangemerge::{IpRange, Range as _, RangeInterest};
use std::collections::HashMap;

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
    networks: Vec<IpRange>,
}

impl NetworkWithMetadata {
    pub fn new(
        metadata: HashMap<&'static str, MetadataValue>,
        networks: Vec<IpRange>,
    ) -> NetworkWithMetadata {
        NetworkWithMetadata { metadata, networks }
    }
}

pub fn filter(
    ranges: Vec<NetworkWithMetadata>,
    filter_program: Option<&str>,
) -> Result<Vec<RangeInterest<IpRange>>, Error> {
    if filter_program.is_none() {
        let out = ranges
            .into_iter()
            .flat_map(|n| n.networks.into_iter().map(|n| RangeInterest::new(n, true)))
            .collect();
        return Ok(out);
    }
    let filter_program = filter_program.unwrap();

    let lua = rlua::Lua::new();
    lua.context(|ctx| -> Result<(), Error> {
        let func = ctx.load(filter_program).into_function()?;
        ctx.globals().set("func", func)?;
        Ok(())
    })?;

    let mut all_networks: Vec<RangeInterest<IpRange>> = Vec::new();
    for range in ranges {
        let metadata = range.metadata;
        lua.context(|ctx| -> Result<(), Error> {
            for (k, v) in metadata {
                match v {
                    MetadataValue::String(val) => ctx.globals().set(k, val)?,
                    MetadataValue::I64(val) => ctx.globals().set(k, val)?,
                }
            }
            Ok(())
        })?;
        for network in range.networks {
            let interesting = lua.context(|ctx| -> Result<bool, Error> {
                ctx.globals().set("is_ipv4", !network.is_ipv6())?;
                ctx.globals().set("is_ipv6", network.is_ipv6())?;
                let func: rlua::Function<'_> = ctx.globals().get("func")?;
                Ok(func.call(())?)
            })?;
            all_networks.push(RangeInterest::new(network, interesting))
        }
    }

    Ok(all_networks)
}
