**SpeedKetchup**
------------------------------

Run periodic internet [speed test](https://speedtest.net), store, display results with builtin web server.

---
### Introduction

This aims to be small, efficient all-in-one program (statically linked, includes web assets) written in [rust](https://rust-lang.org).
The intent is to allow monitoring of your internet connection as easy as possible.
You can use speedketchup for one-off test or keep it running continuously to spot any changes in your internet connection.

Prebuilt executables for some platforms:
| platform | executable | [upx](https://upx.github.io/)-compressed executable |
|-|-|-|
|linux x86_64|[395k](https://github.com/gatispei/speedketchup-files/blob/main/bin/speedketchup-x64)|[173k](https://github.com/gatispei/speedketchup-files/blob/main/bin/speedketchup-x64-upx)|
|linux i686|[382k](https://github.com/gatispei/speedketchup-files/blob/main/bin/speedketchup-i686)|[181k](https://github.com/gatispei/speedketchup-files/blob/main/bin/speedketchup-i686-upx)|
|linux aarch64|[353k](https://github.com/gatispei/speedketchup-files/blob/main/bin/speedketchup-aarch64)|[201k](https://github.com/gatispei/speedketchup-files/blob/main/bin/speedketchup-aarch64-upx)|
|linux arm|[398k](https://github.com/gatispei/speedketchup-files/blob/main/bin/speedketchup-arm)|[175k](https://github.com/gatispei/speedketchup-files/blob/main/bin/speedketchup-arm-upx)|
|linux mips|[1035k](https://github.com/gatispei/speedketchup-files/blob/main/bin/speedketchup-mips)|[360k](https://github.com/gatispei/speedketchup-files/blob/main/bin/speedketchup-mips-upx)|
|linux mipsel|[1035k](https://github.com/gatispei/speedketchup-files/blob/main/bin/speedketchup-mipsel)|[367k](https://github.com/gatispei/speedketchup-files/blob/main/bin/speedketchup-mipsel-upx)|
|windows x86_64|[392k](https://github.com/gatispei/speedketchup-files/blob/main/bin/speedketchup-x64.exe)|[165k](https://github.com/gatispei/speedketchup-files/blob/main/bin/speedketchup-x64-upx.exe)|
|macos universal (x86_64 + aarch64)|[698k](https://github.com/gatispei/speedketchup-files/blob/main/bin/speedketchup-macos)|

### Docker Container

Multi-architecture container images available on DockerHub:

```bash
# Run with default settings (web UI on port 8080)
docker run -p 8080:8080 antonsmt/speedketchup

# Run with custom interval and bind to all interfaces
docker run -p 8080:8080 antonsmt/speedketchup -i 5 -a 0.0.0.0

# Run with persistent data storage
docker run -p 8080:8080 -v $(pwd)/data:/data antonsmt/speedketchup -f /data/results.csv
```

To run on MikroTik devices - add veth interface and setup network as per MikroTik [documentation](https://help.mikrotik.com/docs/spaces/ROS/pages/84901929/Container)

#### Environment Variables

Environment variables can be used instead of command line options (CLI options will override env vars):

| Environment Variable | CLI Option | Default | Description |
|---------------------|------------|---------|-------------|
| `test_interval` | `-i/--interval` | `10` | Test interval in minutes |
| `store_filename` | `-f/--file` | `speedketchup-results.csv` | File to store results |
| `listen_address` | `-a/--address` | `127.0.0.1` | Listen address (use `0.0.0.0` for all) |
| `listen_port` | `-p/--port` | `8080` | Listen port |
| `server_host` | `-s/--server` | auto-select | Speedtest server host[:port] |
| `download_duration` | `-dd/--download-duration` | `10` | Download test duration (seconds) |
| `upload_duration` | `-ud/--upload-duration` | `10` | Upload test duration (seconds) |
| `download_connections` | `-dc/--download-connections` | `8` | Download parallel connections |
| `upload_connections` | `-uc/--upload-connections` | `8` | Upload parallel connections |

```bash
# Example with environment variables
docker run -p 8080:8080 \
  -e test_interval=5 \
  -e listen_address=0.0.0.0 \
  -e store_filename=/data/results.csv \
  -v $(pwd)/data:/data \
  antonsmt/speedketchup
```

For MikroTik RouterOS containers:
```RouterOS
/container envs add list=speedketchup key=test_interval value=5
/container envs add list=speedketchup key=listen_address value=0.0.0.0
/container envs add list=speedketchup key=store_filename value=results.csv
```
Supported container architectures: `amd64`, `arm64`, `arm/v7`


### Builtin web server
![speedketchup webserver](https://github.com/gatispei/speedketchup-files/blob/main/img/ketchup.png "SpeedKetchup webserver")

### Features

- automatic speedtest server selection based on latency or run against specific speedtest server
- uses HTTP protocol for all operations on network
- tiny list of rust dependencies: [std](https://doc.rust-lang.org/std/index.html), [libc](https://crates.io/crates/libc), [roxmltree](https://crates.io/crates/roxmltree) + [xmlparser](https://crates.io/crates/xmlparser)
- results are stored in human readable [.csv](https://en.wikipedia.org/wiki/Comma-separated_values), other storage backends incoming
- builtin web-server to display results with [uPlot](https://github.com/leeoniya/uPlot) chart
- all web assets are included in the program binary
- linux version statically linked with [musl](https://musl.libc.org/) - not dependent on system [libc](https://en.wikipedia.org/wiki/C_standard_library), essentially a single binary container
- acts as a speedtest server itself. Not registered with official speedtest.net, but can be passed as speedtest server to supporting clients

### Alternatives

- https://www.speedtest.net/apps/cli - official speedtest cli tool, binary only
- https://github.com/sivel/speedtest-cli - speedtest cli tool in python
- https://github.com/nelsonjchen/speedtest-rs - speedtest cli tool in rust
- https://github.com/YuLinLee/speedtest - speedtest cli tool in c
- https://github.com/barrycarey/Speedmon - periodic speed test runner in python with various storage backends
