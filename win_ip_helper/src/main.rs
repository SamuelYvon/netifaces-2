#![allow(dead_code)]

use std::mem::size_of;
use std::process::exit;
use windows::Win32::Foundation::{ERROR_BUFFER_OVERFLOW, WIN32_ERROR};
use windows::Win32::NetworkManagement::IpHelper;
use windows::Win32::NetworkManagement::IpHelper::IP_ADAPTER_INFO;

fn adapter_info_size_required() -> Result<u32, WIN32_ERROR> {
    let mut size: u32 = 0;
    unsafe {
        let ret = WIN32_ERROR(IpHelper::GetAdaptersInfo(None, &mut size));
        // Do not check for other return values; as per MS' docs, this will return an error code.
        if ret != ERROR_BUFFER_OVERFLOW {
            Err(ret)
        } else {
            Ok(size)
        }
    }
}

fn main() {
    let size = match adapter_info_size_required() {
        Ok(size) => size,
        Err(err) => {
            println!("Failed to acquire the size. Win error code: {0}", err.0);
            exit(1);
        }
    };
    let number_of_entries = (size as usize) / size_of::<IP_ADAPTER_INFO>();
    println!(
        "To get the adapter's table, {size} bytes are required, or {number_of_entries} structures."
    );
}
