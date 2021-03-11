use crate::{InvalidNetworkError, Network, IpNetwork, NetworkInterest};
use cidr::Cidr;
use std::cmp::Ordering;

fn dummies_first(a: &NetworkInterest, b: &NetworkInterest) -> Option<Ordering> {
    match (a.is_dummy(), b.is_dummy()) {
        (true, true) => Some(Ordering::Equal),
        (true, false) => Some(Ordering::Less),
        (false, true) => Some(Ordering::Greater),
        (false, false) => None,
    }
}

// This sort arranges things first by address type and then
// puts everything in the propper order to find and remove
// overlapping networks.
// ASSUMES: No dummy networks
// IPV4 then IPV6
// Smaller addresses to bigger addresses
// Bigger networks to smaller networks
fn sort_before_merging(a: &NetworkInterest, b: &NetworkInterest) -> Ordering {
    let ipv4_first = a
        .network()
        .host_address()
        .is_ipv6()
        .cmp(&a.network().host_address().is_ipv6());
    let smaller_addresses_first = a.network().host_address().cmp(&b.network().host_address());
    let bigger_networks_first = a
        .network()
        .network_length()
        .cmp(&b.network().network_length());
    ipv4_first
        .then(smaller_addresses_first)
        .then(bigger_networks_first)
}

// This sort is run after all of the networks of the same size
// are merged to prepare for merging networks of the next size.
// ASSUMES: Everything has the same family
// Dummies first
// Smaller networks to bigger networks
// Smaller addresses to bigger addresses
fn sort_during_merging(a: &NetworkInterest, b: &NetworkInterest) -> Ordering {
    if let Some(ord) = dummies_first(a, b) {
        return ord;
    }
    let smaller_networks_first = a
        .network()
        .network_length()
        .cmp(&b.network().network_length())
        .reverse();
    let smaller_addresses_first = a.network().host_address().cmp(&b.network().host_address());
    smaller_networks_first.then(smaller_addresses_first)
}

fn compact(networks: &mut [NetworkInterest]) -> usize {
    if let Some(mut open_idx) = networks.iter().position(|x| x.is_dummy()) {
        let mut start_search = open_idx + 1;
        while let Some(next_item_idx) = networks[start_search..].iter().position(|x| !x.is_dummy())
        {
            networks[open_idx] = networks[start_search + next_item_idx];
            networks[start_search + next_item_idx].set_dummy();
            open_idx += 1;
            start_search += next_item_idx + 1;
        }

        open_idx
    } else {
        networks.len()
    }
}

fn try_merge_overlapping(
    network1: &NetworkInterest,
    network2: &NetworkInterest,
) -> Option<NetworkInterest> {
    assert!(network1.network().host_address() <= network2.network().host_address());
    assert_eq!(
        network1.network().is_ipv6(),
        network2.network().is_ipv6()
    );

    if network1.network().contains(&network2.network()) {
        Some(NetworkInterest::new(
            network1.network(),
            network1.is_interesting() || network2.is_interesting(),
        ))
    } else {
        None
    }
}

// ASSUMES: networks sorted by address (small to large) and then
// by network size (big to small)
fn remove_overlapping_networks_in_place(mut networks: &mut [NetworkInterest]) {
    while networks.len() >= 2 {
        if let Some(n) = try_merge_overlapping(&networks[0], &networks[1]) {
            networks[0].set_dummy();
            networks[1] = n;
        }
        networks = &mut networks[1..];
    }
}

fn try_merge_adjacent(
    network1: &NetworkInterest,
    network2: &NetworkInterest,
) -> Option<NetworkInterest> {
    assert!(network1.network().host_address() < network2.network().host_address());
    assert_eq!(
        network1.network().is_ipv6(),
        network2.network().is_ipv6(),
    );
    assert_eq!(
        network1.network().network_length(),
        network2.network().network_length(),
    );

    // 1) 127.0.0.0/31
    // 2) 127.0.0.2/31
    // 3) 127.0.0.4/31
    // Networks 1 & 2 are adjacent since 127.0.0.0/30 covers them both
    // Networks 2 & 3 are _not_ adjacent since 127.0.0.2/30 is not a valid network

    // Step 1: Try to upgrade network1 into the next biggest sized network
    let bigger_network = match IpNetwork::new(
        network1.network().host_address().clone(),
        network1.network().network_length() - 1,
    ) {
        Ok(n) => n,
        Err(InvalidNetworkError::InvalidHostAddress(_)) => return None,
        Err(InvalidNetworkError::InvalidNetworkLength(_)) => unreachable!(),
    };

    // Step 2: Check to see if that new network contains network2
    if bigger_network.contains(&network2.network()) {
        Some(NetworkInterest::new(
            bigger_network,
            network1.is_interesting() || network2.is_interesting(),
        ))
    } else {
        None
    }
}

