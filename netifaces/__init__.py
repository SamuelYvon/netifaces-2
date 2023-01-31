"""
netifaces(2), netifaces reborn
See https://github.com/SamuelYvon/netifaces-2
"""

from pathlib import Path
from typing import List

from .defs import Addresses, DefaultGatewayEntry, GatewaysTable, InterfaceName
from .netifaces import _ifaddresses, _interfaces

_ROUTE_FILE = Path("/proc/net/route")


def interfaces() -> List[InterfaceName]:
    """
    List the network interfaces that are available

    :return the list of network interfaces that are available
    """

    return _interfaces()


def ifaddresses(if_name: str) -> Addresses:
    """
    List the network addresses for the given interface

    :param if_name: the interface name
    :return a map of network addresses indexed by network address type.
    The values are the addresses, indexed by their roles
    """
    return _ifaddresses(if_name)


def _parse_route_file() -> GatewaysTable:
    from .routes import routes_parse

    route_content = _ROUTE_FILE.read_text()
    return routes_parse(route_content)


def gateways() -> GatewaysTable:
    """
    Get the routing table indexed by interface type

    :return a routing table
    """

    if _ROUTE_FILE.exists():
        return _parse_route_file()
    else:
        raise NotImplementedError("No implementation for `gateways()` yet")

    return {}


def default_gateway() -> DefaultGatewayEntry:
    """
    Get the default gateway for each interface type

    :return the default gateway indexed by each interface type
    """

    default_table: DefaultGatewayEntry = {}

    for if_type, list_of_tuples in gateways().items():
        for gateway_ip, if_name, *rest in list_of_tuples:
            if len(rest) > 0 and rest[0]:
                default_table.update({if_type: (gateway_ip, if_name)})

    return default_table
