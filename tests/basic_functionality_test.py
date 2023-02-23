import netifaces


def test_interfaces() -> None:
    assert len(netifaces.interfaces())


def test_has_ipv4_or_ipv6() -> None:
    has_any_ip = False

    for interface in netifaces.interfaces():
        address_table = netifaces.ifaddresses(interface)

        has_any_ip |= netifaces.AF_INET in address_table
        has_any_ip |= netifaces.AF_INET6 in address_table

    assert has_any_ip, "Test failure; no AF_INET address of any kind found"
