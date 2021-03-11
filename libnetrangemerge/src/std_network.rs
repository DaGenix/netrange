use cidr::{Cidr, Inet};
use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
use crate::Network;

pub struct InvalidNetworkLengthError {
    network_length: u8,
    max_length: u8,
}

impl Debug for InvalidNetworkLengthError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Invalid network length: {}. Network length must be {} or less",
            self.network_length, self.max_length
        )
    }
}

impl Display for InvalidNetworkLengthError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

pub struct InvalidHostAddressError {
    host_address: IpAddr,
    network_length: u8,
}

impl Debug for InvalidHostAddressError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ipinet: cidr::IpInet = cidr::Inet::new(self.host_address, self.network_length).unwrap();
        write!(
            f,
            "Invalid host address: {} is not the first address in the network: {}",
            ipinet.address(),
            ipinet.network(),
        )
    }
}

impl Display for InvalidHostAddressError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

pub enum InvalidNetworkError {
    InvalidNetworkLength(InvalidNetworkLengthError),
    InvalidHostAddress(InvalidHostAddressError),
}

impl Debug for InvalidNetworkError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            InvalidNetworkError::InvalidNetworkLength(err) => Debug::fmt(err, f),
            InvalidNetworkError::InvalidHostAddress(err) => Debug::fmt(err, f),
        }
    }
}

impl Display for InvalidNetworkError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Error for InvalidNetworkError {}

pub struct UnparseableNetworkError {
    text: String,
}

impl Debug for UnparseableNetworkError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Either the host or the network length portions of {} could not be parsed",
            self.text
        )
    }
}

impl Display for UnparseableNetworkError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

pub enum NetworkParseError {
    InvalidNetworkLength(InvalidNetworkLengthError),
    InvalidHostAddress(InvalidHostAddressError),
    UnparseableNetwork(UnparseableNetworkError),
}

impl Debug for NetworkParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            NetworkParseError::InvalidNetworkLength(err) => Debug::fmt(err, f),
            NetworkParseError::InvalidHostAddress(err) => Debug::fmt(err, f),
            NetworkParseError::UnparseableNetwork(err) => Debug::fmt(err, f),
        }
    }
}

impl Display for NetworkParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Error for NetworkParseError {}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct IpNetwork {
    host_address: IpAddr,
    network_length: u8,
}

impl IpNetwork {
    fn new(host_address: IpAddr, network_length: u8) -> Result<IpNetwork, InvalidNetworkError> {
        match host_address {
            IpAddr::V4(_) => {
                if network_length > 32 {
                    return Err(InvalidNetworkError::InvalidNetworkLength(
                        InvalidNetworkLengthError {
                            network_length,
                            max_length: 32,
                        },
                    ));
                }
            }
            IpAddr::V6(_) => {
                if network_length > 128 {
                    return Err(InvalidNetworkError::InvalidNetworkLength(
                        InvalidNetworkLengthError {
                            network_length,
                            max_length: 128,
                        },
                    ));
                }
            }
        }
        let ipinet: cidr::IpInet = cidr::Inet::new(host_address, network_length).unwrap();
        if ipinet.first_address() != host_address {
            return Err(InvalidNetworkError::InvalidHostAddress(
                InvalidHostAddressError {
                    host_address,
                    network_length,
                },
            ));
        }
        Ok(IpNetwork {
            host_address,
            network_length,
        })
    }
}

impl Network for IpNetwork {
    type Address = IpAddr;

    fn embiggen(&self) -> Option<Self> {
        if self.network_length == 0 {
            return None;
        }
        match IpNetwork::new(self.host_address, self.network_length - 1) {
            Ok(n) => Some(n),
            Err(InvalidNetworkError::InvalidHostAddress(_)) => return None,
            Err(InvalidNetworkError::InvalidNetworkLength(_)) => unreachable!(),
        }
    }

    fn host_address(&self) -> &Self::Address {
        &self.host_address
    }

    fn network_length(&self) -> u8 {
        self.network_length
    }

    fn is_ipv6(&self) -> bool {
        let c = cidr::IpCidr::new(self.host_address, self.network_length).unwrap();
        c.is_ipv6()
    }

