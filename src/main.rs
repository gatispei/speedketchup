//#![no_std]
//#![no_main]
//extern crate libc;

pub struct ErrorString(String);
impl std::fmt::Display for ErrorString {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
	write!(f, "{}", self.0)
    }
}

impl From<reqwest::Error> for ErrorString {
    fn from(err: reqwest::Error) -> ErrorString {
        ErrorString(err.to_string())
    }
}
impl From<::std::io::Error> for ErrorString {
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
    let client = reqwest::blocking::Client::new();
    let config_xml = client.get("http://www.speedtest.netlskdj/speedtest-config.php")
        .header(reqwest::header::CONNECTION, "close")
        .header(reqwest::header::USER_AGENT, "netwatch")
        .send()?.text()?;
//    println!("{}", config_xml);

    let config = roxmltree::Document::parse(&config_xml)?;
    println!("config: {:?}", config);

    let servers_xml = client
        .get("http://www.speedtest.net/speedtest-servers.php")
        .header(reqwest::header::CONNECTION, "close")
        .header(reqwest::header::USER_AGENT, "netwatch")
        .send()?.text()?;
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
static _BLA: &str = "hei";

fn type_of<T>(_: &T) -> &'static str {
    return std::any::type_name::<T>();
}

fn xxx<'a>(par: &'a mut String, par2: &String) -> &'a str {
    *par += " add ";
    *par += par2;
//    par.as_str();
    _BLA
}

enum Bla {
    Opt1,
    Opt2(u8),
    Opt3([u32; 10]),
}

enum Bla5 {
    Opt5([u8; 39]),
    Opt6(u8),
    Opt7(u32),
}

enum Bla2 {
    Opt2(Bla5),
    Opt1(Bla),
    Opt3,
}

