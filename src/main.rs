//#![no_std]
//#![no_main]
//extern crate libc;

const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug)]
struct Ipv6Addr(std::net::Ipv6Addr);

impl std::str::FromStr for Ipv6Addr {
    type Err = std::net::AddrParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
	match std::net::Ipv4Addr::from_str(s) {
	    Ok(addr) => Ok(Ipv6Addr(std::net::Ipv4Addr::to_ipv6_mapped(&addr))),
	    Err(_err) => std::net::Ipv6Addr::from_str(s).map(|v| Ipv6Addr(v) )
	}
//	std::net::Ipv6Addr::from_str(s).map(|v| Ipv6Addr(v) )
    }
}
impl std::fmt::Display for Ipv6Addr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
	match self.0.to_ipv4_mapped() {
	    None => write!(f, "{}", self.0),
	    Some(ip) => write!(f, "{}", ip),
	}
    }
}


struct ErrorString(String);
impl std::fmt::Display for ErrorString {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
	write!(f, "{}", self.0)
    }
}

impl From<&str> for ErrorString {
    fn from(err: &str) -> ErrorString {
        ErrorString(err.to_string())
    }
}
impl From<std::str::Utf8Error> for ErrorString {
    fn from(err: std::str::Utf8Error) -> ErrorString {
        ErrorString(err.to_string())
    }
}
impl From<std::io::Error> for ErrorString {
    fn from(err: std::io::Error) -> ErrorString {
        ErrorString(err.to_string())
    }
}
impl From<std::num::ParseFloatError> for ErrorString {
    fn from(err: std::num::ParseFloatError) -> ErrorString {
        ErrorString(err.to_string())
    }
}
impl From<std::num::ParseIntError> for ErrorString {
    fn from(err: std::num::ParseIntError) -> ErrorString {
        ErrorString(err.to_string())
    }
}
impl From<std::net::AddrParseError> for ErrorString {
    fn from(err: std::net::AddrParseError) -> ErrorString {
        ErrorString(err.to_string())
    }
}
impl From<roxmltree::Error> for ErrorString {
    fn from(err: roxmltree::Error) -> ErrorString {
        ErrorString(err.to_string())
    }
}
impl From<std::time::SystemTimeError> for ErrorString {
    fn from(err: std::time::SystemTimeError) -> ErrorString {
        ErrorString(err.to_string())
    }
}
impl From<std::boxed::Box<dyn std::any::Any + std::marker::Send>> for ErrorString {
    fn from(_err: std::boxed::Box<dyn std::any::Any + std::marker::Send>) -> ErrorString {
        ErrorString("thread panciked".to_string())
    }
}

#[derive(Default, Debug)]
struct Point {
    x: f32,
    y: f32,
}
impl Point {
    fn distance(&self, p2: & Point) -> f32 {
	let xd = (self.x - p2.x).abs();
	let yd = (self.y - p2.y).abs();
	(xd * xd + yd * yd).sqrt()
    }
}
const DEGREES_TO_KM: f32 = 40075.0 / 360.0;

impl Default for Ipv6Addr {
    fn default() -> Ipv6Addr {
        Ipv6Addr(std::net::Ipv6Addr::from(0_u128))
    }
}

#[derive(Debug, Default, Clone)]
struct SpeedTestServer {
    descr: String,
    host: String,
//    url: String,
    id: u32,
    distance: f32,
    latency: u32,
}

#[derive(Debug)]
struct SpeedTestResult {
    timestamp: u32,
    latency: u32,
    download: f32,
    upload: f32,
    client_public_ip: Ipv6Addr,
    client_isp: String,
    server: SpeedTestServer,
//    server_id: u32,
//    server_descr: String,
//    server_host: String,
}

#[allow(dead_code)]
fn type_of<T>(_: &T) -> &'static str {
    return std::any::type_name::<T>();
}

fn timestr() -> String {
    let d = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap();
    format!("{}.{:0>3}",
	    libc_strftime::strftime_local("%Y.%m.%d-%H:%M:%S", d.as_secs() as i64),
	    d.subsec_millis())
}

macro_rules! pr {
    ($($arg:tt)*) => {{
	let c = std::thread::current();
	let tn = match c.name() {
	    Some(n) => n,
	    None => ""
	};
	print!("{} {tn}: ", timestr());
	println!($($arg)*);
    }};
}

