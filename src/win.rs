use crate::types::{
    AddrPairs, IfAddrs, ADDR_ADDR, AF_ALG, AF_INET, AF_INET6, AF_NETLINK, AF_PACKET, AF_VSOCK,
    BROADCAST_ADDR, MASK_ADDR, PEER_ADDR,
};
use std::alloc::{self, alloc, dealloc, Layout};
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Error};
use std::mem::size_of;
use std::ptr::NonNull;
use std::thread::sleep;
use std::time::Duration;
use windows::Win32::NetworkManagement::IpHelper;
use windows::Win32::NetworkManagement::IpHelper::{IP_ADAPTER_INDEX_MAP, IP_ADAPTER_INFO, MIB_IPADDRROW_XP, MIB_IPADDRTABLE};

const WIN_API_ALIGN: usize = 4;

fn get_iface_index(iface_name: &str) -> Option<u32> {
    let mut buff_size = 0;
    let mut index: Option<u32> = None;

    unsafe {
        // Get the length
        IpHelper::GetInterfaceInfo(None, &mut buff_size);
    };

    let layout = Layout::from_size_align(buff_size as usize, WIN_API_ALIGN).unwrap();
    let ptr = unsafe {
        alloc(layout) as *mut IpHelper::IP_INTERFACE_INFO
    };

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
        let s = String::from_utf16(&u16_data).unwrap();

        if s == iface_name {
            index = Some(iface[0].Index);
            break;
        }
    }

    unsafe {
        dealloc(ptr as *mut u8, layout)
    };

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

        let ptr_to_data = ((ptr as *mut u8).offset(size_of::<i32>() as isize)) as *mut [IP_ADAPTER_INDEX_MAP; 1];

        for i in 0..interface_count {
            let u16_data = (*(ptr_to_data.offset(i as isize)))[0].Name;
            let s = String::from_utf16(&u16_data)?;
            interfaces.push(s);
        }

        dealloc(ptr as *mut u8, layout);
    }

    Ok(interfaces)
}

pub fn windows_ifaddresses(if_name: &str) -> Result<IfAddrs, Box<dyn std::error::Error>> {
    let mut size = 0;

    let if_index = get_iface_index(if_name).ok_or("The given interface name is not found")?;

    unsafe {
        IpHelper::GetIpAddrTable(None, &mut size, false);
    }

    let layout = Layout::from_size_align(size as usize, WIN_API_ALIGN)?;

    let ptr = unsafe {
        let ptr = alloc(layout) as *mut MIB_IPADDRTABLE;
        IpHelper::GetIpAddrTable(Some(ptr), &mut size, false);
        ptr
    };

    unsafe {
        let mut number_of_entries = 0_usize;
        number_of_entries = (*ptr).dwNumEntries as usize;

        let raw_ptr = ((ptr as *mut u8).offset(size_of::<u32>() as isize)) as *mut u8;

        let ip_addr_table_ptr = raw_ptr as *const [MIB_IPADDRROW_XP; 1];

        for i in 0..number_of_entries {
            let row = (*(ip_addr_table_ptr.offset(i as isize)))[0];
            let row_if_index = row.dwIndex;
            if row_if_index != if_index {
                continue;
            }
        };
    }


    unsafe {
        dealloc(ptr as *mut u8, layout);
    };

    Ok(HashMap::new())
}
