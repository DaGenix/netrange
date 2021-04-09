use crate::utils::expand_ranges::expand_ranges;
use crate::utils::filter::{filter, NetworkWithMetadata};
use anyhow::Error;
use libnetrangemerge::merge_ranges;
use std::fs::File;
use std::io;
use std::io::{BufWriter, Read, Write};
use std::path::PathBuf;

pub fn cloud_process_ranges(
    ranges: Vec<NetworkWithMetadata>,
    filter_program: Option<String>,
    filter_file: Option<PathBuf>,
    min_ipv4_network_size: Option<u8>,
    min_ipv6_network_size: Option<u8>,
    do_merge: bool,
) -> Result<(), Error> {
    let mut filtered_ranges = if let Some(filter_program) = filter_program {
        filter(ranges, Some(&filter_program))?
    } else if let Some(filter_file) = filter_file {
        let mut filter_program = String::new();
        File::open(&filter_file)?.read_to_string(&mut filter_program)?;
        filter(ranges, Some(&filter_program))?
    } else {
        filter(ranges, None)?
    };

    expand_ranges(
        filtered_ranges.iter_mut().map(|n| n.range_mut()),
        min_ipv4_network_size,
        min_ipv6_network_size,
    );

    if do_merge {
        merge_ranges(&mut filtered_ranges);
    }

    let stdout = io::stdout();
    let mut stdout = BufWriter::new(stdout.lock());
    for network in filtered_ranges {
        if network.is_interesting() {
            writeln!(stdout, "{}", network.range())?;
        }
    }
    stdout.flush()?;

    Ok(())
}