fn duration<F, T>(work: F) -> Result<u32, ErrorString> where
    F: Fn() -> Result<T, ErrorString> {
    let now = std::time::Instant::now();
    let _ = work()?;
    Ok(now.elapsed().as_millis() as u32)
}

fn url_get_latency(host: &str, path: &str) -> Result<u32, ErrorString> {
    let latencies: Vec<_> = (0..3).filter_map(
	|_i|
	duration(|| {
	    let dur = std::time::Duration::from_secs(1);
	    Ok(http_request(host, path, "GET", "", dur, 0, dummy_cb)?)
	}).ok()
    )
	.collect();
    let mut lat = u32::MAX;
    if latencies.len() > 0 {
	// assume 2 roundtrips per http request
	lat = latencies.iter().sum::<u32>() / 2 / latencies.len() as u32;
    }
    pr!("result {}: {:?} / {}", path, lat, latencies.len());
    Ok(lat)
}

fn servers_sort_by_latency(servers: &mut Vec<SpeedTestServer>) -> Result<(), ErrorString> {
    let threads: Vec<_> = servers.iter().filter_map(|server| {
	let host = server.host.clone();
	Some(std::thread::Builder::new().name(format!("test {} latency", server.host)).spawn(move || -> Result<u32, ErrorString> {
	    url_get_latency(&host, "speedtest/latency.txt")
	}).ok()?)
    }).collect();
    let mut latencies = vec![];
    for thread in threads {
	let x = thread.join()??;
//	pr!("thread join {:?}", x);
	latencies.push(x);
    }
    for (i, lat) in latencies.iter().enumerate() {
	servers[i].latency = *lat;
    }

    servers.sort_by(|a, b| a.latency.cmp(&b.latency));
    Ok(())
}

fn dummy_cb(_: &[u8]) {
}
//#[inline(never)]
fn http_request<READ>(
    host: &str,
    path: &str,
    method: &str,
    extra_headers: &str,
    duration: std::time::Duration,
    send_data_size: usize,
    mut read_cb: READ)
    -> Result<usize, ErrorString>
