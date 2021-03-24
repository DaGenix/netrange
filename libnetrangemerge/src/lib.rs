#![cfg_attr(not(feature = "std"), no_std)]

mod merge;
mod range;
mod range_interest;
#[cfg(feature = "std")]
mod std_range;

pub use merge::merge_ranges_slice;
pub use range::Range;
pub use range_interest::RangeInterest;
#[cfg(feature = "std")]
pub use std_range::{
    InvalidHostAddressError, InvalidRangeError, InvalidPrefixLengthError, IpRange,
    Ipv4Range, Ipv6Range, RangeParseError, UnparseableRangeError,
};

pub fn merge_ranges<N: Range>(ranges: &mut Vec<RangeInterest<N>>) {
    let len = merge_ranges_slice(ranges);
    ranges.truncate(len)
}
