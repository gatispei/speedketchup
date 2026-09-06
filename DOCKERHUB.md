**SpeedKetchup**
------------------------------

Run periodic internet [speed test](https://speedtest.net), store, display results with builtin web server.

Source, issues and prebuilt executables for bare metal:
[github.com/gatispei/speedketchup](https://github.com/gatispei/speedketchup)

### Quick start

<pre>
docker volume create speedketchup
docker run -p 8080:8080 -v speedketchup:/data gatispei/speedketchup
</pre>

Then open http://localhost:8080. Results are kept in the `/data` volume as
human readable csv.

### The image

`FROM scratch`: one layer holding the statically linked binary and an empty
`/data`, nothing else. No shell, no libc, no base image underneath, around
230k compressed.

Built for `linux/amd64`, `linux/386`, `linux/arm64` and `linux/arm/v6`.

### Configuration

Every option is an environment variable, and the command line overrides it.
Defaults are the values the image ships with.

| variable | default | what it does |
|-|-|-|
|`SPEEDKETCHUP_INTERVAL`|`10`|test interval in minutes|
|`SPEEDKETCHUP_FILE`|`/data/speedketchup-results.csv`|file to store test results in|
|`SPEEDKETCHUP_ADDRESS`|`0.0.0.0`|address to listen on|
|`SPEEDKETCHUP_PORT`|`8080`|port to listen on|
|`SPEEDKETCHUP_SERVER`||speedtest server to use, empty selects automatically|
|`SPEEDKETCHUP_DOWNLOAD_DURATION`|`10`|how long to test download speed, 0 disables|
|`SPEEDKETCHUP_UPLOAD_DURATION`|`10`|how long to test upload speed, 0 disables|
|`SPEEDKETCHUP_DOWNLOAD_CONNECTIONS`|`8`|parallel connections for download|
|`SPEEDKETCHUP_UPLOAD_CONNECTIONS`|`8`|parallel connections for upload|

<pre>
docker run -p 8080:8080 -v speedketchup:/data -e SPEEDKETCHUP_INTERVAL=30 gatispei/speedketchup
</pre>

### Features

- automatic speedtest server selection based on latency or run against specific speedtest server
- uses HTTP protocol for all operations on network
- results are stored in human readable [.csv](https://en.wikipedia.org/wiki/Comma-separated_values)
- builtin web-server to display results with [uPlot](https://github.com/leeoniya/uPlot) chart
- all web assets are included in the program binary
- statically linked with [musl](https://musl.libc.org/), not dependent on system [libc](https://en.wikipedia.org/wiki/C_standard_library)
- acts as a speedtest server itself. Not registered with official speedtest.net, but can be passed as speedtest server to supporting clients