// ASSUMES: networks are sorted first by network size (small to large)
// and then by address (small to large)
fn merge_networks_in_place(networks: &mut [NetworkInterest]) {
    fn find_end_of_chunk_idx(nets: &[NetworkInterest], current_length: u8) -> usize {
        nets.iter()
            .position(|n| !n.is_dummy() && n.network().network_length() != current_length)
            .unwrap_or(nets.len())
    }

    fn merge_networks_of_equal_length_in_place(mut nets: &mut [NetworkInterest]) {
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
    // Step 0: If we have no remaining networks to process, exit
    // Step 1: Select a chunk of networks that all have the same network lengths from remaining networks
    //     1b: After the first iteration - we must re-sort this chunk!
    // Step 2: Merge & sort networks in that chunk
    // Step 3: Find where the next chunk begins - and drop everything before that from the remaining networks
    // Step 4: Go to Step 1

    let mut first_iteration = true;
    let mut remaining = networks;
    while !remaining.is_empty() {
        // Step 1:
        let network_length = remaining[0].network().network_length();
        let end_of_chunk_idx = find_end_of_chunk_idx(remaining, network_length);
        let chunk = &mut remaining[..end_of_chunk_idx];
        if first_iteration {
            first_iteration = false;
        } else {
            chunk.sort_unstable_by(sort_during_merging);
        }

        // Step 2:
        merge_networks_of_equal_length_in_place(chunk);
        chunk.sort_unstable_by(sort_during_merging);

        // Step 3:
        let start_of_next_chunk_idx = find_end_of_chunk_idx(remaining, network_length);
        remaining = &mut remaining[start_of_next_chunk_idx..];
    }
}

pub fn merge_networks(networks: &mut [NetworkInterest]) -> usize {
    networks.sort_unstable_by(sort_before_merging);

    let first_ipv6_network = networks
        .iter()
        .position(|n| n.network().is_ipv6())
        .unwrap_or(networks.len());
    let (ipv4_networks, ipv6_networks) = networks.split_at_mut(first_ipv6_network);

    fn do_merge(mut networks: &mut [NetworkInterest]) {
        remove_overlapping_networks_in_place(networks);

        let len = compact(networks);
        networks = &mut networks[..len];

        networks.sort_unstable_by(sort_during_merging);

        merge_networks_in_place(networks);
    }

    do_merge(ipv4_networks);
    do_merge(ipv6_networks);

    compact(networks)
}

#[cfg(test)]
mod test {
    use crate::merge::merge_networks;
    use crate::{NetworkInterest, IpNetwork, Network};
    use std::cmp::Ordering;

    // ASSUMES: No Dummies
    // IPV4 then IPV6
    // Smaller addresses to bigger addresses
    // Bigger networks to smaller networks
    fn sort_standard(a: &NetworkInterest, b: &NetworkInterest) -> Ordering {
        let ipv4_first = a
            .network()
            .host_address()
            .is_ipv6()
            .cmp(&a.network().host_address().is_ipv6());
        let smaller_addresses_first = a.network().host_address().cmp(&b.network().host_address());
        let bigger_networks_first = a
            .network()
            .network_length()
            .cmp(&b.network().network_length());
        ipv4_first
            .then(smaller_addresses_first)
            .then(bigger_networks_first)
    }

    #[test]
    fn test_remove_overlapping_networks_1() {
        let mut networks = vec![
            NetworkInterest::new("127.0.0.8/29".parse().unwrap(), false),
            NetworkInterest::new("127.0.0.16/29".parse().unwrap(), true),
            NetworkInterest::new("0.0.0.0/0".parse().unwrap(), false),
        ];
        let len = merge_networks(&mut networks);
        networks.truncate(len);
        networks.sort_unstable_by(sort_standard);
        assert_eq!(networks.len(), 1);
        assert_eq!(
            networks[0],
            NetworkInterest::new("0.0.0.0/0".parse().unwrap(), true)
        );
    }

    #[test]
    fn test_remove_overlapping_networks_2() {
        let mut networks = vec![
            NetworkInterest::new("127.0.0.0/32".parse().unwrap(), false),
            NetworkInterest::new("127.0.0.1/32".parse().unwrap(), true),
            NetworkInterest::new("127.0.0.0/31".parse().unwrap(), true),
            NetworkInterest::new("127.0.1.0/32".parse().unwrap(), false),
            NetworkInterest::new("127.0.1.1/32".parse().unwrap(), false),
            NetworkInterest::new("127.0.1.0/31".parse().unwrap(), false),
        ];
        let len = merge_networks(&mut networks);
        networks.truncate(len);
        networks.sort_unstable_by(sort_standard);
        assert_eq!(networks.len(), 2);
        assert_eq!(
            networks[0],
            NetworkInterest::new("127.0.0.0/31".parse().unwrap(), true)
        );
        assert_eq!(
            networks[1],
            NetworkInterest::new("127.0.1.0/31".parse().unwrap(), false)
        );
    }

    #[test]
    fn test_merge_1() {
        let mut networks = vec![
            NetworkInterest::new("127.0.0.0/31".parse().unwrap(), false),
            NetworkInterest::new("127.0.0.2/31".parse().unwrap(), true),
            NetworkInterest::new("127.0.0.4/31".parse().unwrap(), true),
            NetworkInterest::new("127.0.0.8/31".parse().unwrap(), true),
            NetworkInterest::new("127.0.0.10/31".parse().unwrap(), true),
        ];
        let len = merge_networks(&mut networks);
        networks.truncate(len);
        networks.sort_unstable_by(sort_standard);
        assert_eq!(networks.len(), 3);
        assert_eq!(
            networks[0],
            NetworkInterest::new("127.0.0.0/30".parse().unwrap(), true)
        );
        assert_eq!(
            networks[1],
            NetworkInterest::new("127.0.0.4/31".parse().unwrap(), true)
        );
        assert_eq!(
            networks[2],
            NetworkInterest::new("127.0.0.8/30".parse().unwrap(), true)
        );
    }

    #[test]
    fn test_merge_2() {
        let mut networks = vec![
            NetworkInterest::new("127.0.0.0/31".parse().unwrap(), false),
            NetworkInterest::new("127.0.0.2/31".parse().unwrap(), true),
            NetworkInterest::new("127.0.0.4/31".parse().unwrap(), true),
            NetworkInterest::new("127.0.0.6/31".parse().unwrap(), true),
        ];
        let len = merge_networks(&mut networks);
        networks.truncate(len);
        networks.sort_unstable_by(sort_standard);
        assert_eq!(networks.len(), 1);
        assert_eq!(
            networks[0],
            NetworkInterest::new("127.0.0.0/29".parse().unwrap(), true)
        );
    }

    #[test]
    fn test_merge_3() {
        let mut networks = vec![
            NetworkInterest::new("127.0.0.0/31".parse().unwrap(), false),
            NetworkInterest::new("127.0.0.2/31".parse().unwrap(), true),
            NetworkInterest::new("127.0.0.0/30".parse().unwrap(), true),
            NetworkInterest::new("127.0.0.4/30".parse().unwrap(), true),
            NetworkInterest::new("127.0.0.8/31".parse().unwrap(), true),
            NetworkInterest::new("127.0.0.10/31".parse().unwrap(), true),
            NetworkInterest::new("127.0.4.0/23".parse().unwrap(), true),
            NetworkInterest::new("127.0.6.0/23".parse().unwrap(), true),
        ];
        let len = merge_networks(&mut networks);
        networks.truncate(len);
        networks.sort_unstable_by(sort_standard);
        assert_eq!(networks.len(), 3);
        assert_eq!(
            networks[0],
            NetworkInterest::new("127.0.0.0/29".parse().unwrap(), true)
        );
        assert_eq!(
            networks[1],
            NetworkInterest::new("127.0.0.8/30".parse().unwrap(), true)
        );
        assert_eq!(
            networks[2],
            NetworkInterest::new("127.0.4.0/22".parse().unwrap(), true)
        );
    }
}