where
    READ: FnMut(&[u8]) -> ()
{
    let now = std::time::Instant::now();
    let sock_addr = match std::net::ToSocketAddrs::to_socket_addrs(host) {
	Ok(x) => x,
	Err(_) => std::net::ToSocketAddrs::to_socket_addrs(&(host, 80))?
    }.next().ok_or("no addr")?;
//    pr!("http_request {host} {path} {method} {sock_addr} {:?} {send_data_size}", duration);
    let mut tcp_stream = std::net::TcpStream::connect_timeout(&sock_addr, duration)?;

    let mut timeout = Some(duration.checked_sub(now.elapsed()).ok_or("connect too long")?);
    tcp_stream.set_write_timeout(timeout)?;
//    pr!("connect done");
    let req = format!("{method} {path} HTTP/1.1\r
Host: {host}\r
User-Agent: {PKG_NAME}/{PKG_VERSION}\r
Accept: */*\r
Connection: close\r
{extra_headers}\r
");
//    use std::io::prelude::Write;
//    tcp_stream.write(req.as_bytes())?;
    std::io::Write::write_all(&mut tcp_stream, req.as_bytes())?;
    let mut buf = [0_u8; 1024 * 8];
    let mut bytes_written = 0;
//    pr!("write request done");

    // send post data
    loop {
	if bytes_written >= send_data_size {
	    break;
	}
	timeout = duration.checked_sub(now.elapsed());
	if timeout.is_none() {
//	    pr!("write timeout");
	    break;
	}
	tcp_stream.set_write_timeout(timeout)?;
	let mut remain = send_data_size - bytes_written;
	if remain > buf.len() {
	    remain = buf.len();
	}
	match std::io::Write::write(&mut tcp_stream, &buf[0..remain]) {
	    Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
	    Err(_err) => {
//		pr!("write {_err}");
		break
	    }
	    Ok(ret) => {
		if ret == 0 {
//		    pr!("write done");
		    break
		}
//		pr!("write {ret}");
		bytes_written += ret
	    }
	}
    }

    // receive response
    loop {
	timeout = duration.checked_sub(now.elapsed());
	if timeout.is_none() {
//	    pr!("read timeout");
	    break;
	}
	tcp_stream.set_read_timeout(timeout)?;
	match std::io::Read::read(&mut tcp_stream, &mut buf) {
	    Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
	    Err(_err) => {
//		pr!("read {err}");
		break
	    }
	    Ok(bytes_read) => {
		if bytes_read == 0 {
//		    pr!("read done");
		    break
		}
		read_cb(&buf[0..bytes_read]);
//		pr!("read {bytes_read}");
	    }
	}
    }
//    pr!("read");
    Ok(bytes_written)
}

fn http_get(host: &str, path: &str) -> Result<String, ErrorString> {
    let dur = std::time::Duration::from_secs(3);
    let mut resp: Vec<u8> = Vec::new();
    http_request(host, path, "GET", "", dur, 0, |buf| {
	resp.extend_from_slice(buf);
    })?;
    Ok(std::str::from_utf8(&resp)?.to_string())
}
fn http_status_code(resp: &str) -> Result<u32, ErrorString> {
    Ok(resp.split(' ').nth(1).ok_or("no status code")?.parse::<u32>()?)
}
fn http_headers<CB>(resp: &str, mut cb: CB) where
    CB: FnMut(&str, &str) -> () {
    for header in resp.lines() {
	if header.len() == 0 {
	    break;
	}
	let split_at = match header.find(": ") {
	    Some(x) => x,
	    None => continue
	};
	let hdr_type = &header[0..split_at];
	let hdr_val = &header[split_at + 2..];
	//	pr!("heade2: '{hdr_type}' '{hdr_val}'");
	cb(hdr_type, hdr_val);
    }
}
fn http_body(resp: &str) -> Result<String, ErrorString> {
    let mut body = &resp[resp.find("\r\n\r\n").ok_or("no body")? + 4..];
    let mut chunked = false;
    http_headers(resp, |hdr_type, hdr_val| {
	if hdr_type.to_lowercase() == "transfer-encoding" && hdr_val.to_lowercase() == "chunked" {
	    chunked = true;
	}
    });
//    pr!("chunked: {chunked}");
    if chunked == false {
	return Ok(String::from(body));
    }

    let mut body_str = String::new();
    loop {
	let chunk_size_slice: &str = &body[0..body.find(|c| c == ';' || c == '\r').ok_or("bad chunk len")?];
	let chunk_size = usize::from_str_radix(chunk_size_slice, 16)?;
	if chunk_size == 0 {
	    break;
	}
	body = &body[body.find("\r\n").ok_or("no chunk start")? + 2..];
	if chunk_size + 2 > body.len() {
	    return Err("chunk size too big".into());
	}
	body_str += &body[0..chunk_size];
	body = &body[chunk_size + 2..];
	if body.len() == 0 {
	    break;
	}
    }
    Ok(body_str)
}

/*
const HTTP: &str = "http";
fn url_proto(url: &str) -> &str {
    match url.find("://") {
	None => HTTP,
	Some(i) => &url[0..i]
    }
}
*/
fn url_host_and_path(url: &str) -> (&str, &str) {
    let start_idx = match url.find("://") {
	None => 0,
	Some(i) => i + 3
    };
    let end_idx = match url[start_idx..].find('/') {
	None => url.len(),
	Some(i) => start_idx + i
    };
    (&url[start_idx..end_idx], &url[end_idx..])
}

fn http_get_follow_redirects(host: &str, path: &str) -> Result<String, ErrorString> {
    let mut resp;
    let mut h = host.to_string();
    let mut p = path.to_string();
    loop {
	resp = http_get(&h, &p)?;
	match http_status_code(&resp)? {
	    301 | 302 | 303 | 307 | 308 => (),
	    _ => return Ok(resp)
	};
	let mut loc = false;
	http_headers(&resp, |hdr_type, hdr_val| {
	    if hdr_type.to_lowercase() == "location" {
		let (hh, pp) = url_host_and_path(hdr_val);
		pr!("follow redirect to {hh} {pp}");
		h = hh.to_string();
		p = pp.to_string();
		loc = true;
	    }
	});
	if loc == false {
	    return Ok(resp);
	}
    }
}

fn test_download(host_str: &str, duration: std::time::Duration, sizes: &Vec<u32>) -> Result<usize, ErrorString> {
    let mut bytes = 0;
    let now = std::time::Instant::now();
    let mut i = 0;
    loop {
	let timeout = duration.checked_sub(now.elapsed());
//	pr!("download timeout: {:?}", timeout);
	match timeout {
	    None => break,
	    Some(t) => {
		let size = sizes[i];
		let path = format!("speedtest/random{size}x{size}.jpg");
		match http_request(host_str, &path, "GET", "Cache-Control: no-cache\r\n", t, 0, |buf| bytes += buf.len()) {
		    Err(err) => pr!("http error: {err}"),
		    Ok(_) => ()
		}
	    }
	}
	if i < sizes.len() - 1 {
	    i += 1;
	}
    }
    Ok(bytes)
}

fn test_upload(host_str: &str, duration: std::time::Duration, sizes: &Vec<u32>) -> Result<usize, ErrorString> {
    let mut bytes = 0;
    let now = std::time::Instant::now();
    let mut i = 0;
    loop {
	let timeout = duration.checked_sub(now.elapsed());
//	pr!("upload timeout: {:?}", timeout);
	match timeout {
	    None => break,
	    Some(t) => {
		let size = sizes[i];
		let path = "speedtest/upload.php";
		bytes += match http_request(host_str, &path, "POST", &format!("Content-Length: {size}\r\n"), t, size as usize, dummy_cb) {
		    Err(err) => {
			pr!("http error: {err}");
			0
		    }
		    Ok(x) => x
		}
	    }
	}
	if i < sizes.len() - 1 {
	    i += 1;
	}
    }
    Ok(bytes)
}

fn test_multithread(
    host_str: &str,
    duration: std::time::Duration, sizes: &Vec<u32>,
    thread_count: u32,
    func: fn(&str, std::time::Duration, &Vec<u32>) -> Result<usize, ErrorString>)
    -> Result<usize, ErrorString> {
    pr!("test_multithread host:{host_str} duration:{:?} threads:{thread_count} func:{:?}", duration, func);
    let threads: Vec<_> = (0..thread_count).filter_map(|i| {
	let sh: String = host_str.into();
	let ds = sizes.clone();

	Some(std::thread::Builder::new().name(format!("test{i}")).spawn(move || -> usize {
	    match func(&sh, duration, &ds) {
		Ok(bytes) => {
		    pr!("bytes:{bytes}");
		    bytes
		},
		Err(err) => {
		    pr!("{err}");
		    0
		}
	    }
	}).ok()?)
    }).collect();
    let mut bytes = 0;
    for thread in threads {
//	bytes += thread.join()?;
	match thread.join() {
	    Ok(res) => bytes += res,
	    Err(err) => pr!("thread join failed: {:?}", err),
	}
    }
    Ok(bytes)
}

fn speedtest() -> Result<SpeedTestResult, ErrorString> {
    pr!("speedtest");
    let resp = http_get_follow_redirects("www.speedtest.net", "/speedtest-config.php")?;
//    pr!("status:{} body:{}", http_status_code(&resp)?, http_body(&resp)?);
    let config_xml = http_body(&resp)?;
//    pr!("config_xml: {}", config_xml);
    let config = roxmltree::Document::parse(&config_xml)?;
    pr!("config_xml.len(): {:?}", config_xml.len());

    let server_config_node = config.descendants()
        .find(|n| n.has_tag_name("server-config"))
        .ok_or("no server-config")?;
    let download_node = config.descendants()
        .find(|n| n.has_tag_name("download"))
        .ok_or("no download")?;
    let upload_node = config.descendants()
        .find(|n| n.has_tag_name("upload"))
        .ok_or("no upload")?;
    let client_node = config.descendants()
        .find(|n| n.has_tag_name("client"))
        .ok_or("no client")?;

    let ignore_servers: Vec<u32> = server_config_node
        .attribute("ignoreids")
        .ok_or("no ignoreids")?
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.parse::<u32>())
        .collect::<Result<Vec<u32>, _>>()?;

    let client_location: Point = Point {
        x: client_node.attribute("lat").ok_or("no client lat")?.parse()?,
        y: client_node.attribute("lon").ok_or("no client lon")?.parse()?,
    };

    let ratio = upload_node
        .attribute("ratio")
        .ok_or("no ratio")?
        .parse::<usize>()?;

//    let upload_max = upload_node
//        .attribute("maxchunkcount")
//        .ok_or("no maxchunkcount")?
//        .parse::<u32>()?;

    let upload_sizes = vec![32768_u32, 65536, 131072, 262144, 524288, 1048576, 7340032];
    let upload_sizes = upload_sizes.get(ratio - 1..).ok_or("bad upsize")?.to_vec();
    let download_sizes = vec![350_u32, 500, 750, 1000, 1500, 2000, 2500, 3000, 3500, 4000];

//    let size_count = upload_sizes.len();
//    let upload_count = (upload_max as f32 / size_count as f32).ceil() as u32;
//    let download_count = download_node
//        .attribute("threadsperurl")
//        .ok_or("no threadsperurl")?
//        .parse::<u32>()?;

    let upload_threads = upload_node
        .attribute("threads")
        .ok_or("no threads")?
        .parse::<u32>()?;
    let download_threads = server_config_node
        .attribute("threadcount")
        .ok_or("no threadcount")?
        .parse::<u32>()?
        * 2;

    let upload_duration = upload_node
        .attribute("testlength")
        .ok_or("no upload testlength")?
        .parse::<u64>()
        .map(std::time::Duration::from_secs)?;
    let download_duration = download_node
        .attribute("testlength")
        .ok_or("no download testlength")?
        .parse::<u64>()
        .map(std::time::Duration::from_secs)?;

    let client_ip: Ipv6Addr = client_node
        .attribute("ip")
        .ok_or("no client ip")?
        .parse()?;
    let client_isp = client_node
        .attribute("isp")
        .ok_or("no client isp")?
        .to_string();


    let servers_xml = http_body(&http_get_follow_redirects("www.speedtest.net", "/speedtest-servers.php")?)?;
    let servers = roxmltree::Document::parse(&servers_xml)?;
    pr!("servers_xml.len(): {:?}", servers_xml.len());
    let mut servers: Vec<_> = servers
        .descendants()
        .filter(|node| node.tag_name().name() == "server")
        .filter_map(|n| {
            let lll: Point = Point {
                x: n.attribute("lat")?.parse().ok()?,
                y: n.attribute("lon")?.parse().ok()?,
            };
            let country = n.attribute("country")?;
            let name = n.attribute("name")?;
	    let sponsor = n.attribute("sponsor")?;
            Some(SpeedTestServer {
		descr: format!("{}/{}/{}", sponsor, name, country),
                host: n.attribute("host")?.to_string(),
//                url: n.attribute("url")?.to_string(),
                id: n.attribute("id")?.parse().ok()?,
                distance: client_location.distance(&lll) * DEGREES_TO_KM,
		latency: u32::MAX,
            })
        })
        .filter(|server| !ignore_servers.contains(&server.id))
        .collect();
    servers.sort_by(|a, b| {
        a.distance.partial_cmp(&b.distance).unwrap()
    });
    servers.truncate(10);
    servers_sort_by_latency(&mut servers)?;
//    pr!("servers {:#?}", servers);

    let server = servers.iter().next().ok_or("no server")?;
    pr!("test server {:?}", server);
    let bytes = test_multithread(&server.host, download_duration, &download_sizes, download_threads, test_download)?;
    let download_mbps = (bytes as u64 * 8 / download_duration.as_millis() as u64) as f32 / 1000.0;
    pr!("download mbps:{download_mbps} bytes:{bytes}");

    // allow some time for downloads to stop
    std::thread::sleep(std::time::Duration::from_secs(1));

    let bytes = test_multithread(&server.host, upload_duration, &upload_sizes, upload_threads, test_upload)?;
    let upload_mbps = (bytes as u64 * 8 / upload_duration.as_millis() as u64) as f32 / 1000.0;
    pr!("upload mbps:{upload_mbps} bytes:{bytes}");
//    url_upload(&server.host, "speedtest/upload.php", upload_duration, 1000)?;

    let timestamp = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH)?.as_secs() as u32;
    Ok(SpeedTestResult {
	timestamp: timestamp,
	latency: server.latency,
	download: download_mbps,
	upload: upload_mbps,
	client_public_ip: client_ip,
	client_isp: client_isp,
	server: server.clone(),
    })
}

