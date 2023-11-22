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
|linux x86_64|[641k](https://github.com/gatispei/speedketchup-files/blob/main/bin/speedketchup-x64)|[261k](https://github.com/gatispei/speedketchup-files/blob/main/bin/speedketchup-x64-upx)|
|linux i686|[627k](https://github.com/gatispei/speedketchup-files/blob/main/bin/speedketchup-i686)|[276k](https://github.com/gatispei/speedketchup-files/blob/main/bin/speedketchup-i686-upx)|
|linux aarch64|[550k](https://github.com/gatispei/speedketchup-files/blob/main/bin/speedketchup-aarch64)|[250k](https://github.com/gatispei/speedketchup-files/blob/main/bin/speedketchup-aarch64-upx)|
|linux arm|[603k](https://github.com/gatispei/speedketchup-files/blob/main/bin/speedketchup-arm)|[251k](https://github.com/gatispei/speedketchup-files/blob/main/bin/speedketchup-arm-upx)|
|windows x86_64|[541k](https://github.com/gatispei/speedketchup-files/blob/main/bin/speedketchup-x64.exe)|[220k](https://github.com/gatispei/speedketchup-files/blob/main/bin/speedketchup-x64-upx.exe)|
|macos universal (x86_64 + aarch64)|[963k](https://github.com/gatispei/speedketchup-files/blob/main/bin/speedketchup-macos)|

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

### Command Line

<pre>
usage: speedketchup [options]
	-h|--help: print this
	-v|--version: print version
	-i|--interval &ltminutes&gt: test interval in minutes, 10 by default
	-f|--file &ltfilename&gt: file to store test results in, speedketchup-results.csv by default
	-a|--address &ltlocal_addr&gt: address to listen on for incoming connections, 127.0.0.1 by default
		local_addr: use 0.0.0.0 to accept connections on all addresses
	-p|--port &ltport&gt: port to listen on for incoming connections, 8080 by default
	-s|--server &ltserver_host[:server_port]&gt: speedtest server to use, avoids automatic server selection if specified
		server_host: domain_name|ipv4_addr|ipv6_addr
		server_port: port number, 8080 by default
	-dd|--download-duration &ltseconds&gt: how long to test download speed, 0 disables test, 10 seconds by default
	-ud|--upload-duration &ltseconds&gt: how long to test upload speed, 0 disables test, 10 seconds by default
	-dc|--download-connections &ltnumber&gt: how many parallel connections to make for download, 8 connections by default
	-uc|--uplaod-connections &ltnumber&gt: how many parallel connections to make for upload, 8 connections by default
</pre>

### Alternatives

- https://www.speedtest.net/apps/cli - official speedtest cli tool, binary only
- https://github.com/sivel/speedtest-cli - speedtest cli tool in python
- https://github.com/nelsonjchen/speedtest-rs - speedtest cli tool in rust
- https://github.com/YuLinLee/speedtest - speedtest cli tool in c
- https://github.com/barrycarey/Speedmon - periodic speed test runner in python with various storage backends
