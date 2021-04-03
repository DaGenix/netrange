use crate::utils::expand_ranges::expand_ranges;
use crate::MergeOptions;
use anyhow::Error;
use libnetrangemerge::{merge_ranges, IpRange, RangeInterest};
use std::fs::File;
use std::io::{self, BufRead, Write as _};

fn read_ranges<R: io::Read>(
    reader: R,
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

pub fn merge_command(options: MergeOptions) -> Result<(), Error> {
    let mut ranges = Vec::new();
    if let Some(file) = options.file {
        read_ranges(File::open(&file)?, &mut ranges, true)?
    } else {
        read_ranges(io::stdin().lock(), &mut ranges, true)?
    };

    for extra_file in options.extra_file.into_iter() {
        read_ranges(File::open(&extra_file)?, &mut ranges, false)?
    }

    expand_ranges(
        ranges.iter_mut().map(|n| n.range_mut()),
        options.min_ipv4_network_size,
        options.min_ipv6_network_size,
    );

    merge_ranges(&mut ranges);

    let stdout = io::stdout();
    let mut stdout = io::BufWriter::new(stdout.lock());
    for r in ranges {
        writeln!(stdout, "{}", r.range())?;
    }
    stdout.flush()?;

    Ok(())
}
