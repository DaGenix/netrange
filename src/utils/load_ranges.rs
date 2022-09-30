use crate::utils::cloud_config::{CloudName, get_cloud_config};
use crate::utils::filter_select::RangesWithMetadata;
use anyhow::Error;
use libnetrangemerge::{IpRange, RangeInterest};
use std::fs::File;
use std::io::{self, BufRead};
use std::path::PathBuf;

/// Download ranges from the internet for the named
/// service and then load them into a Vec<RangesWithMetadata>
/// suitable for filtering and selecting.
pub fn fetch_and_load_cloud_ranges(service: CloudName) -> Result<Vec<RangesWithMetadata>, Error> {
    let cc = get_cloud_config(service)?;
    let fetch_func = cc.fetch_ranges_func;
    let load_func = cc.load_ranges_func;
    let ranges = load_func(&mut fetch_func()?)?;
    Ok(ranges)
}

/// Load ranges from a file for the named service into a Vec<RangesWithMetadata>
/// suitable for filtering and selecting.
pub fn load_cloud_ranges(service: CloudName, file: PathBuf) -> Result<Vec<RangesWithMetadata>, Error> {
    let stdin = io::stdin();
    let cc = get_cloud_config(service)?;
    let load_func = cc.load_ranges_func;
    let ranges = if let Some("-") = file.as_path().to_str() {
        load_func(&mut stdin.lock())?
    } else {
        load_func(&mut File::open(&file)?)?
    };
    Ok(ranges)
}

/// Load a list of ranges from the `reader`, expecting a single
/// CIDR range per-line. Loaded ranged are pushed on to the end of
/// `ranges`.
pub fn read_single_line_ranges(
    reader: &mut dyn io::Read,
    ranges: &mut Vec<RangeInterest<IpRange>>,
    interesting: bool,
) -> Result<(), Error> {
    let bufreader = io::BufReader::new(reader);
    for line in bufreader.lines() {
        let line = line?;
        let range: IpRange = line.parse()?;
        ranges.push(RangeInterest::new(range, interesting));
    }
    Ok(())
}
