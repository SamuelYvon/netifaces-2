import netifaces


def test_interfaces() -> None:
    assert len(netifaces.interfaces())
