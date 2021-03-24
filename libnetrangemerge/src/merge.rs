use crate::{Range, RangeInterest};
use core::cmp::Ordering;

fn dummies_first<R: Range>(a: &RangeInterest<R>, b: &RangeInterest<R>) -> Option<Ordering> {
    match (a.is_dummy(), b.is_dummy()) {
        (true, true) => Some(Ordering::Equal),
        (true, false) => Some(Ordering::Less),
        (false, true) => Some(Ordering::Greater),
        (false, false) => None,
    }
}

// This sort arranges things first by address type and then
// puts everything in the propper order to find and remove
// overlapping ranges.
// ASSUMES: No dummy ranges
// IPV4 then IPV6
// Smaller addresses to bigger addresses
// Bigger ranges to smaller ranges
fn sort_before_merging<R: Range>(a: &RangeInterest<R>, b: &RangeInterest<R>) -> Ordering {
    let ipv4_first = a.range().is_ipv6().cmp(&a.range().is_ipv6());
    let smaller_addresses_first = a.range().host_address().cmp(&b.range().host_address());
    let bigger_ranges_first = a.range().prefix_length().cmp(&b.range().prefix_length());
    ipv4_first
        .then(smaller_addresses_first)
        .then(bigger_ranges_first)
}

// This sort is run after all of the ranges of the same size
// are merged to prepare for merging ranges of the next size.
// ASSUMES: Everything has the same family
// Dummies first
// Smaller ranges to bigger ranges
// Smaller addresses to bigger addresses
fn sort_during_merging<R: Range>(a: &RangeInterest<R>, b: &RangeInterest<R>) -> Ordering {
    if let Some(ord) = dummies_first(a, b) {
        return ord;
    }
    let smaller_ranges_first = a
        .range()
        .prefix_length()
        .cmp(&b.range().prefix_length())
        .reverse();
    let smaller_addresses_first = a.range().host_address().cmp(&b.range().host_address());
    smaller_ranges_first.then(smaller_addresses_first)
}

fn compact<R: Range>(ranges: &mut [RangeInterest<R>]) -> usize {
    if let Some(mut open_idx) = ranges.iter().position(|x| x.is_dummy()) {
        let mut start_search = open_idx + 1;
        while let Some(next_item_idx) = ranges[start_search..].iter().position(|x| !x.is_dummy()) {
            ranges[open_idx] = ranges[start_search + next_item_idx].clone();
            ranges[start_search + next_item_idx].set_dummy();
            open_idx += 1;
            start_search += next_item_idx + 1;
        }

        open_idx
    } else {
        ranges.len()
    }
}

fn try_merge_overlapping<R: Range>(
    range1: &RangeInterest<R>,
    range2: &RangeInterest<R>,
) -> Option<RangeInterest<R>> {
    assert!(range1.range().host_address() <= range2.range().host_address());
    assert_eq!(range1.range().is_ipv6(), range2.range().is_ipv6());

    if range1.range().contains(&range2.range()) {
        Some(RangeInterest::new(
            range1.range().clone(),
            range1.is_interesting() || range2.is_interesting(),
        ))
    } else {
        None
    }
}

// ASSUMES: ranges sorted by address (small to large) and then
// by range size (big to small)
fn remove_overlapping_ranges_in_place<R: Range>(mut ranges: &mut [RangeInterest<R>]) {
    while ranges.len() >= 2 {
        if let Some(n) = try_merge_overlapping(&ranges[0], &ranges[1]) {
            ranges[0].set_dummy();
            ranges[1] = n;
        }
        ranges = &mut ranges[1..];
    }
}

fn try_merge_adjacent<R: Range>(
    range1: &RangeInterest<R>,
    range2: &RangeInterest<R>,
) -> Option<RangeInterest<R>> {
    assert!(range1.range().host_address() < range2.range().host_address());
    assert_eq!(range1.range().is_ipv6(), range2.range().is_ipv6(),);
    assert_eq!(
        range1.range().prefix_length(),
        range2.range().prefix_length(),
    );

    // 1) 127.0.0.0/31
    // 2) 127.0.0.2/31
    // 3) 127.0.0.4/31
    // Ranges 1 & 2 are adjacent since 127.0.0.0/30 covers them both
    // Ranges 2 & 3 are _not_ adjacent since 127.0.0.2/30 is not a valid range

    // Step 1: Try to upgrade range1 into the next biggest sized range
    let bigger_range = if let Some(bigger_range) = range1.range().embiggen() {
        bigger_range
    } else {
        return None;
    };

    // Step 2: Check to see if that new range contains range2
    if bigger_range.contains(&range2.range()) {
        Some(RangeInterest::new(
            bigger_range,
            range1.is_interesting() || range2.is_interesting(),
        ))
    } else {
        None
    }
}

