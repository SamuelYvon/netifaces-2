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

pub fn windows_interfaces() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut interfaces = vec![];

    let adapter_sz = size_of::<IpHelper::IP_ADAPTER_INFO>();
    print!("Adapter size: {}", adapter_sz);
    sleep(Duration::from_secs(5));

    unsafe {
        let mut number = 1000;
        // IpHelper::GetNumberOfInterfaces(&mut number);

        print!("Before layout");

        let layout = Layout::from_size_align_unchecked(adapter_sz * 10000usize, 1);

        let ptr = alloc(layout) as *mut IpHelper::IP_ADAPTER_INFO;
        print!("After layout");

        let mut adapter_count = 10000;
        IpHelper::GetAdaptersInfo(Some(ptr), &mut adapter_count);

        for i in 0..adapter_count {
            let adapter_ptr: *mut IpHelper::IP_ADAPTER_INFO = ptr.offset(i.try_into().unwrap());
            let mut s = String::new();

            // *adapter_ptr.Description
            for c in (*adapter_ptr).Description {
                if c.0 == b'\0' {
                    break;
                }
                s.push(c.0 as char);
            }

            interfaces.push(s);
        }

        dealloc(ptr as *mut u8, layout);
    }
    Ok(interfaces)
}

pub fn windows_ifaddresses(if_name: &str) -> Result<IfAddrs, Box<dyn std::error::Error>> {
    Ok(HashMap::new())
}
