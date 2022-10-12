"""
netifaces(2), netifaces reborn
See https://github.com/SamuelYvon/netifaces-2
"""
import sys
from pathlib import Path
from typing import List, cast

from .defs import (
    AF_ALG,
    AF_APPLETALK,
    AF_ASH,
    AF_ATMPVC,
    AF_ATMSVC,
    AF_AX25,
    AF_BLUETOOTH,
    AF_BRIDGE,
    AF_CAIF,
    AF_CAN,
    AF_DEC_NET,
    AF_ECONET,
    AF_IB,
    AF_IEEE802154,
    AF_INET,
    AF_INET6,
    AF_IPX,
    AF_IRDA,
    AF_ISDN,
    AF_IUCV,
    AF_KCM,
    AF_KEY,
    AF_LLC,
    AF_LOCAL,
    AF_MAX,
    AF_MCTP,
    AF_MPLS,
    AF_NETBEUI,
    AF_NETLINK,
    AF_NETROM,
    AF_NFC,
    AF_PACKET,
    AF_PHONET,
    AF_PPPOX,
    AF_QIPCRTR,
    AF_RDS,
    AF_ROSE,
    AF_ROUTE,
    AF_RXRPC,
    AF_SECURITY,
    AF_SMC,
    AF_SNA,
    AF_TIPC,
    AF_UNIX,
    AF_UNSPEC,
    AF_VSOCK,
    AF_WANPIPE,
    AF_X25,
    AF_XDP,
    Addresses,
    DefaultGatewayEntry,
    GatewaysTable,
    InterfaceName,
    InterfaceType,
)
from .netifaces import _ifaddresses, _interfaces

__all__ = [
    "InterfaceType",
    "AF_UNSPEC",
    "AF_UNIX",
    "AF_LOCAL",
    "AF_INET",
    "AF_AX25",
    "AF_IPX",
    "AF_APPLETALK",
    "AF_NETROM",
    "AF_BRIDGE",
    "AF_ATMPVC",
    "AF_X25",
    "AF_INET6",
    "AF_ROSE",
    "AF_DEC_NET",
    "AF_NETBEUI",
    "AF_SECURITY",
    "AF_KEY",
    "AF_NETLINK",
    "AF_ROUTE",
    "AF_PACKET",
    "AF_ASH",
    "AF_ECONET",
    "AF_ATMSVC",
    "AF_RDS",
    "AF_SNA",
    "AF_IRDA",
    "AF_PPPOX",
    "AF_WANPIPE",
    "AF_LLC",
    "AF_IB",
    "AF_MPLS",
    "AF_CAN",
    "AF_TIPC",
    "AF_BLUETOOTH",
    "AF_IUCV",
    "AF_RXRPC",
    "AF_ISDN",
    "AF_PHONET",
    "AF_IEEE802154",
    "AF_CAIF",
    "AF_ALG",
    "AF_NFC",
    "AF_VSOCK",
    "AF_KCM",
    "AF_QIPCRTR",
    "AF_SMC",
    "AF_XDP",
    "AF_MCTP",
    "AF_MAX",
]


_ROUTE_FILE = Path("/proc/net/route")


def interfaces() -> List[InterfaceName]:
    """
    List the network interfaces that are available

    :return the list of network interfaces that are available
    """

    return cast(List[InterfaceName], _interfaces())


def ifaddresses(if_name: str) -> Addresses:
    """
    List the network addresses for the given interface

    :param if_name: the interface name
    :return a map of network addresses indexed by network address type.
    The values are the addresses, indexed by their roles
    """
    return cast(Addresses, _ifaddresses(if_name))


def _parse_route_file() -> GatewaysTable:
    from .routes import routes_parse

    route_content = _NIX_ROUTE_FILE.read_text()
    return routes_parse(route_content)


def gateways() -> GatewaysTable:
    """
    Get the routing table indexed by interface type

    :return a routing table
    """

    if _NIX_ROUTE_FILE.exists():
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
