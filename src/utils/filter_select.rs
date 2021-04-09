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

pub struct RangesWithMetadata {
    metadata: HashMap<&'static str, MetadataValue>,
    ranges: Vec<IpRange>,
}

impl RangesWithMetadata {
    pub fn new(
        metadata: HashMap<&'static str, MetadataValue>,
        ranges: Vec<IpRange>,
    ) -> RangesWithMetadata {
        RangesWithMetadata { metadata, ranges }
    }
}

pub fn filter_select(
    range_group: Vec<RangesWithMetadata>,
    filter_program: Option<&str>,
    select_program: Option<&str>,
) -> Result<Vec<RangeInterest<IpRange>>, Error> {
    if filter_program.is_none() && select_program.is_none() {
        let out = range_group
            .into_iter()
            .flat_map(|n| n.ranges.into_iter().map(|n| RangeInterest::new(n, true)))
            .collect();
        return Ok(out);
    }

    let lua = rlua::Lua::new();
    lua.context(|ctx| -> Result<(), Error> {
        if let Some(filter_program) = filter_program {
            let func = ctx.load(filter_program).into_function()?;
            ctx.globals().set("__filter_func", func)?;
        }
        if let Some(select_program) = select_program {
            let func = ctx.load(select_program).into_function()?;
            ctx.globals().set("__select_func", func)?;
        }
        Ok(())
    })?;

    let mut output_ranges: Vec<RangeInterest<IpRange>> = Vec::new();
    for ranges_with_metadata in range_group {
        let metadata = ranges_with_metadata.metadata;
        lua.context(|ctx| -> Result<(), Error> {
            for (k, v) in metadata {
                match v {
                    MetadataValue::String(val) => ctx.globals().set(k, val)?,
                    MetadataValue::I64(val) => ctx.globals().set(k, val)?,
                }
            }
            Ok(())
        })?;

        for range in ranges_with_metadata.ranges {
            let selected = lua.context(|ctx| -> Result<Option<bool>, Error> {
                ctx.globals().set("is_ipv4", !range.is_ipv6())?;
                ctx.globals().set("is_ipv6", range.is_ipv6())?;

                let filter_func: rlua::Function<'_> = ctx.globals().get("__filter_func")?;
                if !filter_func.call(())? {
                    return Ok(None);
                }

                let select_func: rlua::Function<'_> = ctx.globals().get("__select_func")?;
                let selected = select_func.call(())?;
                Ok(Some(selected))
            })?;
            if let Some(selected) = selected {
                output_ranges.push(RangeInterest::new(range, selected))
            }
        }
    }

    Ok(output_ranges)
}
