//! libnetrangemerge implements an algorithm to minimize a list of
//! [CIDR](https://en.wikipedia.org/wiki/Classless_Inter-Domain_Routing) ranges.
//!
//! For example, let's say you have the ranges 127.0.0.0/25 and 127.0.0.128/25.
//! In many applications, you may want to make different decisions about traffic
//! bound for each distinct range. However, in some applications, you may not
//! need to - and in such a case, it can be convenient to instead merge those
//! two CIDR ranges into the single, equivalent range: 127.0.0.0/24.
//!
//! You may also care about some ranges more than other. For example, lets say
//! you have the ranges 127.0.0.0/24, 127.0.1.0/24, 127.0.2.0/24, and 127.0.3.0/24.
//! If, for whatever reason, you don't care about any of those ranges, you can minimize
//! them by just throwing them all away. However, if you care about 1 or more of them,
//! you can minimize the list by merging them to the range 127.0.0.0/22. libnetrangemerge
//! supports this concept by allowing each input range to be tagged with a boolean
//! to indicate interest. After merging, any range that was created from at least
//! one range with the interest flag set to true will also have its interest flag
//! set to true. The caller is then able to filter out ranges that it doesn't care
//! about in the results. Tagging input ranges with an interest flag produces smaller
//! output lists than removing uninteresting ranges before merging - imagine that any of
//! the 4 ranges above were missing - in that case our output would always have to
//! contain two ranges - a /24 and a /23 instead of just the /22.
//!
//! A CIDR range is represented by a struct that implements the [`Range`] trait.
//! 3 implementations of that trait are included in the library: [`IpRange`],
//! [`Ipv4Range`], and [`Ipv6Range`].
//!
//! A struct of range values can be passed to the [`merge_ranges`] or [`merge_ranges_slice`]
//! methods. The former is probably what you want - however, it requires
//! that the input ranges be in a [`Vec`]. If you would prefer to
//! pass in ranges in a regular slice, you can use the latter
//! method - but at the cost that it is slightly more complex to use. Both
//! operate in place and do not allocate.
//!
//! libnetrangemerge is no_std compatible. However, in no_std mode the [`merge_ranges`]
//! method is unavailable as are the built in range types. Any struct that implements
//! the [`Range`] trait should work. However, it needs to be implemented by the caller.

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
    InvalidHostAddressError, InvalidPrefixLengthError, InvalidRangeError, IpRange, Ipv4Range,
    Ipv6Range, RangeParseError, UnparseableRangeError,
};

/// Merges all provided ranges in place. The input is truncated to the
/// number of valid ranges post merging.
///
/// # Example
///
/// ```
/// use libnetrangemerge::{RangeInterest, IpRange, merge_ranges};
///
/// let mut ranges: Vec<RangeInterest<IpRange>> = vec![
///     RangeInterest::new("127.0.0.8/29".parse().unwrap(), false),
///     RangeInterest::new("127.0.0.16/29".parse().unwrap(), true),
///     RangeInterest::new("0.0.0.0/0".parse().unwrap(), false),
/// ];
///
/// merge_ranges(&mut ranges);
/// ```
pub fn merge_ranges<N: Range>(ranges: &mut Vec<RangeInterest<N>>) {
    let len = merge_ranges_slice(ranges);
    ranges.truncate(len)
}