    fn contains(&self, other: &Self) -> bool {
        let c = cidr::IpCidr::new(self.host_address, self.network_length).unwrap();
        c.contains(&other.host_address)
    }
}

impl Debug for IpNetwork {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.host_address, self.network_length)
    }
}

impl Display for IpNetwork {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.host_address, self.network_length)
    }
}

fn parse_error<T>(s: &str) -> Result<T, NetworkParseError> {
    Err(NetworkParseError::UnparseableNetwork(
        UnparseableNetworkError {
            text: s.to_string(),
        },
    ))
}

impl FromStr for IpNetwork {
    type Err = NetworkParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let idx_of_slash = if let Some(pos) = s.find("/") {
            pos
        } else {
            return parse_error(s);
        };

        let host_part = &s[0..idx_of_slash];
        let network_length_part = &s[idx_of_slash..][1..];

        let host_address = if let Ok(host_address) = host_part.parse() {
            host_address
        } else {
            return parse_error(s);
        };

        let network_length = if let Ok(network_length) = network_length_part.parse() {
            network_length
        } else {
            return parse_error(s);
        };

        match IpNetwork::new(host_address, network_length) {
            Ok(network) => Ok(network),
            Err(InvalidNetworkError::InvalidNetworkLength(err)) => {
                Err(NetworkParseError::InvalidNetworkLength(err))
            }
            Err(InvalidNetworkError::InvalidHostAddress(err)) => {
                Err(NetworkParseError::InvalidHostAddress(err))
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Ipv4Network {
    host_address: Ipv4Addr,
    network_length: u8,
}

impl Ipv4Network {
    fn new(host_address: Ipv4Addr, network_length: u8) -> Result<Ipv4Network, InvalidNetworkError> {
        if network_length > 32 {
            return Err(InvalidNetworkError::InvalidNetworkLength(
                InvalidNetworkLengthError {
                    network_length,
                    max_length: 32,
                },
            ));
        }
        let ipinet = cidr::Ipv4Inet::new(host_address, network_length).unwrap();
        if ipinet.first_address() != host_address {
            return Err(InvalidNetworkError::InvalidHostAddress(
                InvalidHostAddressError {
                    host_address: host_address.into(),
                    network_length,
                },
            ));
        }
        Ok(Ipv4Network {
            host_address,
            network_length,
        })
    }
}

impl Network for Ipv4Network {
    type Address = Ipv4Addr;

    fn embiggen(&self) -> Option<Self> {
        if self.network_length == 0 {
            return None;
        }
        match Ipv4Network::new(self.host_address, self.network_length - 1) {
            Ok(n) => Some(n),
            Err(InvalidNetworkError::InvalidHostAddress(_)) => return None,
            Err(InvalidNetworkError::InvalidNetworkLength(_)) => unreachable!(),
        }
    }

    fn host_address(&self) -> &Self::Address {
        &self.host_address
    }

    fn network_length(&self) -> u8 {
        self.network_length
    }

    fn is_ipv6(&self) -> bool {
        false
    }

    fn contains(&self, other: &Self) -> bool {
        let c = cidr::Ipv4Cidr::new(self.host_address, self.network_length).unwrap();
        c.contains(&other.host_address)
    }
}

impl Debug for Ipv4Network {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.host_address, self.network_length)
    }
}

impl Display for Ipv4Network {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.host_address, self.network_length)
    }
}

impl FromStr for Ipv4Network {
    type Err = NetworkParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let idx_of_slash = if let Some(pos) = s.find("/") {
            pos
        } else {
            return parse_error(s);
        };

        let host_part = &s[0..idx_of_slash];
        let network_length_part = &s[idx_of_slash..][1..];

        let host_address = if let Ok(host_address) = host_part.parse() {
            host_address
        } else {
            return parse_error(s);
        };

        let network_length = if let Ok(network_length) = network_length_part.parse() {
            network_length
        } else {
            return parse_error(s);
        };

