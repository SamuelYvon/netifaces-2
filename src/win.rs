#![allow(dead_code)]
use crate::common::InterfaceDisplay;
use crate::types::{IfAddrs, ADDR_ADDR, AF_INET, MASK_ADDR};
use crate::{mac_to_string, types};
use std::alloc::{alloc, dealloc, Layout};
use std::collections::HashMap;
use std::error::Error;
use std::ffi::{c_void, CStr};
use std::mem::size_of;
use std::net::Ipv6Addr;
use std::ptr;

use windows::Win32::Foundation::{CHAR, ERROR_BUFFER_OVERFLOW, NO_ERROR, WIN32_ERROR};
use windows::Win32::NetworkManagement::IpHelper;
use windows::Win32::Networking::WinSock::{
    inet_ntop, AF_INET6, INET6_ADDRSTRLEN, SOCKADDR_IN, SOCKADDR_IN6, SOCKET_ADDRESS,
};

use crate::common::NetifacesError;
use windows::Win32::NetworkManagement::IpHelper::IP_ADDR_STRING;
use windows::Win32::NetworkManagement::IpHelper::{
    GetAdaptersAddresses, GET_ADAPTERS_ADDRESSES_FLAGS, IP_ADAPTER_ADDRESSES_LH, IP_ADAPTER_INFO,
};

const WIN_API_ALIGN: usize = 4;
const AF_LINK: i32 = -1000;

#[derive(Debug)]
struct WinIpInfo {
    ip_address: String,
    mask: String,
}

#[derive(Debug)]
struct WinIface {
    /// The windows name of the interface (some sort of UUID)
    name: String,
    /// The human-readable name of the interface
    description: String,
    index: u32,
    ip_addresses: Vec<WinIpInfo>,
    mac_address: Vec<u8>,
}

type Ipv6Endpoint = (Ipv6Addr, String);

type Ipv6Mapping = HashMap<String, Vec<Ipv6Endpoint>>;

fn win_adapter_name_to_string(arr: &[CHAR]) -> String {
    let mut s = String::new();

    for c in arr {
        match c {
            CHAR(0) => break,
            CHAR(n) => s.push(unsafe { char::from_u32_unchecked(*n as u32) }),
        }
    }

    s
}

fn win_ip_addr_list_to_vec(ip: IP_ADDR_STRING) -> Vec<WinIpInfo> {
    let mut r = vec![];

    loop {
        let info = WinIpInfo {
            ip_address: win_adapter_name_to_string(&ip.IpAddress.String),
            mask: win_adapter_name_to_string(&ip.IpMask.String),
        };

        // In my testing, GetAdaptersInfo returns an IP of "0.0.0.0" when the interface
        // is down and does not have an IP assigned.  So only include the IP if it's
        // not 0.0.0.0
        if info.ip_address != "0.0.0.0" {
            r.push(info);
        }

        let x = ip.Next;
        if x.is_null() {
            break;
        }
    }
    r
}

/// Get the size (in bytes) required for allocating all the IP_ADAPTER_INFO
/// structs. Might return an error in case it is unable to do so.
fn adapter_info_size_required() -> Result<u32, NetifacesError> {
    let mut size: u32 = 0;
    unsafe {
        let ret = WIN32_ERROR(IpHelper::GetAdaptersInfo(None, &mut size));
        // Do not check for other return values; as per MS' docs, this will return an error code.
        if ret != ERROR_BUFFER_OVERFLOW {
            Err(NetifacesError::SystemErrorCode(
                "IpHelper::GetAdaptersInfo(None, 0)".to_string(),
                ret.0,
            ))
        } else {
            Ok(size)
        }
    }
}

