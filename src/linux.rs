use crate::common::InterfaceDisplay;
use crate::types::{
    AddrPairs, IfAddrs, IfacesByIndex, ADDR_ADDR, AF_INET, AF_INET6, AF_PACKET, BROADCAST_ADDR,
    MASK_ADDR, PEER_ADDR,
};
#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "openbsd")))]
use crate::types::{AF_ALG, AF_NETLINK, AF_VSOCK};
use crate::NetifacesError;
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
    let mut found_any = false;

    for if_addr in if_addrs {
        if if_name != if_addr.interface_name {
            continue;
        }
        found_any = true;

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

                #[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "openbsd")))]
                if let Some(net_link) = address.as_netlink_addr() {
                    add_to_types_mat(AF_NETLINK, net_link, name, &mut types_mat, &mut any);
                }

                #[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "openbsd")))]
                if let Some(vsock_addr) = address.as_vsock_addr() {
                    add_to_types_mat(AF_VSOCK, vsock_addr, name, &mut types_mat, &mut any);
                }

                if let Some(inet_addr) = address.as_sockaddr_in() {
                    add_to_types_mat(AF_INET, &inet_addr.ip(), name, &mut types_mat, &mut any);
                }

                #[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "openbsd")))]
                if let Some(alg_addr) = address.as_alg_addr() {
                    add_to_types_mat(AF_ALG, alg_addr, name, &mut types_mat, &mut any);
                }

                if let Some(inet_addr) = address.as_sockaddr_in6() {
                    add_to_types_mat(AF_INET6, &inet_addr.ip(), name, &mut types_mat, &mut any);
                }
            }
        }
    }

    if found_any {
        return Ok(types_mat);
    } else {
        let err_msg = format!("Failed to find an interface with the name {}", if_name);
        return Err(Box::new(NetifacesError(err_msg)));
    }
}

// SIOCGIFFLAGS constant currently not available from the libc crate on Apple platforms.
// Filed an issue: https://github.com/rust-lang/libc/issues/3626
#[cfg(any(target_os = "ios", target_os = "macos"))]
const SIOCGIFFLAGS: libc::c_ulong = 0xc0206911; // extracted from macos headers

// SIOCGIFFLAGS constant currently not available from the libc crate on OpenBSD
#[cfg(target_os = "openbsd")]
const SIOCGIFFLAGS: libc::c_ulong = 0xc0206911; // extracted from sys/sockio.h

#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "openbsd")))]
const SIOCGIFFLAGS: libc::c_ulong = libc::SIOCGIFFLAGS;

/// Read the flags from an interface using the SIOCGIFFLAGS
/// ioctl.
fn read_interface_flags(if_name: &str) -> Result<libc::c_short, Box<dyn std::error::Error>> {
    if if_name.len() >= libc::IFNAMSIZ {
        return Err("Interface name too long!".into());
    }

    // In order to use this IOCTL, we have to create a socket to use it on.
    // Any socket will do; it apparently does not need to be bound to the interface
    // in question.
    let socket = UdpSocket::bind((std::net::Ipv4Addr::UNSPECIFIED, 0))?;

    unsafe {
        // Create a zeroed structure which will be used for the ioctl
        let mut ifreq: libc::ifreq = std::mem::zeroed();

        // Copy in the name.
        // We checked the length earlier so we know it will fit.
        for byte_idx in 0..if_name.as_bytes().len() {
            ifreq.ifr_name[byte_idx] = if_name.as_bytes()[byte_idx] as libc::c_char;
        }
        ifreq.ifr_name[if_name.as_bytes().len()] = 0;

        // Run ioctl
        let ioctl_ret = libc::ioctl(socket.as_raw_fd(), SIOCGIFFLAGS.try_into()?, &ifreq);

        match ioctl_ret {
            0 => Ok(ifreq.ifr_ifru.ifru_flags),
            _ => {
                let err_msg = format!(
                    "Error reading interface flags for {if_name}: {}",
                    nix::errno::Errno::last()
                );
                Err(Box::new(NetifacesError(err_msg)))
            }
        }
    }
}

/// Get the status of an interface (up/down) on POSIX
pub fn posix_interface_is_up(if_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    // Testing on Linux, it looks like the flag we want is IFF_RUNNING, not IFF_UP.
    // IFF_UP is the "administrative" status, i.e. "ip link set <name> up/down".
    // IFF_RUNNING only sets when the interface is both administratively up
    // and can send data.
    // Ref: https://stackoverflow.com/questions/11679514/what-is-the-difference-between-iff-up-and-iff-running
    // On the other hand, MacOS seems to not follow this standard and sets IFF_RUNNING even
    // if there is not a network cable connected.  As far as I can tell, there is no difference between
    // the flags of an ethernet interface with a cable connected and one without!
    // The only way to tell is the absence of an IP address.
    Ok((read_interface_flags(if_name)? & libc::IFF_RUNNING as libc::c_short) != 0)
}
