#![allow(dead_code)]

use crate::mac_to_string;
use crate::types::{IfAddrs, ADDR_ADDR, AF_INET, MASK_ADDR};
use std::alloc::{alloc, dealloc, Layout};
use std::collections::HashMap;
use std::mem::size_of;

use windows::Win32::Foundation::{CHAR, NO_ERROR, WIN32_ERROR};
use windows::Win32::NetworkManagement::IpHelper;

use windows::Win32::NetworkManagement::IpHelper::IP_ADAPTER_INFO;
use windows::Win32::NetworkManagement::IpHelper::IP_ADDR_STRING;

const WIN_API_ALIGN: usize = 4;
const AF_LINK: i32 = -1000;

struct WinIpInfo {
    ip_address: String,
    mask: String,
}

struct WinIface {
    name: String,
    ip_addresses: Vec<WinIpInfo>,
    mac_address: [u8; 6],
}

fn win_adapter_name_to_string(arr: &[CHAR]) -> String {
    let mut s = String::new();

    for c in arr {
        match c {
            CHAR(0) => break,
            CHAR(n) => s.push(unsafe { char::from_u32_unchecked(*n as u32) }),
        }
    }

    return s;
}

/// Given a big endian u32, returns it in little-endian.
/// This is independent of the underlying architecture.
fn be_to_le(s: u32) -> u32 {
    u32::to_le(u32::from_be(s))
}

fn win_ip_addr_list_to_vec(ip: IP_ADDR_STRING) -> Vec<WinIpInfo> {
    let mut r = vec![];

    loop {
        let info = WinIpInfo {
            ip_address: win_adapter_name_to_string(&ip.IpAddress.String),
            mask: win_adapter_name_to_string(&ip.IpMask.String),
        };

        r.push(info);

        unsafe {
            let x = ip.Next;
            if x.is_null() {
                break;
            }
        }
    }

    r
}

/// Return a vector of all windows interfaces detected by the system. This can be a fairly large
/// amount. No filtering is done.
fn win_explore_adapters() -> Result<Vec<WinIface>, Box<dyn std::error::Error>> {
    let mut result_vec = vec![];

    unsafe {
        let size = win_adapter_table_size();
        let layout = Layout::from_size_align(size as usize, WIN_API_ALIGN)?;
        let ptr = alloc(layout) as *mut IP_ADAPTER_INFO;

        if NO_ERROR != WIN32_ERROR(IpHelper::GetAdaptersInfo(Some(ptr), &mut (size as u32))) {
            // TODO: get the error string
            panic!("Failed to access the adapter information table:");
        }

        let number_of_entries = (size as usize) / size_of::<IP_ADAPTER_INFO>();

        for i in 0..number_of_entries {
            let entry = *ptr.add(i);

            // We use the description as it's more useful than a uuid
            let name = win_adapter_name_to_string(&entry.Description);

            let ip_addresses = win_ip_addr_list_to_vec(entry.IpAddressList);
            let mac_address = entry.Address;

            result_vec.push(WinIface {
                name,
                ip_addresses,
                mac_address,
            });
        }

        dealloc(ptr as *mut u8, layout);
    };

    Ok(result_vec)
}

fn win_adapter_table_size() -> usize {
    let mut size = 0_u32;
    unsafe {
        // Do not check return type; as per MS' docs, this will return an error code.
        IpHelper::GetAdaptersInfo(None, &mut size);
    };

    size as usize
}

pub fn windows_interfaces() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    win_explore_adapters().map(|vec| vec.into_iter().map(|win_iface| win_iface.name).collect())
}

fn ifaddresses_ipv4(
    interface: &WinIface,
    if_addrs: &mut IfAddrs,
) -> Result<(), Box<dyn std::error::Error>> {
    for win_ip_info in interface.ip_addresses.iter() {
        let ent = if_addrs.entry(AF_INET.into());
        let addr_vec = ent.or_insert(vec![]);

        // TODO: mask
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

/// List the IPv6 addrs. of the machine
fn ifaddresses_ipv6(
    interface: &WinIface,
    if_addrs: &mut IfAddrs,
) -> Result<(), Box<dyn std::error::Error>> {
    todo!("The API used to get Ipv4 does not work for ipv6");
    //     for win_ip_info in interface..iter() {
    //         let ent = if_addrs.entry(AF_INET6.into());
    //         let addr_vec = ent.or_insert(vec![]);
    //
    //         // TODO: mask
    //         // (
    //         //     BROADCAST_ADDR.to_string(),
    //         //     ip_to_string(be_to_le(broad_addr)),
    //         // ),
    //
    //         addr_vec.push(HashMap::from([
    //             (ADDR_ADDR.to_string(), win_ip_info.ip_address.clone()),
    //             (MASK_ADDR.to_string(), win_ip_info.mask.clone()),
    //         ]));
    //     }
}

fn ifaddresses_mac(
    interface: &WinIface,
    if_addrs: &mut IfAddrs,
) -> Result<(), Box<dyn std::error::Error>> {
    let entry = if_addrs.entry(AF_LINK.into());
    let macs = entry.or_insert(vec![]);

    let m = HashMap::from([(ADDR_ADDR.to_string(), mac_to_string(&interface.mac_address))]);
    macs.push(m);

    Ok(())
}

pub fn windows_ifaddresses(if_name: &str) -> Result<IfAddrs, Box<dyn std::error::Error>> {
    let mut if_addrs: IfAddrs = HashMap::new();

    let adapters = win_explore_adapters()?;
    let search_result: Vec<WinIface> = adapters
        .into_iter()
        .filter(|adapter| adapter.name == if_name)
        .collect();

    if search_result.len() == 0 {
        Err(format!(
            "Cannot find any interface with description {if_name}"
        ))?
    } else if search_result.len() > 1 {
        panic!("More than a single interface with description {if_name}")
    }

    let interface = search_result.get(0).unwrap();

    ifaddresses_ipv4(interface, &mut if_addrs)?;
    // ifaddresses_ipv6(interface, &mut if_addrs)?;
    ifaddresses_mac(interface, &mut if_addrs)?;

    Ok(if_addrs)
}
