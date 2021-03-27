use crate::utils::expand_ranges::expand_ranges;
use crate::MergeOptions;
use anyhow::Error;
use libnetrangemerge::{merge_ranges, IpRange, RangeInterest};
use std::fs::File;
use std::io::{self, BufRead, Write as _};
use std::path::Path;

fn read_ranges<R: io::Read>(reader: R) -> Result<Vec<RangeInterest<IpRange>>, Error> {
    let bufreader = io::BufReader::new(reader);
    bufreader
        .lines()
        .map(|line| {
            let range: IpRange = line?.parse()?;
            Ok(RangeInterest::new(range, true))
        })
        .collect()
}

pub fn merge_command(options: MergeOptions) -> Result<(), Error> {
    let mut ranges = if options.file.as_path() == Path::new("-") {
        read_ranges(io::stdin().lock())?
    } else {
        read_ranges(File::open(&options.file)?)?
    };

    merge_ranges(&mut ranges);

    expand_ranges(
        &mut ranges,
        options.min_ipv4_network_size,
        options.min_ipv6_network_size,
    );

    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    for r in ranges {
        writeln!(stdout, "{}", r.range())?;
    }

    Ok(())
}
