use crate::ip_to_string;
use crate::types::{
    AddrPairs, IfAddrs, ADDR_ADDR, AF_ALG, AF_INET, AF_INET6, AF_NETLINK, AF_PACKET, AF_VSOCK,
    BROADCAST_ADDR, MASK_ADDR, PEER_ADDR,
};
use nix::ifaddrs;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;

pub fn linux_interfaces() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut s: HashSet<String> = HashSet::new();

    let addrs = ifaddrs::getifaddrs()?;

    for addr in addrs {
        s.insert(addr.interface_name);
    }

    Ok(Vec::from_iter(s))
}

fn add_to_types_mat(
    af_class: u8,
    addr: &dyn Display,
    class: &str,
    types_mat: &mut HashMap<i32, Vec<AddrPairs>>,
    any: &mut bool,
) {
    let e = types_mat.entry(af_class.into()).or_default();

    if !*any {
        *any = true;
        e.push(HashMap::new());
    };

    let l = e.len();
    e[l - 1].insert(class.to_string(), format!("{addr}"));
}

pub fn linux_ifaddresses(if_name: &str) -> Result<IfAddrs, Box<dyn std::error::Error>> {
    let mut types_mat: HashMap<i32, Vec<AddrPairs>> = HashMap::new();
    let if_addrs = nix::ifaddrs::getifaddrs()?;

    for if_addr in if_addrs {
        if if_name != if_addr.interface_name {
            continue;
        }

        // Addr of the interface
        let mut any = false;

        for (name, ss) in vec![
            (ADDR_ADDR, if_addr.address),
            (MASK_ADDR, if_addr.netmask),
            (BROADCAST_ADDR, if_addr.broadcast),
            (PEER_ADDR, if_addr.destination),
        ] {
            if let Some(address) = ss {
                if let Some(mac_addr) = address.as_link_addr() {
                    add_to_types_mat(AF_PACKET, mac_addr, name, &mut types_mat, &mut any);
                }

                #[cfg(not(any(target_os = "ios", target_os = "macos")))]
                if let Some(net_link) = address.as_netlink_addr() {
                    add_to_types_mat(AF_NETLINK, net_link, name, &mut types_mat, &mut any);
                }

                #[cfg(not(any(target_os = "ios", target_os = "macos")))]
                if let Some(vsock_addr) = address.as_vsock_addr() {
                    add_to_types_mat(AF_VSOCK, vsock_addr, name, &mut types_mat, &mut any);
                }

                if let Some(inet_addr) = address.as_sockaddr_in() {
                    add_to_types_mat(
                        AF_INET,
                        &ip_to_string(inet_addr.ip()),
                        name,
                        &mut types_mat,
                        &mut any,
                    );
                }

                #[cfg(not(any(target_os = "ios", target_os = "macos")))]
                if let Some(alg_addr) = address.as_alg_addr() {
                    add_to_types_mat(AF_ALG, alg_addr, name, &mut types_mat, &mut any);
                }

                if let Some(inet_addr) = address.as_sockaddr_in6() {
                    add_to_types_mat(AF_INET6, &inet_addr.ip(), name, &mut types_mat, &mut any);
                }
            }
        }
    }

    Ok(types_mat)
}