const CSV_COLS: &str = "Timestamp,Latency(ms),Download(Mbps),Upload(Mbps),ClientPublicIP,ClientISP,ServerDescr,ServerHost,Error\n";
fn save_result(file: &str, result: &Result<SpeedTestResult, ErrorString>) {
    let str = match result {
	Ok(r) => format!("{},{},{},{},{},\"{}\",\"{}\",{},\n", r.timestamp, r.latency, r.download, r.upload, r.client_public_ip, r.client_isp, r.server.descr, r.server.host),
	Err(e) => format!(",,,,,,,,{e}\n"),
    };
    let path = std::path::Path::new(file);
    if path.exists() == false {
	if std::fs::write(file, CSV_COLS).is_err() {
	    exit(&format!("cannot write to {}", file), -1);
	}
    }

    match std::fs::OpenOptions::new().write(true).append(true).open(file) {
	Err(e) => exit(&format!("cannot open {file}: {}", e), -1),
	Ok(mut f) => {
	    if let Err(e) = std::io::Write::write(&mut f, str.as_bytes()) {
		pr!("cannot write to {file}: {}", e);
	    }
	},
    }
}

/*
#[derive(Debug)]
struct Config<'src> {
    hostname: &'src str,
    username: &'src str,
}
fn parse_config<'cfg>(config: &'cfg str) -> Config<'cfg> {
    let key_values: std::collections::HashMap<_, _> = config
        .lines()
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| line.split_once('='))
        .map(|(key, value)| (key.trim(), value.trim()))
        .collect();
    Config {
        hostname: key_values["hostname"],
        username: key_values["username"],
    }
}
 */