/// Return a vector of all windows interfaces detected by the system. This can be a fairly large
/// amount. No filtering is done.
fn win_explore_adapters() -> Result<Vec<WinIface>, Box<dyn std::error::Error>> {
    let mut result_vec = vec![];

    unsafe {
        let mut size = adapter_info_size_required()?;
        let number_of_entries = (size as usize) / size_of::<IP_ADAPTER_INFO>();

        let layout = Layout::from_size_align(size as usize, WIN_API_ALIGN)?;
        let ptr = alloc(layout) as *mut IP_ADAPTER_INFO;

        if NO_ERROR != WIN32_ERROR(IpHelper::GetAdaptersInfo(Some(ptr), &mut size)) {
            // TODO: get the error string
            dealloc(ptr as *mut u8, layout);
            panic!("Failed to access the adapter information table:");
        }

        for i in 0..number_of_entries {
            let entry = *ptr.add(i);

            // We use the description as it's more useful than a uuid
            let name = win_adapter_name_to_string(&entry.AdapterName);
            let description = win_adapter_name_to_string(&entry.Description);
            let index = entry.Index;

            let ip_addresses = win_ip_addr_list_to_vec(entry.IpAddressList);
            let addr_len = entry.AddressLength as usize;
            let address = entry.Address;

            let mac_address = address[..addr_len].to_vec();

            result_vec.push(WinIface {
                name,
                description,
                index,
                ip_addresses,
                mac_address,
            });
        }

        dealloc(ptr as *mut u8, layout);
    };

    Ok(result_vec)
}

fn ifaddresses_ipv4(
    interface: &WinIface,
    if_addrs: &mut IfAddrs,
) -> Result<(), Box<dyn std::error::Error>> {
    for win_ip_info in interface.ip_addresses.iter() {
        let ent = if_addrs.entry(AF_INET.into());
        let addr_vec = ent.or_default();

        // TODO: broadcast
        // (
        //     BROADCAST_ADDR.to_string(),
        //     ip_to_string(be_to_le(broad_addr)),
        // ),

        addr_vec.push(HashMap::from([
            (ADDR_ADDR.to_string(), win_ip_info.ip_address.clone()),
            (MASK_ADDR.to_string(), win_ip_info.mask.clone()),
        ]));
    }

    Ok(())
}

unsafe fn sockaddr_to_string(sock_addr: SOCKET_ADDRESS) -> String {
    let lp_sock = sock_addr.lpSockaddr;
    let sa_family = (*lp_sock).sa_family as u32;

    let mut buff = [0_u8; INET6_ADDRSTRLEN as usize];
    match sa_family {
        // AF_INET
        2 => {
            let s = lp_sock as *mut SOCKADDR_IN;
            let addr = ptr::addr_of!((*s).sin_addr) as *const c_void;
            inet_ntop(AF_INET as i32, addr, &mut buff);
        }
        // AF_INET6
        23 => {
            let s = lp_sock as *mut SOCKADDR_IN6;
            let addr = ptr::addr_of!((*s).sin6_addr) as *const c_void;
            inet_ntop(AF_INET6.0 as i32, addr, &mut buff);
        }
        _ => {}
    };

    CStr::from_bytes_until_nul(&buff)
        .unwrap()
        .to_string_lossy()
        .to_string()
}

/// Get the network adapters' address(es). Only returns Ipv6 information for now but could
/// be extended for v4.
unsafe fn adapters_addresses() -> Result<Ipv6Mapping, Box<dyn std::error::Error>> {
    let af_type = AF_INET6;
    let mut eps = HashMap::new();

    let mut size = 0;

    let flags = GET_ADAPTERS_ADDRESSES_FLAGS(0);

    let ret = WIN32_ERROR(GetAdaptersAddresses(af_type, flags, None, None, &mut size));
    if ERROR_BUFFER_OVERFLOW != ret {
        Err(NetifacesError::SystemErrorCode(
            "GetAdaptersAddresses::(None)".to_string(),
            ret.0,
        ))?;
    }

    let layout = Layout::from_size_align(size as usize, WIN_API_ALIGN)?;
    let ptr = alloc(layout) as *mut IP_ADAPTER_ADDRESSES_LH;

    let ret = WIN32_ERROR(GetAdaptersAddresses(
        af_type,
        flags,
        None,
        Some(ptr),
        &mut size,
    ));
    if NO_ERROR != ret {
        Err(NetifacesError::SystemErrorCode(
            "GetAdaptersAddresses::(ptr)".to_string(),
            ret.0,
        ))?;
    }

    let mut traversal = ptr;
    while !traversal.is_null() {
        let name = (*traversal).Description.to_string().unwrap();
        let phy = (*traversal).PhysicalAddress;
        let phy_len = (*traversal).PhysicalAddressLength as usize;
        let phy = Vec::from(&phy[..phy_len]);
        let phy = mac_to_string(&phy);

        let mut addr_ptr = (*traversal).FirstUnicastAddress;

        while !addr_ptr.is_null() {
            let sock_addr = (*addr_ptr).Address;
            let addr_str = sockaddr_to_string(sock_addr);

            let entry = eps.entry(name.clone()).or_insert(vec![]);

            let addr = addr_str.parse::<Ipv6Addr>()?;
            entry.push((addr, phy.clone()));

            addr_ptr = (*addr_ptr).Next;
        }

        traversal = (*traversal).Next;
    }

    Ok(eps)
}

