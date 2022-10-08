from collections import defaultdict
from typing import List

from .defs import GatewaysTable, InterfaceType
from .netifaces import _ip_to_string


def _safe_split(line: str) -> List[str]:
    simplified = line.replace("\t", " ")
    splat = simplified.split(" ")
    return [x for x in splat if len(x) > 0]


IFACE = "Iface"
DESTINATION = "Destination"
GATEWAY = "Gateway"

NIL_ADDR = "0" * 8


def routes_parse(content: str) -> GatewaysTable:
    lined = content.splitlines()

    if len(lined) == 0:
        raise ValueError(
            "Cannot generate the columns header; cannot understand the routes"
        )

    columns = _safe_split(lined[0])
    entries = [_safe_split(line) for line in lined[1:]]

    gw_column = columns.index(GATEWAY)
    destination_column = columns.index(DESTINATION)
    iface_column = columns.index(IFACE)

    table: GatewaysTable = defaultdict(lambda *_: [])

    for entry in entries:
        type = InterfaceType.AF_INET

        gateway = entry[gw_column]

        if gateway == NIL_ADDR:
            continue

        destination = entry[destination_column]
        iface = entry[iface_column]

        default = destination == NIL_ADDR

        gateway_as_string = ".".join(_ip_to_string(int(gateway, 16)).split(".")[::-1])
        table[type].append(
            (gateway_as_string, iface, True) if default else (gateway_as_string, iface)
        )

    return dict(table)
