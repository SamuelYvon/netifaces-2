import platform
import re

import netifaces
import pytest


def test_interfaces_returns_something() -> None:
    assert len(netifaces.interfaces())


def test_interfaces_by_index_returns_same_interface_list() -> None:
    # Interface indices are difficult to verify, but we can at least check that we get
    # the same set of interfaces in both these functions
    assert set(netifaces.interfaces_by_index().values()) == set(netifaces.interfaces())


def test_has_ipv4_or_ipv6() -> None:
    has_any_ip = False

    for interface in netifaces.interfaces():
        address_table = netifaces.ifaddresses(interface)

        has_any_ip |= netifaces.AF_INET in address_table
        has_any_ip |= netifaces.AF_INET6 in address_table

        if has_any_ip:
            break

    assert has_any_ip, "Test failure; no AF_INET address of any kind found"


def test_has_link_layer() -> None:
    has_any_link = False

    for interface in netifaces.interfaces():
        address_table = netifaces.ifaddresses(interface)

        has_any_link |= netifaces.AF_PACKET in address_table
        has_any_link |= netifaces.AF_LINK in address_table

        if has_any_link:
            break

    assert has_any_link, "Test failure; no AF_PACKET address of any kind found"


@pytest.mark.skipif(platform.system() != "Windows", reason="Windows only")  # type: ignore[misc]
def test_interface_display_formats_windows() -> None:
    """
    Check that the InterfaceDisplay argument can be used to select between a UUID
    and a human readable name
    """

    uuid_regex = r"{[-A-F0-9]+}"

    # The machine readable interface should look like a UUID string
    machine_readable_iface0 = netifaces.interfaces(netifaces.InterfaceDisplay.MachineReadable)[0]
    print(f"Machine readable name of interface 0 is: {machine_readable_iface0}")
    assert re.fullmatch(uuid_regex, machine_readable_iface0) is not None

    # The human readable interface should NOT look like a UUID
    human_readable_iface0 = netifaces.interfaces(netifaces.InterfaceDisplay.HumanReadable)[0]
    print(f"Human readable name of interface 0 is: {human_readable_iface0}")
    assert re.fullmatch(uuid_regex, human_readable_iface0) is None


@pytest.mark.skipif(platform.system() == "Windows", reason="Does not pass yet on Windows")  # type: ignore[misc]
def test_loopback_addr_is_returned() -> None:
    """
    Test that the loopback address is returned in the lists of addresses
    (regression test for a bug)
    """

    loopback_ipv4_found = False
    loopback_ipv6_found = False

    for interface in netifaces.interfaces():
        address_table = netifaces.ifaddresses(interface)

        if netifaces.AF_INET in address_table:
            for ipv4_settings in address_table[netifaces.InterfaceType.AF_INET]:
                print(f"Loopback test: Considering iface {interface} IPv4 address " f"{ipv4_settings['addr']}")
                if ipv4_settings["addr"] == "127.0.0.1":
                    print("Loopback IPv4 found!")
                    loopback_ipv4_found = True

        if netifaces.AF_INET6 in address_table:
            for ipv6_settings in address_table[netifaces.InterfaceType.AF_INET6]:
                print(f"Loopback test: Considering iface {interface} IPv6 address " f"{ipv6_settings['addr']}")
                if ipv6_settings["addr"] == "::1":
                    print("Loopback IPv6 found!")
                    loopback_ipv6_found = True

    assert loopback_ipv4_found
    assert loopback_ipv6_found


def test_all_ifaces_have_ipv4() -> None:
    """
    Test that all interfaces which return IPv4 addresses have a "real" IPv4 address
    and not 0.0.0.0.
    (regression test for a bug)
    """

    for interface in netifaces.interfaces():
        address_table = netifaces.ifaddresses(interface)
        if netifaces.AF_INET in address_table:
            for ipv4_settings in address_table[netifaces.InterfaceType.AF_INET]:
                assert ipv4_settings["addr"] != "0.0.0.0"
