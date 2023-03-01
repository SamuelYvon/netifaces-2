import netifaces


def test_interfaces() -> None:
    assert len(netifaces.interfaces())


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
