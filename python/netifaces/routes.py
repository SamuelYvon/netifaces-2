import subprocess
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


def routes_parse_ip_tool(ip_tool_path: str, old_api: bool = False) -> GatewaysTable:
    ipv4_query = subprocess.run([ip_tool_path, "r"], capture_output=True)
    ipv6_query = subprocess.run([ip_tool_path, "-6", "r"], capture_output=True)

    if ipv4_query.returncode != 0 or ipv6_query.returncode != 0:
        raise RuntimeError("Cannot use the IP tool; although it is present on the system")

    ipv4_lines = ipv4_query.stdout.decode("UTF-8").splitlines()
    ipv6_lines = ipv6_query.stdout.decode("UTF-8").splitlines()

    table: GatewaysTable = defaultdict(lambda *_: [])

    for if_type, lines in [
        (InterfaceType.AF_INET, ipv4_lines),
        (InterfaceType.AF_INET6, ipv6_lines),
    ]:
        for line in lines:
            cols = line.split(" ")

            default = cols[0] == "default"

            # Only check IP* routes
            device = cols[1]
            if device != "via":
                continue

            gateway_ip_with_mask = cols[2]
            gateway_ip = gateway_ip_with_mask.split("/")[0]
            iface = cols[4]

            table[if_type.value if old_api else if_type].append(
                (gateway_ip, iface, True) if default else (gateway_ip, iface)
            )

    return dict(table)


def routes_parse_file(content: str, old_api: bool = False) -> GatewaysTable:
    lined = content.splitlines()

    if len(lined) == 0:
        raise ValueError("Cannot generate the columns header; cannot understand the routes")

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
        table[type.value if old_api else type].append(
            (gateway_as_string, iface, True) if default else (gateway_as_string, iface)
        )

    return dict(table)
