# libnetrangemerge

libnetrangemege implements an algorithm that takes in a list of
[CIDR](https://en.wikipedia.org/wiki/Classless_Inter-Domain_Routing) network
ranges and produces a minimal output set of CIDR ranges where all overlapping
and adjacent ranges have been merged.

[![Crates.io](https://img.shields.io/crates/v/libnetrangemerge.svg)](https://crates.io/crates/libnetrangemerge)
[![Documentation](https://docs.rs/libnetrangemerge/badge.svg)](https://docs.rs/libnetrangemerge)
[![MIT/Apache-2 licensed](https://img.shields.io/crates/l/libnetrangemerge.svg)](./LICENSE-APACHE)
[![Bors enabled](https://bors.tech/images/badge_small.svg)](https://app.bors.tech/repositories/32984)

Given the list of CIDR ranges `0.0.0.0/0` and `127.0.0.0/8`, libnetrangemerge
will produce the output CIDR range `0.0.0.0/0` since this range completely
covers the second range.

Given the list of CIDR ranges `127.0.0.0/25` and `127.0.0.128/25`, libnetrangemerge
will produce the output CIDR range `127.0.0.0/24` since the two input ranges
were adjacent. Not all adjacent ranges can be merged, however. For example,
`127.0.0.1/32` and `127.0.0.2/32` are adjacent, but cannot be merged because
there is no way to represent the range `127.0.0.1` - `127.0.0.2` in CIDR form.

CIDR range merging isn't useful for all applications that operate on CIDR
ranges. If an application wants to make different decisions depending on
the particular range, merging those ranges into a smaller set won't be useful.
However, for applications that treat a list of ranges equivalently, merging
them into a minimal set can be useful.

## Range Selection

Every input CIDR range to libnetrangemerge must be marked with a `selected`
flag. When two ranges are merged, if either input range is marked as `selected`,
the output range will also be marked as `selected`; and if neither is marked
the output range will also not be marked. The `selected` status of ranges
has no other effect on merging.

This feature can be useful in minimizing ranges in some specific cases.
For example, if an application wants to get the set of CIDR ranges that
are used by Amazon's EC2 servers, it can fetch the list of all of the
Amazon IP ranges, filter out anything that isn't used by EC2, and then
merge the resulting set. Unfortunately, not all the EC2 ranges may be
adjacent to each other - there may be ranges in the middle that are used
by other Amazon services. If the application wants to create the smallest
set of ranges possible, it may be useful to use these other ranges in order
to fill in the gaps between the EC2 ranges. The application can accomplish
this by passing every Amazon range into libnetrangemerge, but with the
EC2 ranges marked as `selected` and the other ranges not marked. The
output set can then be filtered by throwing away any output range not
marked as `selected`. The output set will no longer represent _just_
the EC2 ranges in this case - but if the application is ok with that,
this can result in a smaller output set.

The `selected` functionality is optional - if an application doesn't
want to use it, the application can just mark every range as either
`selected` or not and then not filter anything out of the result.

## Usage

A CIDR range is represented by a struct that implements the `Range` trait.
3 implementations of that trait are included in the library: `IpRange`,
`Ipv4Range`, and `Ipv6Range`. An application may also implement the `Range` trait
for their own types as well.

The merging operation is implemented by creating a list of range values
and then passing that list to either `merge_ranges` or `merge_ranges_slice`.
The two functions implement the same algorithm - the difference is that the
former function expects a `&mut Vec<Range>` which it will merge in place
and then truncate to the size of the output set, while the latter expects
a `&mut [Range]` which it will merge in place and then return the size
of the output set. When using `merge_ranges_slice` it is the responsibility
of the application not to access any of the ranges past the returned size -
doing so is safe but will result in a panic.

Both `merge_ranges` and `merge_ranges_slice` operate in place, do not allocate,
and will not fail or panic, assuming that none of the methods on the `Range` type
panics.

## Example

```rust
use libnetrangemerge::{RangeInterest, IpRange, merge_ranges};

let mut ranges: Vec<RangeInterest<IpRange>> = vec![
   RangeInterest::new("127.0.0.8/29".parse().unwrap(), true),
   RangeInterest::new("127.0.0.16/29".parse().unwrap(), true),
   RangeInterest::new("0.0.0.0/0".parse().unwrap(), true),
];

merge_ranges(&mut ranges);
```

## no_std Support

libnetrangemerge is no_std compatible. However, in no_std mode the `merge_ranges`
method is unavailable as are the built in range types. The application can implement
its own type that implements the `Range` trait and pass instances of that
type to `merge_ranges_slice` for merging.

## Minimum Rust version policy

libnetrangemerge supports rustc 1.45 and later.

The minimum supported rustc version may be bumped with minor
revisions.

## License

This project is licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
   https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   https://opensource.org/licenses/MIT)

at your option.
