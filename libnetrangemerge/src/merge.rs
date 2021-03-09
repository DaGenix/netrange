use crate::network::DUMMY_NETWORK;
use crate::{NetworkInterest, DUMMY_NETWORK_INTEREST};
use cidr::Cidr;

fn try_merge_overlapping(
    network1: &NetworkInterest,
    network2: &NetworkInterest,
) -> Option<NetworkInterest> {
    assert!(network1.network() <= network2.network());
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
    if let Some(mut open_idx) = networks.iter().position(|x| x.network() == DUMMY_NETWORK) {
        let mut start_search = open_idx + 1;
        while let Some(next_item_idx) = networks[start_search..]
            .iter()
            .position(|x| x.network() != DUMMY_NETWORK)
        {
            networks[open_idx] = networks[start_search + next_item_idx];
            open_idx += 1;
            start_search = start_search + next_item_idx + 1;
        }

        open_idx
    } else {
        networks.len()
    }
}

fn remove_overlapping_networks(networks: &mut [NetworkInterest]) -> usize {
    fn remove_overlapping_networks_in_place(mut networks: &mut [NetworkInterest]) {
        networks.sort_unstable_by(|a, b| a.network().cmp(&b.network).reverse());

        while networks.len() >= 2 {
            if let Some(n) =
                try_merge_overlapping(&networks[networks.len() - 1], &networks[networks.len() - 2])
            {
                networks[networks.len() - 1] = DUMMY_NETWORK_INTEREST;
                networks[networks.len() - 2] = n;
            }
            let l = networks.len();
            networks = &mut networks[..l - 1];
        }
    }

    remove_overlapping_networks_in_place(networks);
    compact(networks)
}

#[cfg(test)]
mod test {
    use crate::merge::remove_overlapping_networks;
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
        networks.sort_unstable_by(|a, b| a.network.cmp(&b.network));
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
        networks.sort_unstable_by(|a, b| a.network.cmp(&b.network));
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
