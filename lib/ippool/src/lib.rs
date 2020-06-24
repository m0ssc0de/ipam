extern crate ipnetwork;
use std::convert::TryInto;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum IpPoolError {
    CheckOffsetError(std::num::TryFromIntError),
    OffsetTooBig,
}

impl From<std::num::TryFromIntError> for IpPoolError {
    fn from(err: std::num::TryFromIntError) -> IpPoolError {
        IpPoolError::CheckOffsetError(err)
    }
}

impl fmt::Display for IpPoolError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IpPoolError::OffsetTooBig => write!(f, "Offset is bigger than size of networking"),
            IpPoolError::CheckOffsetError(ref t) => t.fmt(f),
        }
    }
}

pub struct IpPool {
    net_iter: std::iter::Skip<ipnetwork::IpNetworkIterator>,
    ip_vec: std::vec::Vec<std::net::IpAddr>,
}

impl IpPool {
    pub fn new(net: ipnetwork::IpNetwork, offset: usize) -> Result<IpPool, IpPoolError> {
        match net.size() {
            ipnetwork::NetworkSize::V4(s) => {
                if offset > s.try_into()? {
                    return Err(IpPoolError::OffsetTooBig);
                }
            }
            ipnetwork::NetworkSize::V6(s) => {
                if offset > s.try_into()? {
                    return Err(IpPoolError::OffsetTooBig);
                }
            }
        }
        Ok(IpPool {
            net_iter: net.into_iter().skip(offset),
            ip_vec: Vec::new(),
        })
    }
    pub fn new_addr(&mut self) -> Option<std::net::IpAddr> {
        if let Some(ip) = self.ip_vec.pop() {
            return Some(ip);
        }
        self.net_iter.next()
    }
    pub fn recycle(&mut self, ip: std::net::IpAddr) {
        self.ip_vec.push(ip)
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let net: ipnetwork::IpNetwork = "192.168.100.1/24".parse().unwrap();
        let mut p = crate::IpPool::new(net, 253).unwrap();
        match p.new_addr() {
            Some(addr) => {
                assert_eq!("192.168.100.253".parse::<std::net::IpAddr>().unwrap(), addr);
            }
            None => assert!(false),
        }

        let net: ipnetwork::IpNetwork = "192.168.100.1/24".parse().unwrap();
        match crate::IpPool::new(net, 300) {
            Err(e) => assert_eq!(crate::IpPoolError::OffsetTooBig, e),
            Ok(_) => assert!(false),
        }

        let net: ipnetwork::IpNetwork = "192.168.100.1/24".parse().unwrap();
        let mut p = crate::IpPool::new(net, 253).unwrap();
        p.recycle("192.168.100.4".parse().unwrap());
        match p.new_addr() {
            Some(addr) => {
                assert_eq!("192.168.100.4".parse::<std::net::IpAddr>().unwrap(), addr);
            }
            None => assert!(false),
        }
        match p.new_addr() {
            Some(addr) => {
                assert_eq!("192.168.100.253".parse::<std::net::IpAddr>().unwrap(), addr);
            }
            None => assert!(false),
        }

        let net: ipnetwork::IpNetwork = "192.168.100.1/24".parse().unwrap();
        let mut p = crate::IpPool::new(net, 255).unwrap();
        match p.new_addr() {
            Some(addr) => {
                assert_eq!("192.168.100.255".parse::<std::net::IpAddr>().unwrap(), addr);
            }
            None => assert!(false),
        }
        match p.new_addr() {
            Some(_) => assert!(false),
            None => assert!(true),
        }
    }
}