// ASSUMES: ranges are sorted first by range size (small to large)
// and then by address (small to large)
fn merge_ranges_in_place<R: Range>(ranges: &mut [RangeInterest<R>]) {
    fn find_end_of_chunk_idx<R: Range>(nets: &[RangeInterest<R>], current_length: u8) -> usize {
        nets.iter()
            .position(|n| !n.is_dummy() && n.range().prefix_length() != current_length)
            .unwrap_or(nets.len())
    }

    fn merge_ranges_of_equal_length_in_place<R: Range>(mut nets: &mut [RangeInterest<R>]) {
        while nets.len() >= 2 {
            if let Some(n) = try_merge_adjacent(&nets[0], &nets[1]) {
                nets[0].set_dummy();
                nets[1] = n;
                nets = &mut nets[2..];
            } else {
                nets = &mut nets[1..];
            }
        }
    }

    // Algorithm:
    // Step 0: If we have no remaining ranges to process, exit
    // Step 1: Select a chunk of ranges that all have the same range lengths from remaining ranges
    //     1b: After the first iteration - we must re-sort this chunk!
    // Step 2: Merge & sort ranges in that chunk
    // Step 3: Find where the next chunk begins - and drop everything before that from the remaining ranges
    // Step 4: Go to Step 1

    let mut first_iteration = true;
    let mut remaining = ranges;
    while !remaining.is_empty() {
        // Step 1:
        let prefix_length = remaining[0].range().prefix_length();
        let end_of_chunk_idx = find_end_of_chunk_idx(remaining, prefix_length);
        let chunk = &mut remaining[..end_of_chunk_idx];
        if first_iteration {
            first_iteration = false;
        } else {
            chunk.sort_unstable_by(sort_during_merging);
        }

        // Step 2:
        merge_ranges_of_equal_length_in_place(chunk);
        chunk.sort_unstable_by(sort_during_merging);

        // Step 3:
        let start_of_next_chunk_idx = find_end_of_chunk_idx(remaining, prefix_length);
        remaining = &mut remaining[start_of_next_chunk_idx..];
    }
}

/// Merges all provided ranges in place, returning the number of valid ranges.
///
/// As this operation is performed in place, after the operation is complete
/// some number of ranges at the end of the input slice may no longer be
/// valid. Attempting to access them in any way will panic!().
///
/// This function does not allocate and is no_std compatible.
///
/// # Example
///
/// ```
/// use libnetrangemerge::{RangeInterest, IpRange, merge_ranges_slice};
///
/// let mut ranges: Vec<RangeInterest<IpRange>> = vec![
///     RangeInterest::new("127.0.0.8/29".parse().unwrap(), false),
///     RangeInterest::new("127.0.0.16/29".parse().unwrap(), true),
///     RangeInterest::new("0.0.0.0/0".parse().unwrap(), false),
/// ];
///
/// let len = merge_ranges_slice(&mut ranges);
///
/// ranges.truncate(len);
/// ```
pub fn merge_ranges_slice<R: Range>(ranges: &mut [RangeInterest<R>]) -> usize {
    ranges.sort_unstable_by(sort_before_merging);

    let first_ipv6_range = ranges
        .iter()
        .position(|n| n.range().is_ipv6())
        .unwrap_or(ranges.len());
    let (ipv4_ranges, ipv6_ranges) = ranges.split_at_mut(first_ipv6_range);

    fn do_merge<R: Range>(mut ranges: &mut [RangeInterest<R>]) {
        remove_overlapping_ranges_in_place(ranges);

        let len = compact(ranges);
        ranges = &mut ranges[..len];

        ranges.sort_unstable_by(sort_during_merging);

        merge_ranges_in_place(ranges);
    }

    do_merge(ipv4_ranges);
    do_merge(ipv6_ranges);

    compact(ranges)
}

#[cfg(test)]
mod test {
    use crate::merge::merge_ranges_slice;
    use crate::{IpRange, Range, RangeInterest};
    use std::cmp::Ordering;

    // ASSUMES: No Dummies
    // IPV4 then IPV6
    // Smaller addresses to bigger addresses
    // Bigger ranges to smaller ranges
    fn sort_standard<R: Range>(a: &RangeInterest<R>, b: &RangeInterest<R>) -> Ordering {
        let ipv4_first = a.range().is_ipv6().cmp(&a.range().is_ipv6());
        let smaller_addresses_first = a.range().host_address().cmp(&b.range().host_address());
        let bigger_ranges_first = a.range().prefix_length().cmp(&b.range().prefix_length());
        ipv4_first
            .then(smaller_addresses_first)
            .then(bigger_ranges_first)
    }