        match Ipv4Network::new(host_address, network_length) {
            Ok(network) => Ok(network),
            Err(InvalidNetworkError::InvalidNetworkLength(err)) => {
                Err(NetworkParseError::InvalidNetworkLength(err))
            }
            Err(InvalidNetworkError::InvalidHostAddress(err)) => {
                Err(NetworkParseError::InvalidHostAddress(err))
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Ipv6Network {
    host_address: Ipv6Addr,
    network_length: u8,
}

impl Ipv6Network {
    fn new(host_address: Ipv6Addr, network_length: u8) -> Result<Ipv6Network, InvalidNetworkError> {
        if network_length > 128 {
            return Err(InvalidNetworkError::InvalidNetworkLength(
                InvalidNetworkLengthError {
                    network_length,
                    max_length: 128,
                },
            ));
        }
        let ipinet = cidr::Ipv6Inet::new(host_address, network_length).unwrap();
        if ipinet.first_address() != host_address {
            return Err(InvalidNetworkError::InvalidHostAddress(
                InvalidHostAddressError {
                    host_address: host_address.into(),
                    network_length,
                },
            ));
        }
        Ok(Ipv6Network {
            host_address,
            network_length,
        })
    }
}

impl Network for Ipv6Network {
    type Address = Ipv6Addr;

    fn embiggen(&self) -> Option<Self> {
        if self.network_length == 0 {
            return None;
        }
        match Ipv6Network::new(self.host_address, self.network_length - 1) {
            Ok(n) => Some(n),
            Err(InvalidNetworkError::InvalidHostAddress(_)) => return None,
            Err(InvalidNetworkError::InvalidNetworkLength(_)) => unreachable!(),
        }
    }

    fn host_address(&self) -> &Self::Address {
        &self.host_address
    }

    fn network_length(&self) -> u8 {
        self.network_length
    }

    fn is_ipv6(&self) -> bool {
        true
    }

    fn contains(&self, other: &Self) -> bool {
        let c = cidr::Ipv6Cidr::new(self.host_address, self.network_length).unwrap();
        c.contains(&other.host_address)
    }
}

impl Debug for Ipv6Network {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.host_address, self.network_length)
    }
}

impl Display for Ipv6Network {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.host_address, self.network_length)
    }
}

impl FromStr for Ipv6Network {
    type Err = NetworkParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let idx_of_slash = if let Some(pos) = s.find("/") {
            pos
        } else {
            return parse_error(s);
        };

        let host_part = &s[0..idx_of_slash];
        let network_length_part = &s[idx_of_slash..][1..];

        let host_address = if let Ok(host_address) = host_part.parse() {
            host_address
        } else {
            return parse_error(s);
        };

        let network_length = if let Ok(network_length) = network_length_part.parse() {
            network_length
        } else {
            return parse_error(s);
        };

