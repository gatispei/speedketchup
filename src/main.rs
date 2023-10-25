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
impl From<ureq::Error> for ErrorString {
    fn from(err: ureq::Error) -> ErrorString {
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
//impl From<url::ParseError> for ErrorString {
//    fn from(err: url::ParseError) -> ErrorString {
//        ErrorString(err.to_string())
//    }
//}
impl From<std::boxed::Box<dyn std::any::Any + std::marker::Send>> for ErrorString {
    fn from(_err: std::boxed::Box<dyn std::any::Any + std::marker::Send>) -> ErrorString {
        ErrorString("thread panciked".to_string())
    }
}

#[derive(Debug)]
struct SpeedtestResult {
    timestamp: u32,
    /*
    latency: u32,
    download: u32,
    upload: u32,
    public_ip: std::net::Ipv6Addr,
    isp: String,
    server_id: u32,
    server_sponsor: String,
    server_host: String,
    */
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

#[derive(Default, Debug)]
struct SpeedTestConfig {
    client_public_ip: Ipv6Addr,
    client_isp: String,
//    client_location: Point,

    upload_sizes: Vec<u32>,
    download_sizes: Vec<u32>,
    upload_count: u32,
    download_count: u32,
    upload_threads: u32,
    download_threads: u32,
    upload_duration: std::time::Duration,
    download_duration: std::time::Duration,
    upload_max: u32,

    servers: Vec<SpeedTestServer>,
    ignore_servers: Vec<u32>,
}

#[derive(Debug, Default)]
struct SpeedTestServer {
    descr: String,
    host: String,
    url: String,
    id: u32,
//    location: Point,
    distance: f32,
    latency: u32,
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
	print!("{}: ", timestr());
	println!($($arg)*);
    }};
}

fn duration<F, T>(work: F) -> Result<u32, ErrorString> where
    F: Fn() -> Result<T, ErrorString> {
    let now = std::time::Instant::now();
    let _ = work()?;
    Ok(now.elapsed().as_millis() as u32)
}

fn get_url_latency(url: &str) -> Result<u32, ErrorString> {
    let latencies: Vec<_> = (0..3).filter_map(
	|_i|
	duration(|| {
	    let dur = std::time::Duration::from_secs(1);
	    let agent = ureq::AgentBuilder::new()
		.timeout_connect(dur)
		.timeout_read(dur)
		.timeout_write(dur)
		.build();
	    Ok(agent.get(&url).call()?)
	}).ok()
    )
	.collect();
    let mut lat = u32::MAX;
    if latencies.len() > 0 {
	lat = latencies.iter().sum::<u32>() / latencies.len() as u32;
    }
    pr!("check {}: {:?} / {}", url, lat, latencies.len());
    Ok(lat)
}

fn download_url(host_str: &str, path: &str, duration: std::time::Duration) -> Result<usize, ErrorString> {
    let now = std::time::Instant::now();
    let mut bytes: usize = 0;
    let sock_addr = std::net::ToSocketAddrs::to_socket_addrs(host_str)?.next().ok_or("no addr")?;
//    pr!("download_url {host_str} {path} {sock_addr}");
    let mut tcp_stream = std::net::TcpStream::connect_timeout(&sock_addr, duration)?;

    let mut timeout = Some(duration.checked_sub(now.elapsed()).ok_or("connect too long")?);
    tcp_stream.set_write_timeout(timeout)?;
//    pr!("connect done");
    let req = format!("GET {path} HTTP/1.1\r
Host: {host_str}\r
User-Agent: {PKG_NAME}/{PKG_VERSION}\r
Accept: */*\r
Cache-control: no-cache\r
Connection: close\r
\r
");
//    use std::io::prelude::Write;
//    tcp_stream.write(req.as_bytes())?;
    std::io::Write::write_all(&mut tcp_stream, req.as_bytes())?;
    let mut buf = [0_u8; 1024 * 8];
//    pr!("write done");
    loop {
	timeout = duration.checked_sub(now.elapsed());
	if timeout.is_none() {
//	    pr!("read timeout");
	    break;
	}
	tcp_stream.set_read_timeout(timeout)?;
	match std::io::Read::read(&mut tcp_stream, &mut buf) {
	    Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
	    Err(err) => {
//		pr!("read {err}");
		break
	    }
	    Ok(bytes_read) => {
		if bytes_read == 0 {
//		    pr!("read done");
		    break
		}
//		pr!("read {bytes_read}");
		bytes += bytes_read
	    }
	}
    }
//    pr!("read {bytes}");
    Ok(bytes)
}

fn download_test(host_str: &str, duration: std::time::Duration, sizes: &Vec<u32>) -> Result<usize, ErrorString> {
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
		bytes += download_url(host_str, &format!("speedtest/random{size}x{size}.jpg"), t)?;
	    }
	}
	if i < sizes.len() - 1 {
	    i += 1;
	}
    }
    Ok(bytes)
}

fn speedtest() -> Result<SpeedtestResult, ErrorString> {
    pr!("download_configuration");
    let config_xml: String = ureq::get("http://www.speedtest.net/speedtest-config.php")
        .call()?.into_string()?;
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

    let upload_max = upload_node
        .attribute("maxchunkcount")
        .ok_or("no maxchunkcount")?
        .parse::<u32>()?;

    let upload_sizes = vec![32768_u32, 65536, 131072, 262144, 524288, 1048576, 7340032];
    let upload_sizes = upload_sizes.get(ratio - 1..).ok_or("bad upsize")?.to_vec();
    let download_sizes = vec![350_u32, 500, 750, 1000, 1500, 2000, 2500, 3000, 3500, 4000];

    let size_count = upload_sizes.len();
    let upload_count = (upload_max as f32 / size_count as f32).ceil() as u32;
    let download_count = download_node
        .attribute("threadsperurl")
        .ok_or("no threadsperurl")?
        .parse::<u32>()?;

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


    let servers_xml: String = ureq::get("http://www.speedtest.net/speedtest-servers.php")
        .call()?.into_string()?;
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
		descr: format!("{}, {}, {}", sponsor, name, country),
                host: n.attribute("host")?.to_string(),
                url: n.attribute("url")?.to_string(),
                id: n.attribute("id")?.parse().ok()?,
                distance: client_location.distance(&lll) * DEGREES_TO_KM,
//		location: lll,
		latency: u32::MAX,
            })
        })
        .filter(|server| !ignore_servers.contains(&server.id))
        .collect();
    servers.sort_by(|a, b| {
        a.distance.partial_cmp(&b.distance).unwrap()
    });
    servers.truncate(10);

//    pr!("servers {:#?}", servers);
    let threads: Vec<_> = servers.iter().filter_map(|server| {
	let path = std::path::Path::new(&server.url);
	let latency_url = format!(
	    "{}/latency.txt",
	    path.parent().ok_or("bad server path").ok()?.display()
	);
	Some(std::thread::spawn(move || -> Result<u32, ErrorString> {
	    get_url_latency(&latency_url)
	}))
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

    let server = servers.iter().next().ok_or("no server")?;

    let duration = std::time::Duration::from_secs(10);
    pr!("test {:?} {:?} {download_threads}", duration, server);
    let threads: Vec<_> = (0..download_threads).map(|_i| {
	let sh = server.host.clone();
	let ds = download_sizes.clone();
	std::thread::spawn(move || -> Result<usize, ErrorString> {
	    download_test(&sh, duration, &ds)
	})
    }).collect();
    let mut bytes = 0;
    for thread in threads {
	bytes += thread.join()??;
    }

//    let bytes = download_test(&server.host, duration, &download_sizes)?;
    let mbps = (bytes as f64) * 8.0 / 1000.0 / (duration.as_millis() as f64);
    pr!("mbps:{mbps} bytes:{bytes}");

/*
    let config = SpeedTestConfig {
	client_public_ip: client_ip,
	client_isp: client_isp,
//	client_location: client_location,

	upload_sizes: upload_sizes,
	download_sizes: download_sizes,
	upload_count: upload_count,
	download_count: download_count,
	upload_threads: upload_threads,
	download_threads: download_threads,
	upload_duration: upload_duration,
	download_duration: download_duration,
	upload_max: upload_max,

	servers: servers,
	ignore_servers: ignore_servers,
    };
    pr!("config: {:#?}", config);
*/
    let timestamp = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH)?.as_secs() as u32;
    Ok(SpeedtestResult {timestamp: timestamp})
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

fn main() {
//    pr!("sizeof opt: {}", std::mem::size_of::<Option<u128>>());
    std::env::set_var("RUST_BACKTRACE", "1");

    let result = match speedtest() {
	Err(err) => {
	    pr!("speedtest error: {}", err);
	    return;
	}
	Ok(r) => r
    };
    pr!("result: {:?}", result);

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