const HELP: &str = "usage: speedtest [options]
	-h|--help: print this
	-v|--version: print version
	-i|--interval <minutes>: set test interval in minutes, 10 by default
	-s|--store <filename>: file to store test results in, results.csv by default";
fn exit(str: &str, code: i32) -> ! {
    eprintln!("{}", str);
    std::process::exit(code);
}
fn main() {
    let mut test_interval: u64 = 10;
    let mut store_filename = "results.csv";
    let args = std::env::args().collect::<Vec<_>>();
    let mut it = args.iter();
    it.next();
    while let Some(arg) = it.next() {
	match arg.as_str() {
	    "-h" | "--help" => exit(HELP, 0),
	    "-v" | "--version" => exit(PKG_VERSION, 0),
	    "-i" | "--interval" => {
		test_interval = match it.next() {
		    Some(x) => match x.parse() {
			Ok(i) => i,
			Err(_) => exit("bad interval", -1),
		    },
		    None => exit("no interval given", -1),
		}
	    },
	    "-s" | "--store" => {
		store_filename = match it.next() {
		    Some(x) => x,
		    None => exit("no filename given", -1),
		}
	    },
	    _ => exit(&format!("unknown arg '{}'", arg), -1),
	}
    }

    std::env::set_var("RUST_BACKTRACE", "1");
    pr!("interval: {test_interval}m");
    pr!("store_filename: {store_filename}");
    let test_interval = std::time::Duration::from_secs(test_interval * 60);
    loop {
	let now = std::time::Instant::now();
	let result = speedtest();
	match &result {
	    Err(err) => {
		pr!("error: {}", err);
	    }
	    Ok(r) => {
		pr!("result: {:#?}", r);
	    }
	};
	save_result(&store_filename, &result);
	match test_interval.checked_sub(now.elapsed()) {
	    Some(dur) => {
		pr!("sleep for {:?}", dur);
		std::thread::sleep(dur);
	    },
	    None => ()
	}
    }

//    let data: Vec<u8> = std::fs::read("db.txt").unwrap();
//    pr!("data.len: {:#?}", data.len());
//    pr!("data.capacity: {:#?}", data.capacity());

//    let config = parse_config(
//        r#"hostname = foobar
//username=barfoo"#,
//    );
//    pr!("Parsed config: {} {} {:?}", config.hostname, config.username, config);
}

/*
#[no_mangle]
pub extern "C" fn main(_argc: isize, _argv: *const *const u8) -> isize {
    // Since we are passing a C string the final null character is mandatory.
    const HELLO: &'static str = "Hello, world!\n\0";
    unsafe {
        libc::printf(HELLO.as_ptr() as *const _);
    }
    0
}

#[panic_handler]
fn my_panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe {
	libc::exit(1);
    }
}
 */
