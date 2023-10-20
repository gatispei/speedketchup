//#![no_std]
//#![no_main]
//extern crate libc;

pub struct ErrorString(String);
impl std::fmt::Display for ErrorString {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
	write!(f, "{}", self.0)
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


pub fn download_configuration() -> Result<bool, ErrorString> {
    let config_xml: String = ureq::get("http://www.speedtest.net/speedtest-config.php")
        .call()?.into_string()?;
    let config = roxmltree::Document::parse(&config_xml)?;
    println!("config: {:?}", config);

    let servers_xml: String = ureq::get("http://www.speedtest.net/speedtest-servers.php")
        .call()?.into_string()?;
    let servers = roxmltree::Document::parse(&servers_xml)?;
    println!("servers: {:?}", servers);
    Ok(true)
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
    println!("cfg: {}", cfg);

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
