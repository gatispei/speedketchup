**SpeedKetchup**
------------------------------

Know what your internet really delivers.

Run periodic internet [speed test](https://speedtest.net), store, display results with builtin web server.

---
### Introduction

This aims to be small, efficient all-in-one program (statically linked, includes web assets) written in [rust](https://rust-lang.org).
The intent is to allow monitoring of your internet connection as easy as possible.
You can use speedketchup for one-off test or keep it running continuously to spot any changes in your internet connection.

**What it measures:** the internet connection of the machine speedketchup runs on.
Every test goes from that machine to a [speedtest.net](https://speedtest.net) server on the internet,
the same servers the speedtest.net website and apps use. The web page only shows the
results - opening it, or starting a test from it, does not measure the link between your
browser and speedketchup, so Wi-Fi between them does not count. To measure the internet
connection rather than a Wi-Fi link, run it on a machine wired to the router, or on the
router or NAS itself.

Prebuilt executables for some platforms:

| platform | executable | [upx](https://upx.github.io/)-compressed executable | app store |
|-|-|-|-|
|linux x86_64|[444k](https://raw.githubusercontent.com/gatispei/speedketchup-files/main/bin/speedketchup-x64)|[195k](https://raw.githubusercontent.com/gatispei/speedketchup-files/main/bin/speedketchup-x64-upx)|
|linux i686|[435k](https://raw.githubusercontent.com/gatispei/speedketchup-files/main/bin/speedketchup-i686)|[199k](https://raw.githubusercontent.com/gatispei/speedketchup-files/main/bin/speedketchup-i686-upx)|
|linux aarch64|[460k](https://raw.githubusercontent.com/gatispei/speedketchup-files/main/bin/speedketchup-aarch64)|[187k](https://raw.githubusercontent.com/gatispei/speedketchup-files/main/bin/speedketchup-aarch64-upx)|
|linux arm|[439k](https://raw.githubusercontent.com/gatispei/speedketchup-files/main/bin/speedketchup-arm)|[195k](https://raw.githubusercontent.com/gatispei/speedketchup-files/main/bin/speedketchup-arm-upx)|
|linux mips|[592k](https://raw.githubusercontent.com/gatispei/speedketchup-files/main/bin/speedketchup-mips)|[222k](https://raw.githubusercontent.com/gatispei/speedketchup-files/main/bin/speedketchup-mips-upx)|
|linux mipsel|[592k](https://raw.githubusercontent.com/gatispei/speedketchup-files/main/bin/speedketchup-mipsel)|[225k](https://raw.githubusercontent.com/gatispei/speedketchup-files/main/bin/speedketchup-mipsel-upx)|
|windows x86_64|[448k](https://raw.githubusercontent.com/gatispei/speedketchup-files/main/bin/speedketchup-x64.exe)|[191k](https://raw.githubusercontent.com/gatispei/speedketchup-files/main/bin/speedketchup-x64-upx.exe)|[Microsoft Store](https://apps.microsoft.com/detail/9P0J5WV54WXH)|
|macos app, universal (x86_64 + aarch64)|[705k](https://raw.githubusercontent.com/gatispei/speedketchup-files/main/bin/SpeedKetchup-macos.zip)|||

### Builtin web server
![speedketchup webserver](https://raw.githubusercontent.com/gatispei/speedketchup-files/main/img/ketchup.png "SpeedKetchup webserver")

### Features

- automatic speedtest server selection based on latency or run against specific speedtest server
- uses HTTP protocol for all operations on network, skips speedtest servers that only serve over HTTPS
- tiny list of rust dependencies: [std](https://doc.rust-lang.org/std/index.html), [libc](https://crates.io/crates/libc), [roxmltree](https://crates.io/crates/roxmltree) + [xmlparser](https://crates.io/crates/xmlparser)
- results are stored in human readable [.csv](https://en.wikipedia.org/wiki/Comma-separated_values), and optionally written to [InfluxDB](https://www.influxdata.com) or anything else that takes its line protocol
- optionally published over MQTT, with [Home Assistant](https://www.home-assistant.io) discovery
- builtin web-server to display results with [uPlot](https://github.com/leeoniya/uPlot) charts:
  - speed and latency charts, each with a distribution chart of the displayed results next to it
  - every result, or daily, monthly, time of day or time of week averages
  - legend with min/avg/max of the displayed results and the share of results in the hovered value range
  - sortable table of the results with every recorded detail, including errors
  - updates in place when a test finishes, keeping the zoom and the picked view
- [prometheus](https://prometheus.io) metrics of the last test at `/metrics`: download and upload speed, idle and loaded latency, server and isp, test and failure counts
- all web assets are included in the program binary
- linux version statically linked with [musl](https://musl.libc.org/) - not dependent on system [libc](https://en.wikipedia.org/wiki/C_standard_library), essentially a single binary container
- also acts as a speedtest server, the other direction from its own tests: speedtest clients on your network can test against the machine it runs on. Not registered with official speedtest.net, but can be passed as speedtest server to supporting clients

### Command Line

<pre>
usage: speedketchup [options]
	-h|--help: print this
	-v|--version: print version
	-i|--interval &ltminutes&gt: test interval in minutes, 10 by default, counted from the last test in the results file after a restart
	-f|--file &ltfilename&gt: file to store test results in, speedketchup-results.csv by default
	-a|--address &ltlocal_addr&gt: address to listen on for incoming connections, 127.0.0.1 by default
		local_addr: use 0.0.0.0 to accept connections on all addresses
	-p|--port &ltport&gt: port to listen on for incoming connections, 8288 by default
	-s|--server &ltserver_host[:server_port]&gt: speedtest server to use, avoids automatic server selection if specified
		server_host: domain_name|ipv4_addr|ipv6_addr
		server_port: port number, 8080 by default
	-dd|--download-duration &ltseconds&gt: how long to test download speed, 0 disables test, 10 seconds by default
	-ud|--upload-duration &ltseconds&gt: how long to test upload speed, 0 disables test, 10 seconds by default
	-dc|--download-connections &ltnumber&gt: how many parallel connections to make for download, 8 connections by default
	-uc|--uplaod-connections &ltnumber&gt: how many parallel connections to make for upload, 8 connections by default
	-iu|--influx-url &lturl&gt: also write every result to this InfluxDB line protocol write url, http only
		eg http://influxdb:8086/api/v2/write?org=home&bucket=speedketchup
	-it|--influx-token &lttoken&gt: token for the influx url, sent as 'Authorization: Token &lttoken&gt'
	-mq|--mqtt &lthost[:port]&gt: also publish every result to this mqtt broker, with home assistant discovery
		port: 1883 by default, plain mqtt only
	-mu|--mqtt-user &ltuser&gt: user name for the mqtt broker
	-mp|--mqtt-password &ltpassword&gt: password for the mqtt broker

every option can also be given as an environment variable, command line options
take precedence:
	SPEEDKETCHUP_INTERVAL, SPEEDKETCHUP_FILE, SPEEDKETCHUP_ADDRESS,
	SPEEDKETCHUP_PORT, SPEEDKETCHUP_SERVER, SPEEDKETCHUP_DOWNLOAD_DURATION,
	SPEEDKETCHUP_UPLOAD_DURATION, SPEEDKETCHUP_DOWNLOAD_CONNECTIONS,
	SPEEDKETCHUP_UPLOAD_CONNECTIONS, SPEEDKETCHUP_INFLUX_URL,
	SPEEDKETCHUP_INFLUX_TOKEN, SPEEDKETCHUP_MQTT, SPEEDKETCHUP_MQTT_USER,
	SPEEDKETCHUP_MQTT_PASSWORD
</pre>

### Metrics

`/metrics` serves the last test in the [prometheus text format](https://prometheus.io/docs/instrumenting/exposition_formats/):
download and upload speed in bits per second, idle, download and upload latency
in seconds, the server and isp as labels on `speedketchup_last_test_info`, and
test and failure counts. Run with `-a 0.0.0.0` so the scraper can reach it.

<pre>
scrape_configs:
  - job_name: speedketchup
    static_configs:
      - targets: ["speedketchup-host:8080"]
</pre>

Anything that scrapes prometheus or [OpenMetrics](https://openmetrics.io) endpoints can collect it, for example:

- [Prometheus](https://prometheus.io), and the long term stores fed by it: [Thanos](https://thanos.io), [Cortex](https://cortexmetrics.io), [Grafana Mimir](https://grafana.com/oss/mimir/)
- [VictoriaMetrics](https://victoriametrics.com) and vmagent
- [Grafana Alloy](https://grafana.com/docs/alloy/) (formerly Grafana Agent), for Grafana Cloud
- [OpenTelemetry Collector](https://opentelemetry.io/docs/collector/) with the prometheus receiver
- [Telegraf](https://github.com/influxdata/telegraf) with the prometheus input, for InfluxDB and the other Telegraf outputs
- [Netdata](https://www.netdata.cloud) with its prometheus collector
- [Zabbix](https://www.zabbix.com) with an HTTP agent item and prometheus preprocessing
- [Elastic Metricbeat](https://www.elastic.co/beats/metricbeat) with the prometheus module
- [Datadog Agent](https://docs.datadoghq.com/integrations/openmetrics/) with the OpenMetrics check
- [Checkmk](https://checkmk.com) with its prometheus integration

### InfluxDB

With `--influx-url` every result is also written as one
[line protocol](https://docs.influxdata.com/influxdb/v2/reference/syntax/line-protocol/)
point, right after it is saved to the csv:

<pre>
speedketchup,server=LIVAS/Riga/Latvia,server_host=speedtest.livas.lv:8080,isp=Tele2 download_mbps=40.6,upload_mbps=39.0,latency_idle_ms=44.2,latency_download_ms=71.7,latency_upload_ms=98.9,client_ip="203.0.113.7" 1792775257000000000
</pre>

The values are rounded like in the csv, fields of a disabled test are left out,
and a failed test is a point with just an `error` string field. A write that
fails is logged and not retried, the csv still has the result.

The url is the complete write url, so the same option covers every server that
takes line protocol over http. Put the token in `--influx-token` or
`SPEEDKETCHUP_INFLUX_TOKEN`, it is sent as `Authorization: Token <token>`:

| server | write url |
|-|-|
| InfluxDB 2 | `http://influxdb:8086/api/v2/write?org=home&bucket=speedketchup` + token |
| InfluxDB 3 Core / Enterprise | `http://influxdb:8181/api/v2/write?bucket=speedketchup` + token |
| InfluxDB 1 | `http://influxdb:8086/write?db=speedketchup`, add `&u=user&p=password` with auth enabled |
| [VictoriaMetrics](https://victoriametrics.com) | `http://victoriametrics:8428/write`, it has no strings, `client_ip` and `error` turn up as series that are always 0 |
| [QuestDB](https://questdb.com) | `http://questdb:9000/write` |
| [Telegraf](https://github.com/influxdata/telegraf) influxdb_listener | `http://telegraf:8186/write`, and from there to any Telegraf output |

Only http is supported, for InfluxDB Cloud or anything else behind https write
to a local Telegraf and let it forward.

### MQTT and Home Assistant

With `--mqtt` every result is also published to an MQTT broker, with
[Home Assistant discovery](https://www.home-assistant.io/integrations/mqtt/#mqtt-discovery)
messages, so a SpeedKetchup device shows up in Home Assistant by itself:
download and upload speed, latency idle and while downloading and uploading,
the time of the last test, a problem sensor that is on when the last test
failed, and the server and isp.

With the Mosquitto broker add-on of Home Assistant, the broker is the Home
Assistant host and the user is a Home Assistant user, best one made for it:

<pre>
speedketchup -mq homeassistant.local -mu speedketchup -mp &ltpassword&gt
</pre>

The result goes to `speedketchup/state` as json, the discovery configs under
`homeassistant/`, all retained, so a Home Assistant that restarts gets them
from the broker. The speed and latency sensors go unavailable when there was no
successful test for two intervals. The sensors of a test disabled with `-dd 0`
or `-ud 0` are removed. speedketchup connects for every result and disconnects
when the broker has acknowledged it. Only plain mqtt is supported, no TLS.

### macOS app

SpeedKetchup.app runs in the menu bar, with no Dock icon and no terminal. The
menu shows the last result and the time to the next test, the live speeds
during a test, opens the web page, runs a test right away, starts
SpeedKetchup at login and opens its settings file.

The settings and results are in `~/Library/Application Support/SpeedKetchup`.
`speedketchup.conf` takes the same `SPEEDKETCHUP_...` variables as the
environment, one per line, and the app applies them as soon as the file is
saved. A bad value, or a port another program has, is shown in red in the
menu. The web page is on [127.0.0.1:8288](http://127.0.0.1:8288).

The app is signed with a Developer ID and notarized by Apple. The binary in it
is the same macos universal binary, which runs from a terminal too.

### Windows app

SpeedKetchup is in the [Microsoft Store](https://apps.microsoft.com/detail/9P0J5WV54WXH).
It runs in the notification area of the taskbar, with the same menu as the
macOS app: the last result and the time to the next test, the live speeds
during a test, the web page, a test right away, start at login and the
settings file, which opens in Notepad. The settings work the same way, a bad
value or a port another program has is shown in red in the menu, and the web
page is on [127.0.0.1:8288](http://127.0.0.1:8288).

The app also adds `speedketchup` to the command line, it runs in the terminal
when started from one.

### Building

A plain `make` gives you a binary for the machine you are on.

The release binaries are smaller than that: they are built on a pinned rust
nightly with `-Z build-std`, `-Zlocation-detail=none` and
`-Cpanic=immediate-abort`, which rebuilds the standard library without the
panic formatting machinery.

<pre>
make release   # every released binary, needs a mac
make macos     # just the macos universal binary
make app       # SpeedKetchup.app in bin/SpeedKetchup-macos.zip
make linux     # just the linux + windows binaries
</pre>

The linux and windows binaries are cross built inside the container image in
`build/Dockerfile` - `make linux` builds that container image and re-enters make inside
it, so the only thing needed on the host is [container](https://github.com/apple/container)
on macos, or `CONTAINER=docker` (or podman) anywhere else. The container image carries the
cross toolchains, a musl built for the tier 3 mips targets, upx and qemu-user -
every binary is run once before the build is considered done. Everything lands
in `./bin`.

### Container

Official container image: [gatispei/speedketchup](https://hub.docker.com/r/gatispei/speedketchup) on Docker Hub.

The container image is `FROM scratch`: one layer holding the statically linked binary and
an empty `/data` for the results volume, nothing else. No shell, no libc, no
base container image underneath. Built for linux/amd64, linux/386, linux/arm64 and
linux/arm/v6.

<pre>
make image
</pre>

That cross builds the binaries and assembles the manifest with the same
container runtime as the rest of the build. With docker, point it at buildx:
`make image IMAGE_BUILD="docker buildx build"`.

<pre>
docker volume create speedketchup
docker run -p 8080:8080 -v speedketchup:/data gatispei/speedketchup
</pre>

### QNAP

Packages for x86_64, arm_64 and arm-x41 QNAP NAS are attached to every
release. To install and update SpeedKetchup from the App Center, add the
repository in App Center > Settings > App Repository:

<pre>
https://speedketchup.com/qnap/repo.xml
</pre>

The packages are not signed by QNAP, so on QTS 4.5 and later first tick
"Allow installation of applications without a valid digital signature" in
App Center > Settings > General. The web page is on port 8288, the settings are in
`speedketchup.conf` in the SpeedKetchup folder.

<pre>
make qpkg
</pre>

builds the packages from the linux binaries in `./bin` with QNAP's QDK.

### Alternatives

How speedketchup compares with other periodic testers, from their
documentation and source as of October 2026:

| | speedketchup | [speedtest-tracker](https://github.com/alexjustesen/speedtest-tracker) | [Speedtest-Tracker v2](https://github.com/henrywhitaker3/Speedtest-Tracker) | [MySpeed](https://github.com/gnmyt/MySpeed) | [Netronome](https://github.com/autobrr/netronome) | [Speedmon](https://github.com/barrycarey/Speedmon) |
|-|-|-|-|-|-|-|
| written in | rust | php (laravel) | go, nuxt | javascript | go, react | python |
| speed test | own speedtest.net client | Ookla cli | Ookla cli | Ookla cli, LibreSpeed, Cloudflare | speedtest.net (speedtest-go), iperf3, LibreSpeed | Ookla cli |
| runs as | single static binary, container image, macOS app, windows exe, QNAP package | container image | container image | linux, windows, container image | binaries for linux, macOS, freebsd, windows, container image | python, container image |
| container image size | **0.23 MB** | 67 MB | 35 MB | 62 MB | 46 MB | 83 MB |
| results stored in | csv | sqlite, mysql, mariadb, postgresql | sqlite, postgresql | sqlite, mysql | sqlite, postgresql | only the backends below |
| web page with charts | ✓ | ✓ | ✓ | ✓ | ✓ | - (grafana) |
| prometheus metrics | ✓ | ✓ | ✓ | ✓ | - | - |
| InfluxDB | ✓ v1, v2, v3 and other line protocol servers | ✓ v2 | - | - | - | ✓ v1, v2, graphite |
| MQTT, Home Assistant | ✓ with discovery | - | - | - | MQTT notifications only | - |
| notifications | - | mail, discord, slack, telegram, ntfy, gotify, pushover, apprise, webhook, healthchecks | webhook, healthchecks | mail, signal, whatsapp, telegram | discord, ntfy, gotify, pushover, mail, mqtt, webhook and more | - |
| login | - (127.0.0.1 by default) | ✓ | optional, OpenID Connect | optional password | ✓ local or OpenID Connect | - |
| also | acts as a speedtest server | thresholds, packet loss, translations | api, opentelemetry tracing | multiple servers, retention period | packet loss, traceroute, dns monitoring, remote agents | |
| latest release | 0.5, Oct 2026 | 1.15.1, Oct 2026 | 2.4.2, Oct 2026, as tags and container images only | 1.0.9, May 2024 | 0.16.0, Oct 2026 | no releases, last change Jan 2022 |

All of them measure from the machine they run on, their web page only shows the
results. The self-hosted test servers further down are the other way around, they
measure between a browser and the server.

The Ookla cli based ones need its license accepted, speedketchup talks to the
speedtest.net servers itself. Container image sizes are the compressed linux/amd64
download from the registry.

Prometheus exporters:

- https://github.com/MiguelNdeCarvalho/speedtest-exporter - prometheus exporter in python around the official speedtest cli
- https://github.com/danopstech/speedtest_exporter - prometheus exporter in go

One-off command line testers:

- https://www.speedtest.net/apps/cli - official speedtest cli tool, binary only
- https://github.com/sivel/speedtest-cli - speedtest cli tool in python, archived
- https://github.com/showwin/speedtest-go - speedtest.net cli and library in go
- https://github.com/nelsonjchen/speedtest-rs - speedtest cli tool in rust, unmaintained
- https://github.com/YuLinLee/speedtest - speedtest cli tool in c
- https://github.com/luizluca/speedtest-shell - speedtest.net client in posix shell for openwrt and other small systems
- https://github.com/sindresorhus/fast-cli - fast.com cli in node
- https://github.com/librespeed/speedtest-cli - cli in go for librespeed servers

Self-hosted test servers, they measure the link between a browser and your own server rather than your internet connection:

- https://github.com/librespeed/speedtest - html5 speed test server
- https://github.com/openspeedtest/Speed-Test - html5 speed test server
