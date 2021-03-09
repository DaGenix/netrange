use crate::utils::expand_ranges::expand_ranges;
use crate::utils::filter_select::{filter_select, RangesWithMetadata};
use crate::utils::load_ranges::read_single_line_ranges;
use anyhow::Error;
use libnetrangemerge::merge_ranges;
use std::fs::File;
use std::io;
use std::io::{BufWriter, Read, Write};
use std::path::PathBuf;

fn get_program(
    program: Option<String>,
    program_file: Option<PathBuf>,
) -> Result<Option<String>, Error> {
    if let Some(program) = program {
        Ok(Some(program))
    } else if let Some(program_file) = program_file {
        let mut program = String::new();
        File::open(&program_file)?.read_to_string(&mut program)?;
        Ok(Some(program))
    } else {
        Ok(None)
    }
}

pub fn cloud_process_ranges(
    ranges: Vec<RangesWithMetadata>,
    filter_program: Option<String>,
    filter_file: Option<PathBuf>,
    select_program: Option<String>,
    select_file: Option<PathBuf>,
    extra_ranges_files: Vec<PathBuf>,
    min_ipv4_network_size: Option<u8>,
    min_ipv6_network_size: Option<u8>,
    do_merge: bool,
) -> Result<(), Error> {
    let filter_program = get_program(filter_program, filter_file)?;
    let select_program = get_program(select_program, select_file)?;
    let mut filtered_ranges =
        filter_select(ranges, filter_program.as_deref(), select_program.as_deref())?;

    expand_ranges(
        filtered_ranges.iter_mut().map(|n| n.range_mut()),
        min_ipv4_network_size,
        min_ipv6_network_size,
    );

    for extra_ranges_file in extra_ranges_files {
        read_single_line_ranges(
            &mut File::open(&extra_ranges_file)?,
            &mut filtered_ranges,
            false,
        )?;
    }

    if do_merge {
        merge_ranges(&mut filtered_ranges);
    }

    let stdout = io::stdout();
    let mut stdout = BufWriter::new(stdout.lock());
    for network in filtered_ranges {
        if network.is_selected() {
            writeln!(stdout, "{}", network.range())?;
        }
    }
    stdout.flush()?;

    Ok(())
}
