use crate::ip_to_string;
use crate::types::{
    AddrPairs, IfAddrs, ADDR_ADDR, AF_ALG, AF_INET, AF_INET6, AF_NETLINK, AF_PACKET, AF_VSOCK,
    BROADCAST_ADDR, MASK_ADDR, PEER_ADDR,
};
use std::alloc::{self, alloc, dealloc, Layout};
use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::fmt::{Display, Error};
use std::mem::size_of;
use std::ptr::NonNull;
use std::thread::sleep;
use std::time::Duration;
use windows::Win32::NetworkManagement::IpHelper;
use windows::Win32::NetworkManagement::IpHelper::{
    IP_ADAPTER_INDEX_MAP, IP_ADAPTER_INFO, MIB_IPADDRROW_XP, MIB_IPADDRTABLE,
};

const WIN_API_ALIGN: usize = 4;

struct WinIface {
    index: u32,
    name: String,
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

    let ptr_to_data = unsafe {
        ((ptr as *mut u8).offset(size_of::<i32>() as isize)) as *mut [IP_ADAPTER_INDEX_MAP; 1]
    };

    for i in 0..interface_count {
        let iface = unsafe { *(ptr_to_data.offset(i as isize)) };
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

pub fn windows_interfaces() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut interfaces = vec![];
    let mut buff_size = 0;

    unsafe {
        // Get the length
        IpHelper::GetInterfaceInfo(None, &mut buff_size);

        let layout = Layout::from_size_align_unchecked(buff_size as usize, WIN_API_ALIGN);
        let ptr = alloc(layout) as *mut IpHelper::IP_INTERFACE_INFO;

        // Actually get data
        IpHelper::GetInterfaceInfo(Some(ptr), &mut buff_size);

        let interface_count = (*ptr).NumAdapters as usize;

        let ptr_to_data =
            ((ptr as *mut u8).offset(size_of::<i32>() as isize)) as *mut [IP_ADAPTER_INDEX_MAP; 1];

        for i in 0..interface_count {
            let u16_data = (*(ptr_to_data.offset(i as isize)))[0].Name;
            interfaces.push(string_from_w16(&u16_data));
        }

        dealloc(ptr as *mut u8, layout);
    }

    Ok(interfaces)
}

fn ifaddresses_ipv4(if_addrs: &mut IfAddrs, if_index: u32) -> Result<(), Box<dyn std::error::Error>> {
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
    let mut number_of_entries = unsafe { (*ptr).dwNumEntries as usize };

    let raw_ptr = unsafe { ((ptr as *mut u8).offset(size_of::<u32>() as isize)) as *mut u8 };

    let ip_addr_table_ptr = raw_ptr as *const [MIB_IPADDRROW_XP; 1];

    for i in 0..number_of_entries {
        let row = unsafe { (*(ip_addr_table_ptr.offset(i as isize)))[0] };

        let row_if_index = row.dwIndex;
        if row_if_index != if_index {
            continue;
        }

        // all of the following are in network byte order (big end) by the API's spec
        let net_addr = row.dwAddr;
        let subnet_mask = row.dwMask;
        let broad_addr = row.dwBCastAddr;

        let mut ent = if_addrs.entry(AF_INET.into());
        let mut addr_vec = ent.or_insert(vec![]);

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

fn ifaddresses_ipv6(if_addrs: &mut IfAddrs, if_index: u32) -> Result<(), Box<dyn std::error::Error>> {
    // Get the buff's size
    let mut size = 0;
    unsafe {
        IpHelper::GetAdaptersAddresses(None, &mut size, false);
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
    let mut number_of_entries = unsafe { (*ptr).dwNumEntries as usize };

    let raw_ptr = unsafe { ((ptr as *mut u8).offset(size_of::<u32>() as isize)) as *mut u8 };

    let ip_addr_table_ptr = raw_ptr as *const [MIB_IPADDRROW_XP; 1];

    for i in 0..number_of_entries {
        let row = unsafe { (*(ip_addr_table_ptr.offset(i as isize)))[0] };

        let row_if_index = row.dwIndex;
        if row_if_index != if_index {
            continue;
        }

        // all of the following are in network byte order (big end) by the API's spec
        let net_addr = row.dwAddr;
        let subnet_mask = row.dwMask;
        let broad_addr = row.dwBCastAddr;

        let mut ent = if_addrs.entry(AF_INET.into());
        let mut addr_vec = ent.or_insert(vec![]);

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

pub fn windows_ifaddresses(if_name: &str) -> Result<IfAddrs, Box<dyn std::error::Error>> {
    let mut if_addrs: IfAddrs = HashMap::new();

    let if_index = get_iface_index(if_name)
        .ok_or_else(|| format!("The given interface name ({}) is not found", if_name))?;


    ifaddresses_ipv4(&mut if_addrs, if_index)?;
    ifaddresses_ipv6(&mut if_addrs, if_index)?;

    Ok(if_addrs)
}
