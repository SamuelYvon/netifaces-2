"""
netifaces(2), netifaces reborn
See https://github.com/SamuelYvon/netifaces-2
"""
import logging
import subprocess
import sys
from pathlib import Path
from typing import List, Optional, cast

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
    AF_LINK,
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
    "AF_LINK",
]

logging.basicConfig(level=logging.ERROR)
logger = logging.getLogger(__name__)
_platform = sys.platform


_NIX_ROUTE_FILE = Path("/proc/net/route")


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


def _ip_tool_path() -> Optional[str]:
    is_linux = _platform == "linux" or _platform == "linux32"
    if not is_linux:
        return None

    which_ip_result = subprocess.run(["which", "ip"], capture_output=True)

    if which_ip_result.returncode == 0:
        ip = which_ip_result.stdout.decode("UTF-8").strip()
    else:
        ip = None

    return ip


def gateways(old_api: bool = False) -> GatewaysTable:
    """
    Get the routing table indexed by interface type

    :return a routing table
    """

    ip_tool_path = _ip_tool_path()

    if ip_tool_path:
        from .routes import routes_parse_ip_tool

        logging.debug("Using ip tool")
        return routes_parse_ip_tool(ip_tool_path, old_api=old_api)
    elif _NIX_ROUTE_FILE.exists():
        from .routes import routes_parse_file

        logging.debug("Using route file")
        return routes_parse_file(_NIX_ROUTE_FILE.read_text(), old_api=old_api)
    else:
        raise NotImplementedError("No implementation for `gateways()` yet")


def default_gateway(old_api: bool = False) -> DefaultGatewayEntry:
    """
    Get the default gateway for each interface type

    :return the default gateway indexed by each interface type
    """

    default_table: DefaultGatewayEntry = {}

    for if_type, list_of_tuples in gateways(old_api=old_api).items():
        for gateway_ip, if_name, *rest in list_of_tuples:
            if len(rest) > 0 and rest[0]:
                default_table.update({if_type: (gateway_ip, if_name)})

    return default_table
