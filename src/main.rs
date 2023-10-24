//#![no_std]
//#![no_main]
//extern crate libc;

#[derive(Debug)]
struct Ipv6Addr(std::net::Ipv6Addr);

impl std::str::FromStr for Ipv6Addr {
    type Err = std::net::AddrParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
	match std::net::Ipv4Addr::from_str(s) {
	    Ok(addr) => Ok(Ipv6Addr(std::net::Ipv4Addr::to_ipv6_mapped(&addr))),
	    Err(err) => std::net::Ipv6Addr::from_str(s).map(|v| Ipv6Addr(v) )
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
//    country: String,
    host: String,
    id: u32,
//    location: Point,
    distance: f32,
//    name: String,
//    sponsor: String,
    url: String,
}

fn download_configuration() -> Result<SpeedtestResult, ErrorString> {
    let config_xml: String = ureq::get("http://www.speedtest.net/speedtest-config.php")
        .call()?.into_string()?;
    let config = roxmltree::Document::parse(&config_xml)?;
    println!("config: {:?}", config);

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
    println!("servers: {:?}", servers);
    let servers: Vec<_> = servers
        .descendants()
        .filter(|node| node.tag_name().name() == "server")
        .map::<Result<_, ErrorString>, _>(|n| {
            let lll: Point = Point {
                x: n.attribute("lat").ok_or("no lat")?.parse()?,
                y: n.attribute("lon").ok_or("no lon")?.parse()?,
            };
            let country = n.attribute("country").ok_or("bad country")?;
            let name = n.attribute("name").ok_or("bad name")?;
	    let sponsor = n.attribute("sponsor").ok_or("bad sponsor")?;
            Ok(SpeedTestServer {
		descr: format!("{}, {}, {}", sponsor, name, country),
                host: n.attribute("host").ok_or("bad host")?.to_string(),
                id: n.attribute("id").ok_or("bad id")?.parse()?,
                distance: client_location.distance(&lll) * DEGREES_TO_KM,
//		location: lll,
                url: n.attribute("url").ok_or("bad url")?.to_string(),
            })
        })
        .filter_map(Result::ok)
//        .filter(|server| !config.ignore_servers.contains(&server.id))
        .collect();

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

    println!("config: {:#?}", config);

    let timestamp = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH)?.as_secs() as u32;
//    let res: SpeedtestResult;
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

//fn type_of<T>(_: &T) -> &'static str {
//    return std::any::type_name::<T>();
//}

fn main() {
/*
    backtrace::trace(|frame| {
	println!("backtrace: {:?}", frame);
	backtrace::resolve_frame(frame, |symbol| {
	    println!("sym: {:?}", symbol);
	});
	true
    });
*/
    println!("sizeof opt: {}", std::mem::size_of::<Option<u8>>());
    println!("sizeof opt: {}", std::mem::size_of::<Option<u16>>());
    println!("sizeof opt: {}", std::mem::size_of::<Option<u32>>());
    println!("sizeof opt: {}", std::mem::size_of::<Option<u64>>());
    println!("sizeof opt: {}", std::mem::size_of::<Option<u128>>());

    std::env::set_var("RUST_BACKTRACE", "1");
//    println!("Hello World!");

//    let mut cmd = std::process::Command::new("speedtest");
//    cmd.arg("--csv");

    let cfg = match download_configuration() {
	Err(err) => {
	    println!("download config error: {}", err);
	    return;
	}
	Ok(cfg) => cfg
    };
    println!("cfg: {:?}", cfg);

//    let data: Vec<u8> = std::fs::read("db.txt").unwrap();
//    println!("data.len: {:#?}", data.len());
//    println!("data.capacity: {:#?}", data.capacity());

//    let config = parse_config(
//        r#"hostname = foobar
//username=barfoo"#,
//    );
//    println!("Parsed config: {} {} {:?}", config.hostname, config.username, config);
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
