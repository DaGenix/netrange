use crate::{NetworkInterest, NetworkInterestState};
use cidr::Cidr;
use std::cmp::Ordering;

fn sort_networks_by_address_then_size(a: &NetworkInterest, b: &NetworkInterest) -> Ordering {
    let dummies_last = a.is_dummy().cmp(&b.is_dummy()).reverse();
    let smaller_host_addresses_first = a.network().host_address().cmp(&b.network().host_address());
    // A smaller network_length means a bigger network
    let bigger_networks_first = a
        .network()
        .network_length()
        .cmp(&b.network().network_length());
    dummies_last
        .then(smaller_host_addresses_first)
        .then(bigger_networks_first)
}

fn try_merge_overlapping(
    network1: &NetworkInterest,
    network2: &NetworkInterest,
) -> Option<NetworkInterest> {
    assert_ne!(
        sort_networks_by_address_then_size(network1, network2),
        Ordering::Greater
    );
    assert_eq!(
        network1.network().cidr().family(),
        network2.network().cidr().family()
    );

    if network1.network().contains(network2.network()) {
        Some(NetworkInterest::new(
            network1.network(),
            network1.is_interesting() || network2.is_interesting(),
        ))
    } else {
        None
    }
}

fn compact(networks: &mut [NetworkInterest]) -> usize {
    if let Some(mut open_idx) = networks.iter().position(|x| x.is_dummy()) {
        let mut start_search = open_idx + 1;
        while let Some(next_item_idx) = networks[start_search..].iter().position(|x| !x.is_dummy())
        {
            networks[open_idx] = networks[start_search + next_item_idx];
            open_idx += 1;
            start_search += next_item_idx + 1;
        }

        open_idx
    } else {
        networks.len()
    }
}

fn remove_overlapping_networks(networks: &mut [NetworkInterest]) -> usize {
    fn remove_overlapping_networks_in_place(mut networks: &mut [NetworkInterest]) {
        networks.sort_unstable_by(sort_networks_by_address_then_size);

        while networks.len() >= 2 {
            if let Some(n) = try_merge_overlapping(&networks[0], &networks[1]) {
                networks[0].set_dummy();
                networks[1] = n;
            }
            networks = &mut networks[1..];
        }
    }

    remove_overlapping_networks_in_place(networks);
    compact(networks)
}

#[cfg(test)]
mod test {
    use crate::merge::{remove_overlapping_networks, sort_networks_by_address_then_size};
    use crate::NetworkInterest;

    #[test]
    fn test_remove_overlapping_networks_1() {
        let mut networks = vec![
            NetworkInterest::new("127.0.0.8/29".parse().unwrap(), false),
            NetworkInterest::new("127.0.0.16/29".parse().unwrap(), true),
            NetworkInterest::new("0.0.0.0/0".parse().unwrap(), false),
        ];
        let len = remove_overlapping_networks(&mut networks);
        networks.truncate(len);
        networks.sort_unstable_by(sort_networks_by_address_then_size);
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
        let len = remove_overlapping_networks(&mut networks);
        networks.truncate(len);
        networks.sort_unstable_by(sort_networks_by_address_then_size);
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
}
