use anyhow::Error;
use libnetrangemerge::{merge_networks, IpNetwork, NetworkInterest};
use serde::Deserialize;
use std::fs::File;
use std::str::FromStr;

const SELECTED_KNOWN_AMAZON_IP_RANGES: &[&'static str] = &[
    "15.177.0.0/16",
    "15.190.0.0/15",
    "15.192.0.0/15",
    "18.128.0.0/9",
    "18.32.0.0/11",
    "18.64.0.0/10",
    "3.0.0.0/8",
    "35.152.0.0/13",
    "35.160.0.0/12",
    "35.176.0.0/13",
    "52.0.0.0/10",
    "52.64.0.0/12",
    "52.84.0.0/14",
    "52.88.0.0/13",
    "54.144.0.0/12",
    "54.160.0.0/11",
    "54.192.0.0/12",
    "54.208.0.0/13",
    "54.216.0.0/14",
    "54.220.0.0/15",
    "54.224.0.0/11",
    "99.150.0.0/17",
    "99.77.128.0/17",
    "99.78.0.0/18",
    "2400:6500::/32",
    "2400:6700::/32",
    "2400:7fc0::/32",
    "2403:b300::/32",
    "2404:c2c0::/32",
    "2406:da00::/24",
    "240f:8000::/24",
    "2600:1F00::/24",
    "2600:9000::/28",
    "2606:F40::/32",
    "2620:107:3000::/44",
    "2620:107:4000::/44",
    "2804:800::/32",
    "2a01:578::/32",
    "2a05:d000::/25",
];

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct AwsRanges {
    #[allow(dead_code)]
    syncToken: String,
    #[allow(dead_code)]
    createDate: String,
    prefixes: Vec<AwsIpv4Range>,
    ipv6_prefixes: Vec<AwsIpv6Range>,
}

#[derive(Deserialize, Debug)]
struct AwsIpv4Range {
    ip_prefix: String,
    region: String,
    service: String,
    network_border_group: String,
}

#[derive(Deserialize, Debug)]
struct AwsIpv6Range {
    ipv6_prefix: String,
    region: String,
    service: String,
    network_border_group: String,
}

fn convert_to_ranges(
    aws_ip_ranges: AwsRanges,
    filter_function: &str,
) -> Result<Vec<NetworkInterest<IpNetwork>>, Error> {
    let lua = rlua::Lua::new();

    lua.context(|ctx| -> Result<(), Error> {
        let func = ctx.load(filter_function).into_function()?;
        ctx.globals().set("func", func)?;
        Ok(())
    })?;

    fn is_interesting(
        lua: &rlua::Lua,
        is_ipv6: bool,
        region: String,
        service: String,
        network_border_group: String,
    ) -> Result<bool, Error> {
        let interesting = lua.context(|ctx| -> Result<bool, Error> {
            ctx.globals().set("is_ipv6", is_ipv6)?;
            ctx.globals().set("region", region)?;
            ctx.globals().set("service", service)?;
            ctx.globals()
                .set("network_border_group", network_border_group)?;
            let func: rlua::Function<'_> = ctx.globals().get("func")?;
            Ok(func.call(())?)
        })?;
        Ok(interesting)
    }

    let ipv4_ranges = aws_ip_ranges.prefixes.into_iter().map(|range| {
        let network = range.ip_prefix.parse()?;
        let interesting = is_interesting(
            &lua,
            false,
            range.region,
            range.service,
            range.network_border_group,
        )?;
        Ok(NetworkInterest::new(network, interesting))
    });
    let ipv6_ranges = aws_ip_ranges.ipv6_prefixes.into_iter().map(|range| {
        let network = range.ipv6_prefix.parse()?;
        let interesting = is_interesting(
            &lua,
            true,
            range.region,
            range.service,
            range.network_border_group,
        )?;
        Ok(NetworkInterest::new(network, interesting))
    });

    ipv4_ranges.chain(ipv6_ranges).collect()
}

fn main() -> Result<(), Error> {
    let file = File::open("ip-ranges.json")?;
    let aws_ranges: AwsRanges = serde_json::from_reader(file)?;

    let mut all_networks = convert_to_ranges(
        aws_ranges,
        "return region == 'us-east-1' and service == 'EC2' and not is_ipv6",
    )?;

    // AWS owns some IP ranges that we know about - so, we can add those in
    // as extra knowledge to reduce the number of networks.
    for known_range in SELECTED_KNOWN_AMAZON_IP_RANGES {
        all_networks.push(NetworkInterest::new(
            IpNetwork::from_str(known_range).unwrap(),
            false,
        ));
    }

    merge_networks(&mut all_networks);

    for network in all_networks {
        if network.is_interesting() {
            println!("{}", network.network());
        }
    }

    Ok(())
}