fn main() {
    //    let e = Bla::Opt2(7);
    let e1 = Bla2::Opt1(Bla::Opt1);
    let e2 = Bla2::Opt1(Bla::Opt2(99));
    let e3 = Bla2::Opt1(Bla::Opt3([99; 10]));
    let e4 = Bla2::Opt2(Bla5::Opt5([5; 39]));
    let e5 = Bla2::Opt2(Bla5::Opt6(100));
    let e6 = Bla2::Opt2(Bla5::Opt7(100));
    let e7 = Bla2::Opt3;
    unsafe {
	println!("e1: {:?}", std::mem::transmute::<Bla2, [u8; 44]>(e1));
	println!("e2: {:?}", std::mem::transmute::<Bla2, [u8; 44]>(e2));
	println!("e3: {:?}", std::mem::transmute::<Bla2, [u8; 44]>(e3));
	println!("e4: {:?}", std::mem::transmute::<Bla2, [u8; 44]>(e4));
	println!("e5: {:?}", std::mem::transmute::<Bla2, [u8; 44]>(e5));
	println!("e6: {:?}", std::mem::transmute::<Bla2, [u8; 44]>(e6));
	println!("e7: {:?}", std::mem::transmute::<Bla2, [u8; 44]>(e7));
    }
    let mut s = String::from("bla");
    let s2 = String::from("bla2");
//    {
    let ss: &str = xxx(& mut s, & s2);
    println!("ss: {}", ss);
    println!("s: {}", s);
//    }
    println!("s2: {}", s2);

    let s = "bla";
    println!("s: {:?}", s);
    println!("typeof s: {}", type_of(&s));
    let a = Some(5);
    println!("a: {:?}", a);
    println!("typeof a: {}", type_of(&a));
    let x: u8 = 5;
    let y: u16 = x as u16 + 255;
    println!("x: {}", x);

    let z: u8 = y as u8;
    println!("z: {}", z);

    let zzz: Result<u8, _> = y.try_into();
    println!("typeof zzz: {}", type_of(&zzz));
    println!("zzz:{:?}", zzz);
    println!("zzz.err: {}", zzz.err().unwrap().to_string());
    let res: Result<u32, _> = "bla".parse();
    println!("typeof res: {}", type_of(&res));
//    println!("sizeof ErrorString: {}", std::mem::size_of::<ErrorString>());
/*
    backtrace::trace(|frame| {
	println!("backtrace: {:?}", frame);
	backtrace::resolve_frame(frame, |symbol| {
	    println!("sym: {:?}", symbol);
	});
	true
    });
*/
    println!("sizeof Bla: {}", std::mem::size_of::<Bla>());
    println!("sizeof Bla2: {}", std::mem::size_of::<Bla2>());
    println!("sizeof opt: {}", std::mem::size_of::<Option<u8>>());
    println!("sizeof opt: {}", std::mem::size_of::<Option<u16>>());
    println!("sizeof opt: {}", std::mem::size_of::<Option<u32>>());
    println!("sizeof opt: {}", std::mem::size_of::<Option<u64>>());
    println!("sizeof opt: {}", std::mem::size_of::<Option<u128>>());
    println!("sizeof reqwest::Error: {}", std::mem::size_of::<reqwest::Error>());
    println!("sizeof std::io::Error: {}", std::mem::size_of::<std::io::Error>());
    println!("sizeof std::num::ParseFloatError: {}", std::mem::size_of::<std::num::ParseFloatError>());
    println!("sizeof std::num::ParseIntError: {}", std::mem::size_of::<std::num::ParseIntError>());
    println!("sizeof std::net::AddrParseError: {}", std::mem::size_of::<std::net::AddrParseError>());
    println!("sizeof roxmltree::Error: {}", std::mem::size_of::<roxmltree::Error>());

    println!("sizeof std::num::ParseIntError: {}", std::mem::size_of::<std::num::ParseIntError>());
    println!("sizeof &std::num::ParseIntError: {}", std::mem::size_of::<&std::num::ParseIntError>());
    println!("sizeof u8: {}", std::mem::size_of::<u8>());
    println!("sizeof u16: {}", std::mem::size_of::<u16>());
    println!("sizeof u32: {}", std::mem::size_of::<u32>());
    println!("sizeof u64: {}", std::mem::size_of::<u64>());
    println!("sizeof u128: {}", std::mem::size_of::<u128>());

    println!("sizeof &u8: {}", std::mem::size_of::<&u8>());
    println!("sizeof &u16: {}", std::mem::size_of::<&u16>());
    println!("sizeof &u32: {}", std::mem::size_of::<&u32>());
    println!("sizeof &u64: {}", std::mem::size_of::<&u64>());
    println!("sizeof &u128: {}", std::mem::size_of::<&u128>());

    println!("sizeof res8: {}", std::mem::size_of::<Result<u8, std::num::ParseIntError>>());
    println!("sizeof res16: {}", std::mem::size_of::<Result<u16, std::num::ParseIntError>>());
    println!("sizeof res32: {}", std::mem::size_of::<Result<u32, std::num::ParseIntError>>());
    println!("sizeof res64: {}", std::mem::size_of::<Result<u64, std::num::ParseIntError>>());
    println!("sizeof res128: {}", std::mem::size_of::<Result<u128, std::num::ParseIntError>>());

    println!("sizeof res8: {}", std::mem::size_of::<Result<&u8, &std::num::ParseIntError>>());
    println!("sizeof res16: {}", std::mem::size_of::<Result<&u16, &std::num::ParseIntError>>());
    println!("sizeof res32: {}", std::mem::size_of::<Result<&u32, &std::num::ParseIntError>>());
    println!("sizeof res64: {}", std::mem::size_of::<Result<&u64, &std::num::ParseIntError>>());
    println!("sizeof res128: {}", std::mem::size_of::<Result<&u128, &std::num::ParseIntError>>());
    //    println!("typeof res: {}", typeof::type_of(res));
    //    println!("res.ok: {:?}", res.ok());
    let res2 = res.as_ref();
    println!("typeof res2: {}", type_of(&res2));
    if res2.is_err() {
	println!("res.err: {}", res2.err().unwrap().to_string());
    }
    if res2.is_err() {
	println!("res.err: {:?}", res2.err());
    }
    if res2.is_err() {
	println!("res.err: {:#?}", res2.err());
    }
    println!("res.err: {:?}", res.err().unwrap().to_string());
//    return;
//    let _guess: u32 = "bla".parse().expect("Not a num!");

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
