import netifaces

if __name__ == "__main__":
    assert len(netifaces.interfaces())
