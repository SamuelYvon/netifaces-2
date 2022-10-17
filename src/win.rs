#![allow(dead_code)]

use crate::types::{IfAddrs, ADDR_ADDR, AF_INET, BROADCAST_ADDR, MASK_ADDR};
use crate::{ip_to_string, mac_to_string};
use std::alloc::{alloc, dealloc, Layout};
use std::collections::HashMap;
use std::mem::size_of;
use windows::Win32::NetworkManagement::IpHelper;
use windows::Win32::NetworkManagement::IpHelper::{
    IP_ADAPTER_INDEX_MAP, MIB_IFROW, MIB_IFTABLE, MIB_IPADDRROW_XP, MIB_IPADDRTABLE,
};
use windows::Win32::Networking::WinSock::AF_LINK;

const WIN_API_ALIGN: usize = 4;

struct WinIface {
    index: u32,
    name: String,
    phy_addr: [u8; 8],
}

fn string_from_w16(arr: &[u16]) -> String {
    let mut s = String::new();

    for a in arr.iter().map(|&c| c as u32) {
        // I don't know much about Window's API and even less about their string
        // fuckery, so hopefully this does it.
        match a {
            0 => break,
            n => s.push(unsafe { char::from_u32_unchecked(n) }),
        }
    }

    s
}

/// Given a big endian u32, returns it in little-endian.
/// This is independent of the underlying architecture.
fn be_to_le(s: u32) -> u32 {
    u32::to_le(u32::from_be(s))
}

// fn

fn get_iface_index(iface_name: &str) -> Option<u32> {
    let mut buff_size = 0;
    let mut index: Option<u32> = None;

    unsafe {
        // Get the length
        IpHelper::GetInterfaceInfo(None, &mut buff_size);
    };

    let layout = Layout::from_size_align(buff_size as usize, WIN_API_ALIGN).unwrap();
    let ptr = unsafe { alloc(layout) as *mut IpHelper::IP_INTERFACE_INFO };

    let interface_count = unsafe {
        // Actually get data
        IpHelper::GetInterfaceInfo(Some(ptr), &mut buff_size);
        (*ptr).NumAdapters as usize
    };

    let ptr_to_data =
        unsafe { ((ptr as *mut u8).add(size_of::<i32>())) as *mut [IP_ADAPTER_INDEX_MAP; 1] };

    for i in 0..interface_count {
        let iface = unsafe { *(ptr_to_data.add(i)) };
        let u16_data = iface[0].Name;
        let s = string_from_w16(&u16_data);

        if s == iface_name {
            index = Some(iface[0].Index);
            break;
        }
    }

    unsafe { dealloc(ptr as *mut u8, layout) };

    index
}

/// Return a vector of all windows interfaces detected by the system. This can be a fairly large
/// amount. No filtering is done.
fn windows_full_interfaces() -> Result<Vec<WinIface>, Box<dyn std::error::Error>> {
    let mut size = 0_u32;

    unsafe { IpHelper::GetIfTable(None, &mut size, false) };

    let layout = Layout::from_size_align(size as usize, WIN_API_ALIGN)?;

    let ptr = unsafe { alloc(layout) as *mut MIB_IFTABLE };

    unsafe { IpHelper::GetIfTable(Some(ptr), &mut size, false) };

    let num_entries = unsafe { (*ptr).dwNumEntries } as usize;
    let raw_ptr = ptr as *mut u8;
    let data_ptr = unsafe { raw_ptr.offset(size_of::<u32>().try_into()?) as *mut [MIB_IFROW; 1] };

    let mut result_vec = vec![];

    for i in 0..num_entries {
        let entry_ptr = unsafe { data_ptr.add(i) };
        let entry = unsafe { (*entry_ptr)[0] };

        let if_index = entry.dwIndex;
        let u16_name = entry.wszName;
        let phy_addr = entry.bPhysAddr;

        let name = string_from_w16(&u16_name);

        result_vec.push(WinIface {
            index: if_index,
            name,
            phy_addr,
        });
    }

    Ok(result_vec)
}

pub fn windows_interfaces() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    windows_full_interfaces().map(|vec| vec.into_iter().map(|win_iface| win_iface.name).collect())
}

fn ifaddresses_ipv4(
    if_addrs: &mut IfAddrs,
    if_index: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get the buff's size
    let mut size = 0;
    unsafe {
        IpHelper::GetIpAddrTable(None, &mut size, false);
    }

    // Create the layout
    let layout = Layout::from_size_align(size as usize, WIN_API_ALIGN)?;

    // Alloc
    let ptr = unsafe {
        let ptr = alloc(layout) as *mut MIB_IPADDRTABLE;
        IpHelper::GetIpAddrTable(Some(ptr), &mut size, false);
        ptr
    };

    // Deal w/it
    let number_of_entries = unsafe { (*ptr).dwNumEntries as usize };

    let raw_ptr = unsafe { ((ptr as *mut u8).add(size_of::<u32>())) as *mut u8 };

    let ip_addr_table_ptr = raw_ptr as *const [MIB_IPADDRROW_XP; 1];

    for i in 0..number_of_entries {
        let row = unsafe { (*(ip_addr_table_ptr.add(i)))[0] };

        let row_if_index = row.dwIndex;
        if row_if_index != if_index {
            continue;
        }

        // all of the following are in network byte order (big end) by the API's spec
        let net_addr = row.dwAddr;
        let subnet_mask = row.dwMask;
        let broad_addr = row.dwBCastAddr;

        let ent = if_addrs.entry(AF_INET.into());
        let addr_vec = ent.or_insert(vec![]);

        addr_vec.push(HashMap::from([
            (ADDR_ADDR.to_string(), ip_to_string(be_to_le(net_addr))),
            (
                BROADCAST_ADDR.to_string(),
                ip_to_string(be_to_le(broad_addr)),
            ),
            (MASK_ADDR.to_string(), ip_to_string(be_to_le(subnet_mask))),
        ]));
    }

    unsafe {
        dealloc(ptr as *mut u8, layout);
    };

    Ok(())
}

fn ifaddresses_mac(
    if_addrs: &mut IfAddrs,
    if_index: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut all_ifaces = windows_full_interfaces()?;

    let ifaces_with_index: Vec<WinIface> = all_ifaces
        .into_iter()
        .filter(|iface| iface.index == if_index)
        .collect();

    let entry = if_addrs.entry(AF_LINK.into());
    let macs = entry.or_insert(vec![]);

    for iface in ifaces_with_index {
        let m = HashMap::from([(ADDR_ADDR.to_string(), mac_to_string(&iface.phy_addr))]);
        macs.push(m);
    }

    Ok(())
}

pub fn windows_ifaddresses(if_name: &str) -> Result<IfAddrs, Box<dyn std::error::Error>> {
    let mut if_addrs: IfAddrs = HashMap::new();

    let if_index = get_iface_index(if_name)
        .ok_or_else(|| format!("The given interface name ({}) is not found", if_name))?;

    ifaddresses_ipv4(&mut if_addrs, if_index)?;
    ifaddresses_mac(&mut if_addrs, if_index)?;

    Ok(if_addrs)
}
