use crate::types::{
    AddrPairs, IfAddrs, ADDR_ADDR, AF_ALG, AF_INET, AF_INET6, AF_NETLINK, AF_PACKET, AF_VSOCK,
    BROADCAST_ADDR, MASK_ADDR, PEER_ADDR,
};
use std::alloc::{self, alloc, dealloc, Layout};
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::mem::size_of;
use std::ptr::NonNull;
use std::thread::sleep;
use std::time::Duration;
use windows::Win32::NetworkManagement::IpHelper;
use windows::Win32::NetworkManagement::IpHelper::{IP_ADAPTER_INDEX_MAP, IP_ADAPTER_INFO};

pub fn windows_interfaces() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut interfaces = vec![];
    let mut buff_size = 0;

    unsafe {
        // Get the length
        IpHelper::GetInterfaceInfo(None, &mut buff_size);

        let layout = Layout::from_size_align_unchecked(buff_size as usize, 4);
        let ptr = alloc(layout) as *mut IpHelper::IP_INTERFACE_INFO;

        // Actually get data
        IpHelper::GetInterfaceInfo(Some(ptr), &mut buff_size);

        let interface_count = (*ptr).NumAdapters as usize;

        let ptr_to_data = ((ptr as *mut u8).offset(size_of::<i32>() as isize)) as *mut [IP_ADAPTER_INDEX_MAP; 1];

        for i in 0..interface_count {
            let u16_data = *ptr_to_data.offset(i as isize)[0];
            let s = String::from_utf16(u16_data)?;
            interfaces.push(s);
        }

        dealloc(ptr as *mut u8, layout);
    }

    Ok(interfaces)
}

pub fn windows_ifaddresses(if_name: &str) -> Result<IfAddrs, Box<dyn std::error::Error>> {
    Ok(HashMap::new())
}
