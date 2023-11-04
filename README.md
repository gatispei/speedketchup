**SpeedKetchup**
------------------------------

Run periodic internet [speed test](https://speedtest.net), store, display results with builtin web server.

---
### Introduction

This aims to be small, efficient all-in-one program (statically linked, includes web assets) written in [rust](https://rust-lang.org).
The intent is to allow monitoring of your internet connection as easy as possible.
You can use speedketchup for one-off test or keep it running continuously to spot any changes in your internet connection.

Prebuilt binaries for some platforms:
- [linux x86_64 binary](bin/speedketchup) (750k)
- [linux arm binary](bin/speedketchup-arm) (650k)
- [windows x86_64 binary](bin/speedketchup.exe) (545k)
- [macos universal binary](bin/speedketchup-macos) (1090k)

---
### Builtin web server
![speedketchup webserver](img/ketchup.png "SpeedKetchup webserver")

---
### Features

- automatic speedtest server selection based on latency or run against specific speedtest server
- uses HTTP protocol for all operations on network
- tiny list of rust dependencies: [std](https://doc.rust-lang.org/std/index.html), [libc](https://crates.io/crates/libc), [roxmltree](https://crates.io/crates/roxmltree) + [xmlparser](https://crates.io/crates/xmlparser)
- results are stored in human readable [.csv](https://en.wikipedia.org/wiki/Comma-separated_values), other storage backends incoming
- builtin web-server to display results with [uPlot](https://github.com/leeoniya/uPlot) chart
- all web assets are included in the program binary
- linux version statically linked with [musl](https://musl.libc.org/) - not dependent on system [libc](https://en.wikipedia.org/wiki/C_standard_library), essentially a single binary container

---
### Alternatives

- https://www.speedtest.net/apps/cli - official speedtest cli tool, binary only
- https://github.com/sivel/speedtest-cli - speedtest cli tool in python
- https://github.com/nelsonjchen/speedtest-rs - speedtest cli tool in rust
- https://github.com/YuLinLee/speedtest - speedtest cli tool in c
- https://github.com/barrycarey/Speedmon - periodic speed test runner in python with various storage backends
