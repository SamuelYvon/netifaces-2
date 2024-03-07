#!/usr/bin/env python3

"""
Rudimentary replica of the `ip addr` tool, implemented using netifaces2.

This can be used when developing netifaces as a way to sanity check its output by comparing
to the interfaces listed by the native tool.
"""

import ipaddress
from typing import Dict

import netifaces
from netifaces import InterfaceType


def netmask_string_to_prefix_len_v6(netmask: str) -> int:
    """
    Sadly the ipaddress library does not support converting an IPv6 subnet mask into a prefix length
    So we have to do this one manually.
    """

    # From https://stackoverflow.com/a/33533007/7083698
    bit_count = [
        0,
        0x8000,
        0xC000,
        0xE000,
        0xF000,
        0xF800,
        0xFC00,
        0xFE00,
        0xFF00,
        0xFF80,
        0xFFC0,
        0xFFE0,
        0xFFF0,
        0xFFF8,
        0xFFFC,
        0xFFFE,
        0xFFFF,
    ]

    count = 0
    for w in netmask.split(":"):
        if not w or int(w, 16) == 0:
            break
        count += bit_count.index(int(w, 16))

    return count


def print_ip_addr_entry(
    ip_addr_entry: Dict[netifaces.defs.AddressType, netifaces.defs.Address], iface_type: InterfaceType
) -> None:
    """
    Print a single IP address (v4 or v6) to the console.
    """

    print(f"    inet{'6' if iface_type == InterfaceType.AF_INET6 else ''} {ip_addr_entry['addr']}", end="")

    # Print peer address if it exists
    if "peer" in ip_addr_entry:
        print(f" peer {ip_addr_entry['peer']}", end="")

    # If the netmask is available, compute the prefix.
    # Per here: https://serverfault.com/questions/998915/netmask-for-point-to-point-ip-address
    # the prefix length is traditionally printed on the peer address when it exists.
    if "mask" in ip_addr_entry:
        if iface_type == InterfaceType.AF_INET:
            # We can use ipaddress's network class to get the integer prefix from an address and subnet
            iface_with_mask = ipaddress.IPv4Network((ip_addr_entry["addr"] + "/" + ip_addr_entry["mask"]), strict=False)
            prefix_len = iface_with_mask.prefixlen
        else:
            prefix_len = netmask_string_to_prefix_len_v6(ip_addr_entry["mask"])

        print(f"/{prefix_len}", end="")

    # Print broadcast address if it exists
    if "broadcast" in ip_addr_entry:
        print(f" brd {ip_addr_entry['broadcast']}", end="")

    print("")


def print_ifaces() -> None:
    # First, enumerate all the interfaces on the machine
    sorted_ifaces = dict(sorted(netifaces.interfaces_by_index(netifaces.InterfaceDisplay.HumanReadable).items()))

    for index, name in sorted_ifaces.items():
        print(f"{index}: {name}")

        # Get addresses of this interface at each level
        addrs = netifaces.ifaddresses(name)

        # Print mac addresses
        if InterfaceType.AF_PACKET in addrs:
            for mac_addr_entry in addrs[InterfaceType.AF_PACKET]:
                print(f"    link/ether {mac_addr_entry['addr']}", end="")
                if "broadcast" in mac_addr_entry:
                    print(f" brd {mac_addr_entry['broadcast']}", end="")
                print("")

        # Print IPv4 addresses
        if InterfaceType.AF_INET in addrs:
            for ip_addr_entry in addrs[InterfaceType.AF_INET]:
                print_ip_addr_entry(ip_addr_entry, InterfaceType.AF_INET)

        # Print IPv6 addresses
        if InterfaceType.AF_INET6 in addrs:
            for ip_addr_entry in addrs[InterfaceType.AF_INET6]:
                print_ip_addr_entry(ip_addr_entry, InterfaceType.AF_INET6)


if __name__ == "__main__":
    print_ifaces()
