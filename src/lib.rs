#![allow(non_snake_case)]

extern crate core;

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use std::fmt::Write;

mod types;

#[cfg(not(target_family = "windows"))]
mod linux;

#[cfg(not(target_family = "windows"))]
use linux::{
    posix_ifaddresses as ifaddresses, posix_interface_is_up as interface_is_up,
    posix_interfaces as interfaces, posix_interfaces_by_index as interfaces_by_index,
};

mod common;
#[cfg(target_family = "windows")]
mod win;

use crate::common::InterfaceDisplay;
#[cfg(target_family = "windows")]
use win::{
    windows_ifaddresses as ifaddresses, windows_interfaces as interfaces,
    windows_interfaces_by_index as interfaces_by_index,
};

/// Given an u32 in little endian, return the String representation
/// of it into the colloquial IPV4 string format
pub fn ip_to_string(ip: u32) -> String {
    let mut s = String::new();

    for i in 0_u8..4 {
        let grp_idx = 4 - i - 1;
        let group_val = (ip >> (grp_idx * 8)) & 0xFF;

        let sep = if i < 3 { "." } else { "" };

        let formatted = format!("{group_val}{sep}");
        s.push_str(&formatted);
    }

    s
}

/// Given the bytes that makes up a mac address, return the String
/// representation as it would be expected in the colloquial form.
pub fn mac_to_string(mac: &Vec<u8>) -> String {
    let mut s = String::new();

    for i in 0..mac.len() {
        write!(&mut s, "{:02X?}", mac[i]).unwrap();
        if i + 1 < mac.len() {
            s.push(':');
        }
    }

    s
}

#[pyfunction]
pub fn _ip_to_string(ip: u32) -> String {
    ip_to_string(ip)
}

#[pyfunction]
fn _interfaces(interface_display: i32) -> PyResult<Vec<String>> {
    let interface_display = InterfaceDisplay::try_from(interface_display)?;
    let maybe_ifs = interfaces(interface_display);

    maybe_ifs.map_err(|e| {
        let str_message = e.to_string();
        PyErr::new::<PyRuntimeError, _>(str_message)
    })
}

#[pyfunction]
fn _interfaces_by_index(interface_display: i32) -> PyResult<types::IfacesByIndex> {
    let interface_display = InterfaceDisplay::try_from(interface_display)?;
    let maybe_ifs = interfaces_by_index(interface_display);

    maybe_ifs.map_err(|e| {
        let str_message = e.to_string();
        PyErr::new::<PyRuntimeError, _>(str_message)
    })
}

#[pyfunction]
fn _ifaddresses(if_name: &str) -> PyResult<types::IfAddrs> {
    let maybe_ifaddrs = ifaddresses(if_name);

    maybe_ifaddrs.map_err(|e| {
        let str_message = e.to_string();
        PyErr::new::<PyRuntimeError, _>(str_message)
    })
}

#[pyfunction]
fn _interface_is_up(if_name: &str) -> PyResult<bool> {
    let maybe_if_status = interface_is_up(if_name);

    maybe_if_status.map_err(|e| {
        let str_message = e.to_string();
        PyErr::new::<PyRuntimeError, _>(str_message)
    })
}

#[pymodule]
fn netifaces(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(_interfaces, m)?)?;
    m.add_function(wrap_pyfunction!(_interfaces_by_index, m)?)?;
    m.add_function(wrap_pyfunction!(_ifaddresses, m)?)?;
    m.add_function(wrap_pyfunction!(_ip_to_string, m)?)?;
    m.add_function(wrap_pyfunction!(_interface_is_up, m)?)?;
    Ok(())
}
