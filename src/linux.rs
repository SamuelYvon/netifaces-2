use crate::common::InterfaceDisplay;
use crate::types::{
    AddrPairs, IfAddrs, IfacesByIndex, ADDR_ADDR, AF_ALG, AF_INET, AF_INET6, AF_NETLINK, AF_PACKET,
    AF_VSOCK, BROADCAST_ADDR, MASK_ADDR, PEER_ADDR,
};
use nix::ifaddrs;
use nix::net::if_::if_nameindex;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::net::UdpSocket;
use std::os::fd::AsRawFd;

pub fn posix_interfaces(
    _display: InterfaceDisplay,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut s: HashSet<String> = HashSet::new();

    let addrs = ifaddrs::getifaddrs()?;

    for addr in addrs {
        s.insert(addr.interface_name);
    }

    Ok(Vec::from_iter(s))
}

pub fn posix_interfaces_by_index(
    _display: InterfaceDisplay,
) -> Result<IfacesByIndex, Box<dyn std::error::Error>> {
    let mut interfaces = IfacesByIndex::new();

    let iface_names_idxes = if_nameindex()?;

    for iface in &iface_names_idxes {
        let iface_index = iface.index() as usize;
        interfaces.insert(iface_index, iface.name().to_string_lossy().to_string());
    }

    Ok(interfaces)
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

pub fn posix_ifaddresses(if_name: &str) -> Result<IfAddrs, Box<dyn std::error::Error>> {
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
                    add_to_types_mat(AF_INET, &inet_addr.ip(), name, &mut types_mat, &mut any);
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

// Structures for using the SIOCGIFFLAGS ioctl.
// See here for reference: https://man7.org/linux/man-pages/man7/netdevice.7.html
// Original implementation from https://users.rust-lang.org/t/using-libc-ioctl-to-read-interface-flags/32506
const IFNAMSIZ: usize = 16;

#[repr(C)]
#[derive(Copy, Clone)]
struct Ifmap {
    pub mem_start: libc::c_ulong,
    pub mem_end: libc::c_ulong,
    pub base_addr: libc::c_ushort,
    pub irq: libc::c_uchar,
    pub dma: libc::c_uchar,
    pub port: libc::c_uchar,
}

#[repr(C)]
union IfReqPayload {
    pub ifr_addr: libc::sockaddr,
    pub ifr_dstaddr: libc::sockaddr,
    pub ifr_broadaddr: libc::sockaddr,
    pub ifr_netmask: libc::sockaddr,
    pub ifr_hwaddr: libc::sockaddr,
    pub ifr_flags: libc::c_short,
    pub ifr_ifindex: libc::c_int,
    pub ifr_metric: libc::c_int,
    pub ifr_mtu: libc::c_int,
    pub ifr_map: Ifmap,
    pub ifr_slave: [libc::c_uchar; IFNAMSIZ],
    pub ifr_newname: [libc::c_uchar; IFNAMSIZ],
    pub ifr_data: *const libc::c_uchar,
}

#[repr(C)]
struct IfReq {
    pub ifr_name: [libc::c_uchar; IFNAMSIZ],
    pub data_union: IfReqPayload,
}

/// Read the flags from an interface using the SIOCGIFFLAGS
/// ioctl.
fn read_interface_flags(if_name: &str) -> Result<libc::c_short, Box<dyn std::error::Error>> {
    if if_name.len() >= IFNAMSIZ {
        return Err("Interface name too long!".into());
    }

    // In order to use this IOCTL, we have to create a socket to use it on.
    // Any socket will do; it apparently does not need to be bound to the interface
    // in question.
    let socket = UdpSocket::bind((std::net::Ipv4Addr::UNSPECIFIED, 0))?;

    unsafe {
        // Create a zeroed structure which will be used for the ioctl
        let mut ifreq: IfReq = std::mem::zeroed();

        // Copy in the name.
        // We checked the length earlier so we know it will fit.
        ifreq.ifr_name[0..if_name.as_bytes().len()].copy_from_slice(if_name.as_bytes());
        ifreq.ifr_name[if_name.as_bytes().len()] = 0;

        // Run ioctl
        libc::ioctl(socket.as_raw_fd(), libc::SIOCGIFFLAGS, &ifreq);

        Ok(ifreq.data_union.ifr_flags)
    }
}