/// List the IPv6 addrs. of the machine
fn ifaddresses_ipv6(
    interface: &WinIface,
    if_addrs: &mut IfAddrs,
) -> Result<(), Box<dyn std::error::Error>> {
    let addrs_per_iface = unsafe { adapters_addresses() }?;

    let addrs = match addrs_per_iface.get(&interface.description) {
        Some(arr) => arr,
        None => {
            return Ok(());
        }
    };

    for (ip, mask) in addrs {
        let addr_vec = if_addrs.entry(types::AF_INET6 as i32).or_default();

        addr_vec.push(HashMap::from([
            (ADDR_ADDR.to_string(), ip.to_string()),
            (MASK_ADDR.to_string(), mask.to_string()),
        ]));
    }

    Ok(())
}

fn ifaddresses_mac(
    interface: &WinIface,
    if_addrs: &mut IfAddrs,
) -> Result<(), Box<dyn std::error::Error>> {
    let entry = if_addrs.entry(AF_LINK);
    let macs = entry.or_default();

    let m = HashMap::from([(ADDR_ADDR.to_string(), mac_to_string(&interface.mac_address))]);
    macs.push(m);

    Ok(())
}

#[test]
fn adapters() -> Result<(), Box<dyn Error>> {
    let adapters = win_explore_adapters()?;
    dbg!(adapters);
    Ok(())
}

/// Given an interface name, returns all the addresses associated with that interface. The result
/// is shaped loosely in a map.
pub fn windows_ifaddresses(if_name: &str) -> Result<IfAddrs, Box<dyn std::error::Error>> {
    let mut if_addrs: IfAddrs = HashMap::new();

    let adapters = win_explore_adapters()?;
    let search_result: Vec<WinIface> = adapters
        .into_iter()
        .filter(|adapter| adapter.description == if_name)
        .collect();

    if search_result.is_empty() {
        Err(format!(
            "Cannot find any interface with description {if_name}"
        ))?
    } else if search_result.len() > 1 {
        panic!("More than a single interface with description {if_name}")
    }

    let interface = search_result.get(0).unwrap();

    ifaddresses_ipv4(interface, &mut if_addrs)?;
    ifaddresses_ipv6(interface, &mut if_addrs)?;
    ifaddresses_mac(interface, &mut if_addrs)?;

    Ok(if_addrs)
}

/// List all the network interfaces available on the system.
///
/// # Params
/// - `display`: an [InterfaceDisplay] that controls what ID is returned from the call to
///              identify the interface.
pub fn windows_interfaces(display: InterfaceDisplay) -> Result<Vec<String>, Box<dyn Error>> {
    win_explore_adapters().map(|vec| {
        vec.into_iter()
            .map(|win_iface| match display {
                InterfaceDisplay::HumanReadable => win_iface.description,
                InterfaceDisplay::MachineReadable => win_iface.name,
            })
            .collect()
    })
}

/// List all the network interfaces available on the system by their indexes
pub fn windows_interfaces_by_index(
    display: InterfaceDisplay,
) -> Result<types::IfacesByIndex, Box<dyn std::error::Error>> {
    let interfaces = win_explore_adapters();

    match interfaces {
        Ok(win_ifaces) => {
            let mut interfaces = types::IfacesByIndex::new();
            for win_iface in win_ifaces {
                if display == InterfaceDisplay::HumanReadable {
                    interfaces.insert(win_iface.index, win_iface.description);
                } else {
                    interfaces.insert(win_iface.index, win_iface.name);
                }
            }
            Ok(interfaces)
        }
        Err(err) => Err(err),
    }
}
