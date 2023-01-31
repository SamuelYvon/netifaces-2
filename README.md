# netifaces (2)

## 1. What is this?

The original [netifaces](https://github.com/al45tair/netifaces) was abandonned by it's maintainer,
leaving us without the option to get network addresses of any kind in Python. Unfortunately, the
original sources are more akin to arcane magic, so picking where it's been left off is a difficult
task.

I decided to rewrite `netifaces`, keeping the **almost exact same API** but adding the following:

- Support for future python versions
- Type annotations
- Maybe a more "queriable" API in the future

This project aims to be a drop-in replacement for those project who use `netifaces`, but I do not
guarantee anything.


### 1.1 What is not working **right-now**

- The `gateways` API is only working if your system has a `/proc/net/route` file
- The `windows` API is non-functional

The following section is taken from the origin netifaces:

## 2. Usage

For now the API is the same as the original `netifaces`, so please refer to [it](https://github.com/al45tair/netifaces).

Install:
```shell
pip install netifaces2
```

Import:
```python
>>> import netifaces
>>> netifaces.interfaces()  
    ...
>>> netifaces.ifaddresses('en0')
    ...
>>> netifacs.gateways()
    ...
```

## 3. API differences between this and al45tair/netifaces

### `gateways`

The `gateways` function does not support indexing through the `default` special key. This is because it makes a
sane typing definition difficult to write and be understandable. Instead, if you want the same functionality,
the following is exposed:

```python
>>> netifaces.default_gateway()
    ...
```

The result will be the default gateway for each interface type. The result may be an empty dict if no default
route is set.

## 4. Platform support

### Wheels
Building Linux, Windows and macOS cp37-abi3 wheels (requires Python 3.7 and newer)  
Install using pip:  
`python -m pip install netifaces2`

#### Linux  
Targeting manylinux_2_28 (requires pip>=20.3)  
Building also cp36m-manylinux2014 wheels for distros using Python 3.6

## 5. License

This software is distributed under a MIT license.