        match Ipv6Network::new(host_address, network_length) {
            Ok(network) => Ok(network),
            Err(InvalidNetworkError::InvalidNetworkLength(err)) => {
                Err(NetworkParseError::InvalidNetworkLength(err))
            }
            Err(InvalidNetworkError::InvalidHostAddress(err)) => {
                Err(NetworkParseError::InvalidHostAddress(err))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::network::Network;
    use crate::std_network::{InvalidNetworkError, IpNetwork, NetworkParseError};

    #[test]
    fn test_new_ipv4() {
        IpNetwork::new("0.0.0.0".parse().unwrap(), 0).unwrap();
        IpNetwork::new("127.0.0.0".parse().unwrap(), 12).unwrap();
        IpNetwork::new("127.0.8.0".parse().unwrap(), 21).unwrap();

        match IpNetwork::new("127.0.0.1".parse().unwrap(), 12) {
            Err(InvalidNetworkError::InvalidHostAddress(_)) => {}
            _ => panic!("Expected InvalidHostAddress failure"),
        }

        match IpNetwork::new("127.0.0.0".parse().unwrap(), 99) {
            Err(InvalidNetworkError::InvalidNetworkLength(_)) => {}
            _ => panic!("Expected InvalidNetworkLength failure"),
        }

        match IpNetwork::new("127.0.0.0".parse().unwrap(), 0) {
            Err(InvalidNetworkError::InvalidHostAddress(_)) => {}
            _ => panic!("Expected InvalidHostAddress failure"),
        }
    }

    #[test]
    fn test_parsing_ipv4() {
        "0.0.0.0/0".parse::<IpNetwork>().unwrap();
        "127.0.0.0/12".parse::<IpNetwork>().unwrap();
        "127.0.8.0/21".parse::<IpNetwork>().unwrap();

        match "127.0.0.1/12".parse::<IpNetwork>() {
            Err(NetworkParseError::InvalidHostAddress(_)) => {}
            _ => panic!("Expected InvalidHostAddress failure"),
        }

        match "127.0.0.0/99".parse::<IpNetwork>() {
            Err(NetworkParseError::InvalidNetworkLength(_)) => {}
            _ => panic!("Expected InvalidNetworkLength failure"),
        }

        match "127.0.0.0/0".parse::<IpNetwork>() {
            Err(NetworkParseError::InvalidHostAddress(_)) => {}
            _ => panic!("Expected InvalidHostAddress failure"),
        }

        match "127.0.0.0x12".parse::<IpNetwork>() {
            Err(NetworkParseError::UnparseableNetwork(_)) => {}
            _ => panic!("Expected UnparseableNetwork failure"),
        }

        match "127.0.0x0/12".parse::<IpNetwork>() {
            Err(NetworkParseError::UnparseableNetwork(_)) => {}
            _ => panic!("Expected UnparseableNetwork failure"),
        }

        match "127.0.0.0/x".parse::<IpNetwork>() {
            Err(NetworkParseError::UnparseableNetwork(_)) => {}
            _ => panic!("Expected UnparseableNetwork failure"),
        }

        match "127.0.0.0/".parse::<IpNetwork>() {
            Err(NetworkParseError::UnparseableNetwork(_)) => {}
            _ => panic!("Expected UnparseableNetwork failure"),
        }
    }

    #[test]
    fn test_new_ipv6() {
        IpNetwork::new("::".parse().unwrap(), 0).unwrap();
        IpNetwork::new("::".parse().unwrap(), 125).unwrap();
        IpNetwork::new("::8".parse().unwrap(), 125).unwrap();

        match IpNetwork::new("::1".parse().unwrap(), 12) {
            Err(InvalidNetworkError::InvalidHostAddress(_)) => {}
            _ => panic!("Expected InvalidHostAddress failure"),
        }

        match IpNetwork::new("::".parse().unwrap(), 129) {
            Err(InvalidNetworkError::InvalidNetworkLength(_)) => {}
            _ => panic!("Expected InvalidNetworkLength failure"),
        }

        match IpNetwork::new("::8".parse().unwrap(), 0) {
            Err(InvalidNetworkError::InvalidHostAddress(_)) => {}
            _ => panic!("Expected InvalidHostAddress failure"),
        }
    }

    #[test]
    fn test_parsing_ipv6() {
        "::/0".parse::<IpNetwork>().unwrap();
        "::/125".parse::<IpNetwork>().unwrap();
        "::8/125".parse::<IpNetwork>().unwrap();

        match "::1/12".parse::<IpNetwork>() {
            Err(NetworkParseError::InvalidHostAddress(_)) => {}
            _ => panic!("Expected InvalidHostAddress failure"),
        }

        match "::/129".parse::<IpNetwork>() {
            Err(NetworkParseError::InvalidNetworkLength(_)) => {}
            _ => panic!("Expected InvalidNetworkLength failure"),
        }

        match "::8/1".parse::<IpNetwork>() {
            Err(NetworkParseError::InvalidHostAddress(_)) => {}
            _ => panic!("Expected InvalidHostAddress failure"),
        }

        match "::8x125".parse::<IpNetwork>() {
            Err(NetworkParseError::UnparseableNetwork(_)) => {}
            _ => panic!("Expected UnparseableNetwork failure"),
        }

        match "::x/125".parse::<IpNetwork>() {
            Err(NetworkParseError::UnparseableNetwork(_)) => {}
            _ => panic!("Expected UnparseableNetwork failure"),
        }

        match "::8/x".parse::<IpNetwork>() {
            Err(NetworkParseError::UnparseableNetwork(_)) => {}
            _ => panic!("Expected UnparseableNetwork failure"),
        }

        match "::8/".parse::<IpNetwork>() {
            Err(NetworkParseError::UnparseableNetwork(_)) => {}
            _ => panic!("Expected UnparseableNetwork failure"),
        }
    }
}
