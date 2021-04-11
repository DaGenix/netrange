# netrange

netrange is a CLI utility that is able to fetch lists of
IP ranges used by common cloud services, filter those lists
by attributes provided by the cloud providers, and then
optionally minimize the set of resulting ranges.

[![Crates.io](https://img.shields.io/crates/v/netrange.svg)](https://crates.io/crates/netrange)
[![Bors enabled](https://bors.tech/images/badge_small.svg)](https://app.bors.tech/repositories/32984)

An example:

```sh
netrange cloud get-merge aws --filter "return service == 'EC2' and region == 'us-east-1'"
```

will download the current list of IP ranges being used by AWS,
filter out everything except those being used for EC2 servers
in the us-east-1 region, and then minimize the result by merging
any adjacent IP ranges.

## Installation

[Download and install Rust](https://www.rust-lang.org/learn/get-started). Then, run:

```sh
cargo install netrange
```

## Merging and Reading

netrange support "merge" and "read" operations. Both operations
will read a set of ranges published by a cloud provider
and write ranges, 1-per line to STDOUT. The "merge" operations
will, however, also minimize the output by merging adjacent
ranges. The "read" operations will not merge any ranges, however -
whatever the cloud service lists for ranges is what will be output.

## Filtering and Selecting

netrange supports extracting a smaller set of ranges of interest
from the full set of ranges published by the cloud services
using LUA scripts. There are two operations available:
filtering and selecting. A filter program should return
a False value for any ranges that should be thrown away and
a True value for other ranges. A select program runs after
filtering and it should return a True value for any ranges
that _must_ be present in the output and a False value for
ranges that _may_ be present in the output, but don't have to
be.

As an example,

```sh
netrange cloud get-read aws --filter "return service == 'EC2' and region == 'us-east-1'"
```

Currently returns 124 ranges, as this is the number of IP ranges
that AWS publishes for EC2 servers in the us-east-1 region.

```sh
netrange cloud get-merge aws --filter "return service == 'EC2' and region == 'us-east-1'"
```

Currently returns 112 ranges, as some of the 124 ranges that
AWS publishes are adjacent and can be merged to produce a smaller
output set.

```sh
netrange cloud get-merge aws --select "return service == 'EC2' and region == 'us-east-1'"
```

Currently returns 90 ranges. We get the smaller number of output
ranges because we didn't throw away non-EC2 and non-us-east-1 ranges
but instead used them to minimize the output set: some EC2 ranges
may have gaps between them used by other services and by using those
ranges to fill in the gaps we can merge ranges more aggressively.
The tradeoff, of course, is that the output no longer represents
only the EC2 ranges.

Different cloud services provide different attributes available
for filtering and selecting. The `cloud filter-help <service>` subcommand
cane be used to see which attributes are available for a particular
service.

## Commands

### Cloud Get

The `cloud get` subcommand will fetch the source file
that contains IP ranges published by the given service
and write it to STDOUT. This will often be a JSON file - but
can be other formats as well.

Example:

```sh
netrange cloud get aws
```

### Cloud Read

The `cloud read` subcommand will read in the range file
provided by the service (which may have
been retrieved by `cloud get`) and write all IP ranges,
1-per line, to STDOUT.

Adjacent ranges are _not_ merged in the output.

The ranges that are printed may optionally be filtered
by attributes that the cloud service provides.

Example:

```sh
netrange cloud read aws aws-ip-ranges.json
```

### Cloud Merge

The `cloud merge` subcommand will read in the range file
provided by the service (which may have
been retrieved by `cloud get`) and write all IP ranges,
1-per line, to STDOUT.

Adjacent ranges _are_ merged in the output.

Filter and select LUA programs may be used to control
which ranges are represented in the output.

Example:

```sh
netrange cloud merge aws aws-ip-ranges.json
```

### Cloud Get Read

The `cloud get-read` subcommand is a shortcut for
first using the `cloud get` subcommand and then
feeding the result into the `cloud read` subcommand.

```sh
netrange cloud cloud-get aws
```

### Cloud Get Merge

The `cloud get-merge` subcommand is a shortcut for
first using the `cloud get` subcommand and then
feeding the result into the `cloud merge` subcommand.

```sh
netrange cloud cloud-merge aws
```

### Cloud Filter Help

The `cloud filter-help` subcommand will print to
STDOUT information about what attributes are available
for filtering and selecting for the given cloud service.

```sh
netrange cloud filter-help aws
```

### Merge

The `merge` subcommand will read in a list of IP
ranges from the given file (or STDIN if no file is
"-"), merge adjacent ranges, and then print
the resulting ranges to STDOUT.

```sh
netrange merge input-ranges.txt
```

## Minimum Rust version policy

netrange supports rustc 1.45 and later.

The minimum supported rustc version may be bumped with minor
revisions.

## License

This project is licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
   https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   https://opensource.org/licenses/MIT)

at your option.
