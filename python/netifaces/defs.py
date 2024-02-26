import sys
from enum import IntEnum
from typing import Dict, List, Tuple, Union

if sys.version_info >= (3, 8):
    from typing import Literal
else:
    from typing_extensions import Literal

AF_UNSPEC = 0
AF_UNIX = 1
AF_LOCAL = 1
AF_INET = 2
AF_AX25 = 3
AF_IPX = 4
AF_APPLETALK = 5
AF_NETROM = 6
AF_BRIDGE = 7
AF_ATMPVC = 8
AF_X25 = 9
AF_INET6 = 10
AF_ROSE = 11
AF_DEC_NET = 12
AF_NETBEUI = 13
AF_SECURITY = 14
AF_KEY = 15
AF_NETLINK = 16
AF_ROUTE = 16
AF_PACKET = 17
AF_ASH = 18
AF_ECONET = 19
AF_ATMSVC = 20
AF_RDS = 21
AF_SNA = 22
AF_IRDA = 23
AF_PPPOX = 24
AF_WANPIPE = 25
AF_LLC = 26
AF_IB = 27
AF_MPLS = 28
AF_CAN = 29
AF_TIPC = 30
AF_BLUETOOTH = 31
AF_IUCV = 32
AF_RXRPC = 33
AF_ISDN = 34
AF_PHONET = 35
AF_IEEE802154 = 36
AF_CAIF = 37
AF_ALG = 38
AF_NFC = 39
AF_VSOCK = 40
AF_KCM = 41
AF_QIPCRTR = 42
AF_SMC = 43
AF_XDP = 44
AF_MCTP = 45
AF_MAX = 46
AF_LINK = -1000  # Windows Link Layer as defined by netifaces(1)
AF_INTERFACE_INDEX = -1001  # Magic value for the interface index


class InterfaceType(IntEnum):
    AF_UNSPEC = AF_UNSPEC
    AF_UNIX = AF_UNIX
    AF_LOCAL = AF_LOCAL
    AF_INET = AF_INET
    AF_AX25 = AF_AX25
    AF_IPX = AF_IPX
    AF_APPLETALK = AF_APPLETALK
    AF_NETROM = AF_NETROM
    AF_BRIDGE = AF_BRIDGE
    AF_ATMPVC = AF_ATMPVC
    AF_X25 = AF_X25
    AF_INET6 = AF_INET6
    AF_ROSE = AF_ROSE
    AF_DEC_NET = AF_DEC_NET
    AF_NETBEUI = AF_NETBEUI
    AF_SECURITY = AF_SECURITY
    AF_KEY = AF_KEY
    AF_NETLINK = AF_NETLINK
    AF_ROUTE = AF_ROUTE
    AF_PACKET = AF_PACKET
    AF_ASH = AF_ASH
    AF_ECONET = AF_ECONET
    AF_ATMSVC = AF_ATMSVC
    AF_RDS = AF_RDS
    AF_SNA = AF_SNA
    AF_IRDA = AF_IRDA
    AF_PPPOX = AF_PPPOX
    AF_WANPIPE = AF_WANPIPE
    AF_LLC = AF_LLC
    AF_IB = AF_IB
    AF_MPLS = AF_MPLS
    AF_CAN = AF_CAN
    AF_TIPC = AF_TIPC
    AF_BLUETOOTH = AF_BLUETOOTH
    AF_IUCV = AF_IUCV
    AF_RXRPC = AF_RXRPC
    AF_ISDN = AF_ISDN
    AF_PHONET = AF_PHONET
    AF_IEEE802154 = AF_IEEE802154
    AF_CAIF = AF_CAIF
    AF_ALG = AF_ALG
    AF_NFC = AF_NFC
    AF_VSOCK = AF_VSOCK
    AF_KCM = AF_KCM
    AF_QIPCRTR = AF_QIPCRTR
    AF_SMC = AF_SMC
    AF_XDP = AF_XDP
    AF_MCTP = AF_MCTP
    AF_MAX = AF_MAX
    AF_LINK = AF_LINK


InterfaceName = str
AddressType = Union[
    Literal["addr"],
    Literal["peer"],
    Literal["mask"],
    Literal["broadcast"],
]
Address = str

Addresses = Dict[InterfaceType, List[Dict[AddressType, Address]]]

GatewayEntry = Union[Tuple[str, str], Tuple[str, str, bool]]
GatewaysTable = Dict[Union[InterfaceType, int], List[GatewayEntry]]

DefaultGatewayEntry = Dict[Union[InterfaceType, int], Tuple[str, str]]