    #[test]
    fn test_remove_overlapping_ranges_1() {
        let mut ranges: Vec<RangeInterest<IpRange>> = vec![
            RangeInterest::new("127.0.0.8/29".parse().unwrap(), false),
            RangeInterest::new("127.0.0.16/29".parse().unwrap(), true),
            RangeInterest::new("0.0.0.0/0".parse().unwrap(), false),
        ];
        let len = merge_ranges_slice(&mut ranges);
        ranges.truncate(len);
        ranges.sort_unstable_by(sort_standard);
        assert_eq!(ranges.len(), 1);
        assert_eq!(
            ranges[0],
            RangeInterest::new("0.0.0.0/0".parse().unwrap(), true)
        );
    }

    #[test]
    fn test_remove_overlapping_ranges_2() {
        let mut ranges: Vec<RangeInterest<IpRange>> = vec![
            RangeInterest::new("127.0.0.0/32".parse().unwrap(), false),
            RangeInterest::new("127.0.0.1/32".parse().unwrap(), true),
            RangeInterest::new("127.0.0.0/31".parse().unwrap(), true),
            RangeInterest::new("127.0.1.0/32".parse().unwrap(), false),
            RangeInterest::new("127.0.1.1/32".parse().unwrap(), false),
            RangeInterest::new("127.0.1.0/31".parse().unwrap(), false),
        ];
        let len = merge_ranges_slice(&mut ranges);
        ranges.truncate(len);
        ranges.sort_unstable_by(sort_standard);
        assert_eq!(ranges.len(), 2);
        assert_eq!(
            ranges[0],
            RangeInterest::new("127.0.0.0/31".parse().unwrap(), true)
        );
        assert_eq!(
            ranges[1],
            RangeInterest::new("127.0.1.0/31".parse().unwrap(), false)
        );
    }

    #[test]
    fn test_merge_1() {
        let mut ranges: Vec<RangeInterest<IpRange>> = vec![
            RangeInterest::new("127.0.0.0/31".parse().unwrap(), false),
            RangeInterest::new("127.0.0.2/31".parse().unwrap(), true),
            RangeInterest::new("127.0.0.4/31".parse().unwrap(), true),
            RangeInterest::new("127.0.0.8/31".parse().unwrap(), true),
            RangeInterest::new("127.0.0.10/31".parse().unwrap(), true),
        ];
        let len = merge_ranges_slice(&mut ranges);
        ranges.truncate(len);
        ranges.sort_unstable_by(sort_standard);
        assert_eq!(ranges.len(), 3);
        assert_eq!(
            ranges[0],
            RangeInterest::new("127.0.0.0/30".parse().unwrap(), true)
        );
        assert_eq!(
            ranges[1],
            RangeInterest::new("127.0.0.4/31".parse().unwrap(), true)
        );
        assert_eq!(
            ranges[2],
            RangeInterest::new("127.0.0.8/30".parse().unwrap(), true)
        );
    }

    #[test]
    fn test_merge_2() {
        let mut ranges: Vec<RangeInterest<IpRange>> = vec![
            RangeInterest::new("127.0.0.0/31".parse().unwrap(), false),
            RangeInterest::new("127.0.0.2/31".parse().unwrap(), true),
            RangeInterest::new("127.0.0.4/31".parse().unwrap(), true),
            RangeInterest::new("127.0.0.6/31".parse().unwrap(), true),
        ];
        let len = merge_ranges_slice(&mut ranges);
        ranges.truncate(len);
        ranges.sort_unstable_by(sort_standard);
        assert_eq!(ranges.len(), 1);
        assert_eq!(
            ranges[0],
            RangeInterest::new("127.0.0.0/29".parse().unwrap(), true)
        );
    }

    #[test]
    fn test_merge_3() {
        let mut ranges: Vec<RangeInterest<IpRange>> = vec![
            RangeInterest::new("127.0.0.0/31".parse().unwrap(), false),
            RangeInterest::new("127.0.0.2/31".parse().unwrap(), true),
            RangeInterest::new("127.0.0.0/30".parse().unwrap(), true),
            RangeInterest::new("127.0.0.4/30".parse().unwrap(), true),
            RangeInterest::new("127.0.0.8/31".parse().unwrap(), true),
            RangeInterest::new("127.0.0.10/31".parse().unwrap(), true),
            RangeInterest::new("127.0.4.0/23".parse().unwrap(), true),
            RangeInterest::new("127.0.6.0/23".parse().unwrap(), true),
        ];
        let len = merge_ranges_slice(&mut ranges);
        ranges.truncate(len);
        ranges.sort_unstable_by(sort_standard);
        assert_eq!(ranges.len(), 3);
        assert_eq!(
            ranges[0],
            RangeInterest::new("127.0.0.0/29".parse().unwrap(), true)
        );
        assert_eq!(
            ranges[1],
            RangeInterest::new("127.0.0.8/30".parse().unwrap(), true)
        );
        assert_eq!(
            ranges[2],
            RangeInterest::new("127.0.4.0/22".parse().unwrap(), true)
        );
    }
}
