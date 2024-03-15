#![allow(dead_code)]
use crate::common::InterfaceDisplay;
use crate::types;
use crate::types::{IfAddrs, ADDR_ADDR, AF_INET, AF_INET6, AF_PACKET};
use crate::NetifacesError;
use std::collections::HashMap;
use std::error::Error;
use std::net::IpAddr;

use windows::core::HSTRING;

use windows::Win32::NetworkManagement::IpHelper::GetAdapterIndex;

use get_adapters_addresses;
use get_adapters_addresses::{Adapter, PhysicalAddress};

fn ifaddresses_ip(
    adapter: &Adapter,
    if_addrs: &mut IfAddrs,
) -> Result<(), Box<dyn std::error::Error>> {
    for unicast_addr in adapter.unicast_addresses() {
        // Create entries based on type
        let entry = match unicast_addr {
            IpAddr::V4(_) => if_addrs.entry(AF_INET.into()),
            IpAddr::V6(_) => if_addrs.entry(AF_INET6.into()),
        };
        let addr_vec = entry.or_default();

        let addr_string = format!("{}", unicast_addr);

        // TODO currently get_adapters_addresses does not provide access to the
        // prefix length :(, so we cannot report the netmask or broadcast address in the data on Windows.
        // Per here: https://stackoverflow.com/a/64358443/7083698
        // this information is available in the IP_ADAPTER_UNICAST_ADDRESS_LH.OnLinkPrefixLength
        // struct field from Windows.

        addr_vec.push(HashMap::from([(ADDR_ADDR.to_string(), addr_string)]));
    }

    Ok(())
}

fn ifaddresses_mac(
    adapter: &Adapter,
    if_addrs: &mut IfAddrs,
) -> Result<(), Box<dyn std::error::Error>> {
    let phys_addr_option: Option<PhysicalAddress> = adapter.physical_address();

    // If a MAC address was found for the interface, add it to the if addrs
    match phys_addr_option {
        Some(phys_addr) => {
            let entry = if_addrs.entry(AF_PACKET as i32);
            let macs = entry.or_default();

            // Annoyingly, the get_adapters_addresses PhysicalAddress type does not implement
            // array access to the bytes, and its string conversion does not produce the exact
            // format that we need to be compatible with netifaces.  So we have to do a bit
            // of string munging.
            let mac_str_hyphens = format!("{}", phys_addr);
            let mac_str = mac_str_hyphens.replace("-", ":").to_lowercase();

            let m = HashMap::from([(ADDR_ADDR.to_string(), mac_str)]);
            macs.push(m);
        }
        None => {}
    }

    Ok(())
}
/// Find a given adapter in the list of adapters.
/// Returns error if the interface is not found
fn find_adapter<'a>(
    adapters: &'a get_adapters_addresses::AdaptersAddresses,
    if_name: &str,
) -> Result<Adapter<'a>, Box<NetifacesError>> {
    // first find the interface, matching either the description or the name
    let mut search_results: Vec<Adapter> = adapters
        .into_iter()
        .filter(|adapter| {
            adapter.description().into_string().unwrap() == if_name
                || adapter.adapter_name() == if_name
        })
        .collect();

    if search_results.len() > 1 {
        let err_msg = format!("More than a single interface with description/name '{if_name}'");
        return Err(Box::new(NetifacesError(err_msg)));
    }

    match search_results.get(0) {
        // Note: Adapter is not copyable/cloneable, so we cannot call search_results.get()
        // here.  Instead we have to use swap_remove.
        Some(_) => Ok(search_results.swap_remove(0)),
        None => {
            let err_msg = format!("Cannot find any interface with description or name '{if_name}'");
            Err(Box::new(NetifacesError(err_msg)))
        }
    }
}

/// Given an interface name, returns all the addresses associated with that interface. The result
/// is shaped loosely in a map.
pub fn windows_ifaddresses(if_name: &str) -> Result<IfAddrs, Box<dyn std::error::Error>> {
    let mut if_addrs: IfAddrs = HashMap::new();

    let adapter_addresses = get_adapters_addresses::AdaptersAddresses::try_new(
        get_adapters_addresses::Family::Unspec,
        // Turn off stuff that we don't need in this call
        // Note: would really like to use include_prefix() here, but there's no way to get the prefix
        *get_adapters_addresses::Flags::default()
            .skip_multicast()
            .skip_dns_server(),
    )?;

    let interface = find_adapter(&adapter_addresses, if_name)?;

    ifaddresses_ip(&interface, &mut if_addrs)?;
    ifaddresses_mac(&interface, &mut if_addrs)?;

    Ok(if_addrs)
}

/// List all the network interfaces available on the system.
///
/// # Params
/// - `display`: an [InterfaceDisplay] that controls what ID is returned from the call to
///              identify the interface.
pub fn windows_interfaces(display: InterfaceDisplay) -> Result<Vec<String>, Box<dyn Error>> {
    let adapter_addresses = get_adapters_addresses::AdaptersAddresses::try_new(
        get_adapters_addresses::Family::Unspec,
        *get_adapters_addresses::Flags::default()
            .skip_multicast()
            .skip_dns_server(),
    )?;

    let mut ifaces: Vec<String> = Vec::new();
    for adapter in &adapter_addresses {
        ifaces.push(match display {
            InterfaceDisplay::HumanReadable => adapter.description().into_string().unwrap(),
            InterfaceDisplay::MachineReadable => adapter.adapter_name(),
        });
    }

    Ok(ifaces)
}

/// List all the network interfaces available on the system by their indexes
pub fn windows_interfaces_by_index(
    display: InterfaceDisplay,
) -> Result<types::IfacesByIndex, Box<dyn std::error::Error>> {
    let adapter_addresses = get_adapters_addresses::AdaptersAddresses::try_new(
        get_adapters_addresses::Family::Unspec,
        *get_adapters_addresses::Flags::default()
            .skip_multicast()
            .skip_dns_server(),
    )?;

    let mut ifaces_by_index = types::IfacesByIndex::new();
    for adapter in &adapter_addresses {
        let value = match display {
            InterfaceDisplay::HumanReadable => adapter.description().into_string().unwrap(),
            InterfaceDisplay::MachineReadable => adapter.adapter_name(),
        };

        // Sadly get_adapters_addresses does not implement a getter for the
        // interface index, so we have to use the Win32 function to look up the adapter
        // index by its name.
        // I did create a feature request for this: https://gitlab.com/cratesio/get_adapters_addresses/-/issues/1
        // so hopefully someday it will be added and we can remove this code.
        let adapter_name = adapter.adapter_name();
        let mut index: u32 = 0;
        let full_adapter_name = format!("\\DEVICE\\TCPIP_{adapter_name}");
        let full_adapter_name_hstring = &HSTRING::from(&full_adapter_name);
        unsafe { GetAdapterIndex(full_adapter_name_hstring, &mut index) };

        ifaces_by_index.insert(index as usize, value);
    }

    Ok(ifaces_by_index)
}

/// Given an interface name, checks if the interface is up or not.
pub fn windows_interface_is_up(if_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let adapter_addresses = get_adapters_addresses::AdaptersAddresses::try_new(
        get_adapters_addresses::Family::Unspec,
        *get_adapters_addresses::Flags::default()
            .skip_multicast()
            .skip_dns_server()
            .skip_anycast()
            .skip_unicast(),
    )?;

    let interface = find_adapter(&adapter_addresses, if_name)?;

    match interface.operational_status() {
        get_adapters_addresses::OperStatus::Up => Ok(true),
        _ => Ok(false),
    }
}
