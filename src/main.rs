//#![no_std]
//#![no_main]
extern crate libc;

const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

use std::sync::{Arc, Mutex, PoisonError, MutexGuard, mpsc};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::hash_map::HashMap;
use std::time::{Duration, Instant};
use std::ffi::CStr;

/***   Ipv6Addr   ********************************/
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
impl Default for Ipv6Addr {
    fn default() -> Ipv6Addr {
        Ipv6Addr(std::net::Ipv6Addr::from(0_u128))
    }
}


/***   ErrorString   ********************************/

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
impl From<String> for ErrorString {
    fn from(err: String) -> ErrorString {
        ErrorString(err)
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
impl From<PoisonError<MutexGuard<'_, SpeedTestState>>> for ErrorString {
    fn from(_err: PoisonError<MutexGuard<'_, SpeedTestState>>) -> ErrorString {
        ErrorString("mutex error".to_string())
    }
}


/***   Point   ********************************/

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


#[derive(Debug, Clone)]
struct SpeedTestServer {
    descr: String,
    host: String,
//    url: String,
    id: u32,
    distance: f32,
    latency: Option<Duration>,
}

#[derive(Debug)]
struct SpeedTestResult {
//    timestamp: u32,
    latency_idle: Option<Duration>,
    latency_download: Option<Duration>,
    latency_upload: Option<Duration>,
    download: Option<f32>,
    upload: Option<f32>,
    client_public_ip: Ipv6Addr,
    client_isp: String,
    server: SpeedTestServer,
//    server_id: u32,
//    server_descr: String,
//    server_host: String,
}

#[derive(Clone)]
struct SpeedTestConfig {
    test_interval: u64,
    store_filename: String,
    listen_port: u16,
    listen_address: String,
    server_host: Option<String>,

    download_duration: u32,
    download_connections: u32,
    upload_duration: u32,
    upload_connections: u32,
}

struct SpeedTestState {
    status: String,
    idle_until: Option<Instant>,
    gauge_latency: Option<Duration>,
    gauge_download_progress: Option<u32>,
    gauge_download_mbps: Option<f32>,
    gauge_download_latency: Option<Duration>,
    gauge_upload_progress: Option<u32>,
    gauge_upload_mbps: Option<f32>,
    gauge_upload_latency: Option<Duration>,

    config: SpeedTestConfig,
//    last_result: Option<SpeedTestResult>,

    to_conn_senders: HashMap<std::thread::ThreadId, mpsc::Sender<()>>,
    to_main_sender: mpsc::Sender<()>,

    test_requested: Option<bool>,
}

#[allow(dead_code)]
fn type_of<T>(_: &T) -> &'static str {
    return std::any::type_name::<T>();
}

macro_rules! cstr {
    ($($arg:tt)*) => {{
	unsafe {
	    CStr::from_bytes_with_nul_unchecked(concat!($($arg)*, "\0").as_bytes())
	}
    }};
}

#[cfg(windows)]
mod c {
    #[repr(C)]
    pub struct TIMEVAL {
	pub tv_sec: i32,
	pub tv_usec: i32,
    }
    #[repr(C)]
    #[derive(Clone)]
    pub struct FD_SET {
	pub fd_count: u32,
	pub fd_array: [usize; 64],
    }
    extern "C" {
        pub fn strftime(
            s: *mut libc::c_char,
            max: libc::size_t,
            format: *const libc::c_char,
            tm: *const libc::tm,
        ) -> usize;
        pub fn _gmtime64_s(tm: *mut libc::tm, t: *const libc::time_t);
	pub fn _mkgmtime(tm: *const libc::tm) -> libc::time_t;
	pub fn select(
	    nfds: i32,
	    readfds: *mut FD_SET,
	    writefds: *mut FD_SET,
	    exceptfds: *mut FD_SET,
	    timeout: *const TIMEVAL,
	) -> i32;
    }
}

fn strftime_gmt(format: &CStr, epoch: i64) -> String {
    let mut buf = Vec::with_capacity(100);
    unsafe {
	let mut now: libc::tm = std::mem::zeroed(); 
        #[cfg(unix)] {
            libc::gmtime_r(&(epoch as libc::c_long), &mut now);
	    buf.set_len(libc::strftime(buf.as_ptr() as _, buf.capacity(), format.as_ptr() as *const _, &now));
	}
        #[cfg(windows)] {
            c::_gmtime64_s(&mut now, &epoch);
	    buf.set_len(c::strftime(buf.as_ptr() as _, buf.capacity(), format.as_ptr() as *const _, &now));
	}
	std::string::String::from_utf8_unchecked(buf)
    }
}

enum WaitSocketOp {
    Read,
    Write
}
fn wait_socket(stream: &std::net::TcpStream, timeout: Duration, op: WaitSocketOp) -> Result<(), ErrorString> {
    #[cfg(unix)]
    {
        let mut pollfd = libc::pollfd {
	    fd: std::os::unix::io::AsRawFd::as_raw_fd(stream),
	    events: match op {
		WaitSocketOp::Read => libc::POLLIN,
		WaitSocketOp::Write => libc::POLLOUT,
	    },
	    revents: 0
	};
        let now = Instant::now();
	loop {
	    let to = timeout.checked_sub(now.elapsed()).ok_or("timeout")?;
            match unsafe { libc::poll(&mut pollfd, 1, to.as_millis() as libc::c_int) } {
                -1 => {
                    let err = std::io::Error::last_os_error();
		    if err.kind() != std::io::ErrorKind::Interrupted {
                        return Err(err.into());
                    }
                }
		0 => return Err("timeout".into()),
		_ => break,
	    }
	}
	Ok(())
    }
    #[cfg(windows)]
    {
	let mut fds: c::FD_SET = {
            let mut fds = c::FD_SET{ fd_count: 1, fd_array: [0; 64] };
            fds.fd_array[0] = std::os::windows::io::AsRawSocket::as_raw_socket(stream) as usize;
            fds
	};
	let mut errorfds: c::FD_SET = fds.clone();

	let mut readfdsp: *mut c::FD_SET = std::ptr::null_mut();
	let mut writefdsp: *mut c::FD_SET = std::ptr::null_mut();
	match op {
	    WaitSocketOp::Read => readfdsp = &mut fds,
	    WaitSocketOp::Write => writefdsp = &mut fds,
	}
	let to = c::TIMEVAL{ tv_sec: timeout.as_secs() as i32, tv_usec: timeout.subsec_micros() as i32 };

        let result = unsafe {
            c::select(1, readfdsp, writefdsp, &mut errorfds, &to)
        };
	if result == -1 {
	    return Err(std::io::Error::last_os_error().into());
	}
	if fds.fd_count == 0 {
	    return Err("timeout".into());
	}
	Ok(())
    }
}

fn timestr() -> String {
    let d = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap();
    format!("{}", strftime_gmt(cstr!("%Y.%m.%d-%H:%M:%S"), d.as_secs() as i64))
}

fn timestr_millis() -> String {
    let d = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap();
    format!("{}.{:0>3}", strftime_gmt(cstr!("%Y.%m.%d-%H:%M:%S"), d.as_secs() as i64), d.subsec_millis())
}

fn parse_timestr(str: &str) -> Result<u32, ErrorString> {
    let mut i = str.split("-");
    let date: Vec<_> = i.next().ok_or("no date")?.split(".").collect();
    let time: Vec<_> = i.next().ok_or("no time")?.split(":").collect();

    let x = unsafe {
	let mut tm: libc::tm = std::mem::zeroed();
	tm.tm_year = date[0].parse()?;
	tm.tm_year -= 1900;
	tm.tm_mon = date[1].parse()?;
	tm.tm_mon -= 1;
	tm.tm_mday = date[2].parse()?;
	tm.tm_hour = time[0].parse()?;
	tm.tm_min = time[1].parse()?;
	tm.tm_sec = time[2].parse()?;
        #[cfg(unix)]
        let t = libc::timegm(&mut tm);
        #[cfg(windows)]
        let t = c::_mkgmtime(&tm);
	t as u32
    };

    Ok(x)
}

macro_rules! pr {
    ($($arg:tt)*) => {{
	let c = std::thread::current();
	let tn = match c.name() {
	    Some(n) => n,
	    None => ""
	};
	print!("{} {tn}: ", timestr_millis());
	println!($($arg)*);
    }};
}


const ATTR_BOLD: &str = "\x1b[1m";
const ATTR_RED: &str = "\x1b[31m";
const ATTR_GREEN: &str = "\x1b[32m";
//const ATTR_BLUE: &str = "\x1b[34m";
const ATTR_RESET: &str = "\x1b[m";

fn memmem<T>(haystack: &[T], needle: &[T]) -> Option<usize>
where for<'a> &'a [T]: PartialEq {
    haystack.windows(needle.len()).position(|window| window == needle)
}

fn duration<F, T>(work: F) -> Result<Duration, ErrorString> where
    F: Fn() -> Result<T, ErrorString> {
    let now = Instant::now();
    let _ = work()?;
    Ok(now.elapsed())
}

fn url_get_latency<CB>(
    host: &str,
    path: &str,
    max_iters: u32,
    total_dur: Duration,
    intertest_dur: Duration,
    mut cb: Option<CB>)
    -> Result<Option<Duration>, ErrorString>
where
    CB: FnMut(Duration) -> ()
{
    let now = Instant::now();
    let mut lat_tot = Duration::from_secs(0);
    let mut samples = 0;
    let latencies: Vec<_> = (0..max_iters).filter_map(|_i| {
	let mut timeout = total_dur.checked_sub(now.elapsed())?;
	let mut ret = match duration(|| {
	    http_request(host, path, "GET", "", timeout, 0, None::<fn(&[u8])>, None::<fn(usize)>)
	}) {
	    Err(e) => {
		pr!("error: {}", e);
		return None;
	    },
	    Ok(r) => r
	};
	ret /= 2;
	lat_tot += ret;
	samples += 1;
	if let Some(ref mut cb) = cb {
	    cb(lat_tot / samples);
	}
	timeout = total_dur.checked_sub(now.elapsed())?;
	if timeout >= intertest_dur {
	    std::thread::sleep(intertest_dur);
	}
	else {
	    std::thread::sleep(timeout);
	}
	Some(ret)
    }).collect();

    let mut lat = None;
    if latencies.len() > 0 {
	// assume 2 roundtrips per http request
	lat = Some(latencies.iter().sum::<Duration>() / latencies.len() as u32);
    }
    pr!("result: {:?} => {:?}ms", latencies, lat);
    Ok(lat)
}

fn servers_sort_by_latency(servers: &mut Vec<SpeedTestServer>) -> Result<(), ErrorString> {
    let threads: Vec<_> = servers.iter().filter_map(|server| {
	let host = server.host.clone();
	Some(std::thread::Builder::new().name(format!("test latency {}", server.host)).spawn(move || -> Result<Option<Duration>, ErrorString> {
	    url_get_latency(&host, "speedtest/latency.txt", 3,
			    Duration::from_secs(3),
			    Duration::from_millis(50),
			    None::<fn(Duration)>)
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

//#[inline(never)]
fn http_request<READ, WRITE>(
    host: &str,
    path: &str,
    method: &str,
    extra_headers: &str,
    duration: Duration,
    send_data_size: usize,
    mut read_cb: Option<READ>,
    mut write_cb: Option<WRITE>)
    -> Result<(), ErrorString>
where
    READ: FnMut(&[u8]) -> (),
    WRITE: FnMut(usize) -> ()
{
    let now = Instant::now();
    let sock_addr = match std::net::ToSocketAddrs::to_socket_addrs(host) {
	Ok(x) => x,
	Err(_) => std::net::ToSocketAddrs::to_socket_addrs(&(host, 80))
	    .map_err(|e| format!("http_request: to_socket_addrs {e}"))?
    }.next().ok_or("htpp_request: no addr")?;
//    pr!("http_request {host} {path} {method} {sock_addr} {:?} {send_data_size}", duration);
    let mut tcp_stream = std::net::TcpStream::connect_timeout(&sock_addr, duration)
	.map_err(|e| format!("http_request: connect {e}"))?;
    tcp_stream.set_nonblocking(true)?;

    let mut timeout = duration.checked_sub(now.elapsed())
	.ok_or("http_request: connect too long")?;
//    pr!("connect done");
    let req = format!("{method} {path} HTTP/1.1\r
Host: {host}\r
User-Agent: {PKG_NAME}/{PKG_VERSION}\r
Accept: */*\r
Connection: close\r
{extra_headers}\r
");

    // send request
    let mut bytes_written = 0;
    while bytes_written < req.as_bytes().len() {
	wait_socket(&tcp_stream, timeout, WaitSocketOp::Write)
	    .map_err(|e| format!("http_request: wait_socket to send request {e}"))?;
	bytes_written += match std::io::Write::write(&mut tcp_stream, req.as_bytes()) {
	    Err(e) if e.kind() == std::io::ErrorKind::Interrupted => 0,
	    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => 0,
	    Err(err) => return Err(format!("http_request: send request: {err}").into()),
	    Ok(ret) => {
		if ret == 0 {
		    return Err("http_request: send request: closed".into());
		}
		ret
	    }
	}
    }

//    pr!("write request done");
    let mut buf = [0_u8; 1024 * 8];
    bytes_written = 0;

    // send post data
    while bytes_written < send_data_size {
	timeout = duration.checked_sub(now.elapsed())
	    .ok_or("http_request: send post data: timeout")?;
	wait_socket(&tcp_stream, timeout, WaitSocketOp::Write)
	    .map_err(|e| format!("http_request: wait_socket to send post data {e}"))?;
	let mut remain = send_data_size - bytes_written;
	if remain > buf.len() {
	    remain = buf.len();
	}
	match std::io::Write::write(&mut tcp_stream, &buf[0..remain]) {
	    Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
	    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
	    Err(err) => return Err(format!("http_request: send post data: {err}").into()),
	    Ok(ret) => {
		if ret == 0 {
		    return Err("http_request: send post data: closed".into());
		}
//		pr!("write {ret}");
		bytes_written += ret;
		if let Some(ref mut cb) = write_cb {
		    cb(ret);
		}
	    }
	}
    }
//    pr!("write post data done");

    // receive response
    loop {
	timeout = duration.checked_sub(now.elapsed()).ok_or("http_request: receive response timeout")?;
	wait_socket(&tcp_stream, timeout, WaitSocketOp::Read)
	    .map_err(|e| format!("http_request: wait_socket to receive response {e}"))?;
	match std::io::Read::read(&mut tcp_stream, &mut buf) {
	    Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
	    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
	    Err(_err) => {
//		pr!("read {_err}");
		break
	    }
	    Ok(bytes_read) => {
		if bytes_read == 0 {
//		    pr!("read done");
		    break
		}
		if let Some(ref mut cb) = read_cb {
		    cb(&buf[0..bytes_read]);
		}
//		pr!("read {bytes_read}");
	    }
	}
    }
//    pr!("read");
    Ok(())
}

fn http_get(host: &str, path: &str) -> Result<Vec<u8>, ErrorString> {
    let dur = Duration::from_secs(3);
    let mut resp: Vec<u8> = Vec::new();
    http_request(host, path, "GET", "", dur, 0, Some(|buf: &[u8]| {
	resp.extend_from_slice(buf);
    }), None::<fn(usize)>).map_err(|e| format!("http_get {host} {path}: {e}"))?;
    Ok(resp)
}
fn http_status_code(resp: &[u8]) -> Result<u32, ErrorString> {
    let codestr = resp.split(|c| *c == b' ').nth(1).ok_or("http no status code")?;
    Ok(std::str::from_utf8(codestr)?.parse::<u32>()?)
}
fn http_headers<CB>(resp: &[u8], mut cb: CB) where
    CB: FnMut(&[u8], &[u8]) -> () {
    for header in resp.split(|c| *c == b'\n') {
	if header.len() == 0 || header[header.len() - 1] != b'\r' {
	    break;
	}
	let split_at = match memmem(header, b": ") {
	    Some(x) => x,
	    None => continue
	};
	let hdr_type = &header[0..split_at];
	let hdr_val = &header[split_at + 2..header.len() - 1];
//	pr!("heade2: '{:?}' '{:?}'", std::str::from_utf8(hdr_type), std::str::from_utf8(hdr_val));
	cb(hdr_type, hdr_val);
    }
}
fn http_body(resp: &[u8]) -> Result<Vec<u8>, ErrorString> {
    let mut body = &resp[memmem(resp, b"\r\n\r\n").ok_or("no body")? + 4..];
    let mut chunked = false;
    http_headers(resp, |hdr_type, hdr_val| {
	if hdr_type.to_ascii_lowercase() == b"transfer-encoding" && hdr_val.to_ascii_lowercase() == b"chunked" {
	    chunked = true;
	}
    });
//    pr!("chunked: {chunked}");
    if chunked == false {
	return Ok(body.into());
    }

    let mut body_str = Vec::<u8>::new();
    loop {
	let chunk_size_slice: &[u8] = &body.split(|c| *c == b';' || *c == b'\r').next().ok_or("bad chunk len")?;
	let chunk_size = usize::from_str_radix(std::str::from_utf8(chunk_size_slice)?, 16)?;
	if chunk_size == 0 {
	    break;
	}
	body = &body[memmem(body, b"\r\n").ok_or("no chunk start")? + 2..];
	if chunk_size + 2 > body.len() {
	    return Err("chunk size too big".into());
	}
	body_str.extend_from_slice(&body[0..chunk_size]);
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

fn http_get_follow_redirects(host: &str, path: &str) -> Result<Vec<u8>, ErrorString> {
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
	    if hdr_type.to_ascii_lowercase() != b"location" {
		return
	    }
	    if let Ok(v) = std::str::from_utf8(hdr_val) {
		let (hh, pp) = url_host_and_path(v);
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

fn test_download(host_str: &str, duration: Duration, sizes: &Vec<u32>, progress: &Arc<AtomicUsize>) -> Result<usize, ErrorString> {
    let mut bytes = 0;
    let now = Instant::now();
    let mut i = 0;
    loop {
	let timeout = duration.checked_sub(now.elapsed());
//	pr!("download timeout: {:?}", timeout);
	match timeout {
	    None => break,
	    Some(t) if t < Duration::from_millis(1) => break,
	    Some(t) => {
		let size = sizes[i];
		let path = format!("speedtest/random{size}x{size}.jpg");
		match http_request(host_str, &path, "GET", "Cache-Control: no-cache\r\n", t, 0, Some(|buf: &[u8]| {
		    progress.fetch_add(buf.len(), Ordering::Relaxed);
		    bytes += buf.len()
		}), None::<fn(usize)>) {
		    Err(err) => pr!("downlaod {path}: {err}"),
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

fn test_upload(host_str: &str, duration: Duration, sizes: &Vec<u32>, progress: &Arc<AtomicUsize>) -> Result<usize, ErrorString> {
    let mut bytes = 0;
    let now = Instant::now();
    let mut i = 0;
    loop {
	let timeout = duration.checked_sub(now.elapsed());
//	pr!("upload timeout: {:?}", timeout);
	match timeout {
	    None => break,
	    Some(t) if t < Duration::from_millis(1) => break,
	    Some(t) => {
		let size = sizes[i];
		let path = "speedtest/upload.php";
		match http_request(host_str, &path, "POST", &format!("Content-Length: {size}\r\n"), t, size as usize, None::<fn(&[u8])>, Some(|b: usize| {
		    progress.fetch_add(b, Ordering::Relaxed);
		    bytes += b;
		})) {
		    Err(err) => {
			pr!("upload {size}: {err}");
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

fn test_multithread<CB>(
    test_type: &str,
    host_str: &str,
    duration: Duration, sizes: &Vec<u32>,
    thread_count: u32,
    func: fn(&str, Duration, &Vec<u32>, progress: &Arc<AtomicUsize>) -> Result<usize, ErrorString>,
    mut cb: CB)
    -> Result<(Option<f32>, Option<Duration>), ErrorString>
where
    CB: FnMut(u32, f32, Option<Duration>) -> ()
{
    let h: String = host_str.to_string();
    let lat_progress = Arc::new(Mutex::new(Option::None));
    let pr = lat_progress.clone();
    let latency_thread = std::thread::Builder::new().name(format!("test latency {}", h)).spawn(move || -> Result<Option<Duration>, ErrorString> {
	let start_delay = Duration::from_millis(500);
	std::thread::sleep(start_delay);
	url_get_latency(&h, "speedtest/latency.txt", 10,
			duration.checked_sub(start_delay).ok_or("test duration too short for latency test")?,
			Duration::from_secs(1),
			Some(|lat: Duration| {
			    if let Ok(mut p) = pr.lock() {
				*p = Some(lat);
			    }
			}))
    })?;

    pr!("test_multithread host:{host_str} duration:{:?} threads:{thread_count} func:{:?}", duration, func);
    let now = Instant::now();
    let progress = Arc::new(AtomicUsize::new(0));

    let threads: Vec<_> = (0..thread_count).filter_map(|i| {
	let sh: String = host_str.into();
	let ds = sizes.clone();

	let progress = progress.clone();
	Some(std::thread::Builder::new().name(format!("test{i}")).spawn(move || -> usize {
	    match func(&sh, duration, &ds, &progress) {
		Ok(bytes) => {
//		    pr!("bytes:{bytes}");
		    bytes
		},
		Err(err) => {
		    pr!("{err}");
		    0
		}
	    }
	}).ok()?)
    }).collect();

    std::thread::sleep(Duration::from_millis(100));
    while now.elapsed() < duration {
	let bytes = progress.load(Ordering::Relaxed) as u64;
	if let Ok(lat) = lat_progress.lock() {
	    cb(now.elapsed().as_millis() as u32 * 100 / duration.as_millis() as u32,
	       (bytes * 8 / now.elapsed().as_millis() as u64) as f32 / 1000.0,
	       *lat);
	}
	std::thread::sleep(Duration::from_millis(100));
    }

    let mut bytes = vec!();
    for thread in threads {
	match thread.join() {
	    Ok(res) => bytes.push(res),
	    Err(err) => pr!("thread join failed: {:?}", err),
	}
    }

    let latency = match latency_thread.join()? {
	Ok(l) => l,
	Err(e) => {
	    pr!("latency error:{}", e);
	    None
	},
    };

    let mbps = (bytes.iter().sum::<usize>() as u64 * 8 / duration.as_millis() as u64) as f32 / 1000.0;
    cb(100, mbps, latency);
    pr!("{test_type} mbps:{mbps} bytes:{:?}", bytes);
    Ok((Some(mbps), latency))
}

fn set_status(state: &Arc<Mutex<SpeedTestState>>, new_status: &str) ->Result<(), ErrorString> {
    let mut state = state.lock()?;
    state.status = new_status.to_string();
    for (_threadid, tx) in state.to_conn_senders.iter() {
	let _ = tx.send(());
    }
    Ok(())
}

fn speedtest(server: &Option<String>, state: &Arc<Mutex<SpeedTestState>>) -> Result<SpeedTestResult, ErrorString> {
    pr!("speedtest");
    set_status(state, "get provider info")?;
    let cfg = state.lock()?.config.clone();
    let resp = http_get_follow_redirects("www.speedtest.net", "/speedtest-config.php")
	.map_err(|e| format!("get config: {e}"))?;
//    pr!("status:{} body:{}", http_status_code(&resp)?, http_body(&resp)?);
    let config_xml = http_body(&resp).map_err(|e| format!("config no body: {e}"))?;
    //    pr!("config_xml: {}", config_xml);
    let config_body = std::str::from_utf8(&config_xml)
	.map_err(|e| format!("config body bad utf8: {e}"))?;
    let config = roxmltree::Document::parse(&config_body)
	.map_err(|e| format!("config bad xml: {e}"))?;
    pr!("config_xml.len(): {:?}", config_xml.len());

    let server_config_node = config.descendants()
        .find(|n| n.has_tag_name("server-config"))
        .ok_or("no server-config")?;
//    let download_node = config.descendants()
//        .find(|n| n.has_tag_name("download"))
//        .ok_or("no download")?;
    let upload_node = config.descendants()
        .find(|n| n.has_tag_name("upload"))
        .ok_or("no upload config")?;
    let client_node = config.descendants()
        .find(|n| n.has_tag_name("client"))
        .ok_or("no client info")?;

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

//     let upload_threads = upload_node
//         .attribute("threads")
//         .ok_or("no threads")?
//         .parse::<u32>()?;
//     let download_threads = server_config_node
//         .attribute("threadcount")
//         .ok_or("no threadcount")?
//         .parse::<u32>()?
//         * 2;

//     let upload_duration = upload_node
//         .attribute("testlength")
//         .ok_or("no upload testlength")?
//         .parse::<u64>()
//         .map(Duration::from_secs)?;
//     let download_duration = download_node
//         .attribute("testlength")
//         .ok_or("no download testlength")?
//         .parse::<u64>()
//         .map(Duration::from_secs)?;

    let client_ip: Ipv6Addr = client_node
        .attribute("ip")
        .ok_or("no client ip")?
        .parse()?;
    let client_isp = client_node
        .attribute("isp")
        .ok_or("no client isp")?
        .to_string();


    let mut servers: Vec<_>;
    match server {
	None => {
	    let servers_xml = http_body(&http_get_follow_redirects("www.speedtest.net", "/speedtest-servers.php")?)?;
	    pr!("servers_xml.len(): {:?}", servers_xml.len());
	    let servers_xml = roxmltree::Document::parse(&std::str::from_utf8(&servers_xml)?)?;
	    servers = servers_xml
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
			latency: None,
		    })
		})
		.filter(|server| !ignore_servers.contains(&server.id))
		.collect();
	    if servers.len() == 0 {
		pr!("servers_xml {:#?}", servers_xml);
	    }
	    servers.sort_by(|a, b| {
		a.distance.partial_cmp(&b.distance).unwrap()
	    });
	},
	Some(x) => {
	    servers = vec!(SpeedTestServer {
		descr: "".into(),
		host: x.to_string(),
		id: 0,
		distance: 0.0,
		latency: None,
	    });
	}
    };
    set_status(state, "test latency")?;
    servers.truncate(10);
    servers_sort_by_latency(&mut servers)?;

    let server = servers.iter().filter(|s| s.latency.is_some() )
	.next();
    let server = match server {
	Some(s) => s,
	None => {
	    pr!("servers {:#?}", servers);
	    return Err(ErrorString("no good server".into()));
	}
    };
    pr!("test server {:?}", server);
    if let Ok(mut state) = state.lock() {
	state.gauge_latency = server.latency;
    }

    let mut download_mbps = None;
    let mut download_latency = None;
    let mut upload_mbps = None;
    let mut upload_latency = None;
    if cfg.download_duration > 0 {
	set_status(state, "test download")?;
	(download_mbps, download_latency) = test_multithread("download", &server.host, Duration::from_secs(cfg.download_duration as u64), &download_sizes, cfg.download_connections, test_download, |progress, speed, latency|{
	    if let Ok(mut state) = state.lock() {
		state.gauge_download_progress = Some(progress);
		state.gauge_download_mbps = Some(speed);
		state.gauge_download_latency = latency;
		for (_threadid, tx) in state.to_conn_senders.iter() {
		    let _ = tx.send(());
		}
	    }
	})?;

	if cfg.upload_duration > 0 {
	    // allow some time for downloads to stop
	    set_status(state, "test wait")?;
	    std::thread::sleep(Duration::from_secs(1));
	}
    }

    if cfg.upload_duration > 0 {
	set_status(state, "test upload")?;
	(upload_mbps, upload_latency) = test_multithread("upload", &server.host, Duration::from_secs(cfg.upload_duration as u64), &upload_sizes, cfg.upload_connections, test_upload, |progress, speed, latency|{
	    if let Ok(mut state) = state.lock() {
		state.gauge_upload_progress = Some(progress);
		state.gauge_upload_mbps = Some(speed);
		state.gauge_upload_latency = latency;
		for (_threadid, tx) in state.to_conn_senders.iter() {
		    let _ = tx.send(());
		}
	    }
	})?;
    }

    Ok(SpeedTestResult {
	latency_idle: server.latency,
	latency_download: download_latency,
	latency_upload: upload_latency,
	download: download_mbps,
	upload: upload_mbps,
	client_public_ip: client_ip,
	client_isp: client_isp,
	server: server.clone(),
    })
}

fn float_to_str(n: f32) -> String {
    if n >= 1000.0 || n <= 0.0{
	return format!("{}", n);
    }
    format!("{:.1$}", n, (2 - n.log10().floor() as i32) as usize)
}

fn dur_to_str(n: Option<Duration>) -> String {
    match n {
	None => String::new(),
	Some(n) => float_to_str(n.as_nanos() as f32 / 1000_000.0),
    }
}

fn opt_float_to_str(n: Option<f32>) -> String {
    match n {
	None => String::new(),
	Some(n) => float_to_str(n),
    }
}

const CSV_COLS: &str = "Timestamp,LatencyIdle(ms),LatencyDownload(ms),LatencyUpload(ms),Download(Mbps),Upload(Mbps),ClientPublicIP,ClientISP,ServerDescr,ServerHost,Error\n";
fn save_result(file: &str, result: &Result<SpeedTestResult, ErrorString>) {
    let str = match result {
	Ok(r) => format!("{},{},{},{},{},{},{},\"{}\",\"{}\",{},\n",
			 timestr(),
			 dur_to_str(r.latency_idle),
			 dur_to_str(r.latency_download),
			 dur_to_str(r.latency_upload),
			 opt_float_to_str(r.download), opt_float_to_str(r.upload),
			 r.client_public_ip, r.client_isp,
			 r.server.descr, r.server.host),
	Err(e) => format!("{},,,,,,,,,,\"{e}\"\n", timestr()),
    };
    let path = std::path::Path::new(file);
    if path.exists() == false {
	if std::fs::write(file, CSV_COLS).is_err() {
	    exit(&format!("cannot write to {file}"), -1);
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

#[allow(dead_code)]
fn iterate_lines_in_file_backwards<CB>(file: &str, mut cb: CB) -> Result<(), ErrorString>
where CB: FnMut(&[u8]) -> bool {
    let mut fh = std::fs::File::open(file)?;
    let size = fh.metadata()?.len() as usize;
    if size == 0 {
	return Err(ErrorString("empty file".into()));
    }
    const BUF_LEN: usize = 1024 * 4;
    let mut buf = [0_u8; BUF_LEN * 2];
    let mut off = size - (size & (BUF_LEN - 1));
    let mut rlen = size - off;
    let mut extra = 0;
    if off == size {
	off -= BUF_LEN;
	rlen = BUF_LEN;
    }
    loop {
	std::io::Seek::seek(&mut fh, std::io::SeekFrom::Start(off as u64))?;
	let mut rbuf = &mut buf[0..rlen];
	std::io::Read::read_exact(&mut fh, &mut rbuf)?;
	let start = memmem(rbuf, b"\n").unwrap_or(rlen);
	for line in buf[(start + 1)..(rlen + extra)].rsplit(|c| *c == b'\n') {
	    if cb(line) == false {
		return Ok(());
	    }
	}

	if off == 0 {
	    if start > 0 {
		cb(&buf[0..start]);
	    }
	    break;
	}
	buf.copy_within(0..start, BUF_LEN);
	extra = start;
	rlen = BUF_LEN;
	off -= BUF_LEN;
    }
    Ok(())
}


const ASSET_INDEX_HTML: &[u8] = include_bytes!("../asset/index.html");
const ASSET_FAVICON_SVG: &[u8] = include_bytes!("../asset/favicon.svg");
const ASSET_UPLOT_JS: &[u8] = include_bytes!("../asset/uplot.js");
const ASSET_UPLOT_CSS: &[u8] = include_bytes!("../asset/uplot.css");

#[derive(Clone)]
struct JSDur(Option<Duration>);
impl std::fmt::Debug for JSDur {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
	match self.0 {
	    None => write!(f, "null"),
	    Some(_) => write!(f, "{}", dur_to_str(self.0)),
	}
    }
}
#[derive(Clone)]
struct JSf32(f32);
impl std::fmt::Debug for JSf32 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
	if self.0.is_nan() {
	    return write!(f, "null")
	}
	write!(f, "{}", float_to_str(self.0))
    }
}
fn server_get_data(state: &Arc<Mutex<SpeedTestState>>) -> Result<String, ErrorString> {
    let filename = state.lock()?.config.store_filename.clone();
    let mut timestamps: Vec<u32> = vec!();
    let mut lat_idles: Vec<JSDur> = vec!();
    let mut lat_dls: Vec<JSDur> = vec!();
    let mut lat_uls: Vec<JSDur> = vec!();
    let mut downloads: Vec<JSf32> = vec!();
    let mut uploads: Vec<JSf32> = vec!();
    let data = match std::fs::read(&filename) {
	Ok(d) => d,
	Err(_e) => Vec::<u8>::new(),
    };
    let mut it = std::str::from_utf8(&data)?.lines();
    it.next();
    while let Some(line) = it.next() {
	let x: Vec<_> = line.split(",").take(6).collect();
	if x.len() < 4 {
	    continue;
	}
	let time = match parse_timestr(x[0]) {
	    Err(_) => continue,
	    Ok(t) => t,
	};
	let lat_idle = match x[1].parse::<f32>() {
	    Err(_) => None,
	    Ok(t) => Some(Duration::from_nanos((t * 1000_000.0) as u64)),
	};
	let lat_dl = match x[2].parse::<f32>() {
	    Err(_) => None,
	    Ok(t) => Some(Duration::from_nanos((t * 1000_000.0) as u64)),
	};
	let lat_ul = match x[3].parse::<f32>() {
	    Err(_) => None,
	    Ok(t) => Some(Duration::from_nanos((t * 1000_000.0) as u64)),
	};
	let dl = match x[4].parse::<f32>() {
	    Err(_) => f32::NAN,
	    Ok(t) => t,
	};
	let ul = match x[5].parse::<f32>() {
	    Err(_) => f32::NAN,
	    Ok(t) => t,
	};
	timestamps.push(time);
	lat_idles.push(JSDur(lat_idle));
	lat_dls.push(JSDur(lat_dl));
	lat_uls.push(JSDur(lat_ul));
	downloads.push(JSf32(dl));
	uploads.push(JSf32(ul));
    }
    if timestamps.len() == 1 {
	timestamps.push(timestamps[0].clone() + 1);
	lat_idles.push(lat_idles[0].clone());
	lat_dls.push(lat_dls[0].clone());
	lat_uls.push(lat_uls[0].clone());
	downloads.push(downloads[0].clone());
	uploads.push(uploads[0].clone());
    }
    Ok(format!("let data = [{:?}, {:?}, {:?}, {:?}, {:?}, {:?}]; let stateObj = {};", timestamps, lat_idles, lat_dls, lat_uls, downloads, uploads, server_get_status(state)?))
}

fn server_get_status(state: &Arc<Mutex<SpeedTestState>>) -> Result<String, ErrorString> {
    let state = state.lock()?;
    let mut str = format!("{{ \"state\":\"{}\"", state.status);
    if let Some(x) = state.idle_until {
	str += &format!(", \"idle_time\": {}", (x - Instant::now()).as_secs());
    }
    if let Some(x) = state.gauge_latency {
	str += &format!(", \"latency\": {}", dur_to_str(Some(x)));
    }
    if let Some(x) = state.gauge_download_progress {
	str += &format!(", \"download_progress\": {}", x);
    }
    if let Some(x) = state.gauge_download_mbps {
	str += &format!(", \"download_mbps\": {}", float_to_str(x));
    }
    if let Some(x) = state.gauge_download_latency {
	str += &format!(", \"download_latency\": {}", dur_to_str(Some(x)));
    }
    if let Some(x) = state.gauge_upload_progress {
	str += &format!(", \"upload_progress\": {}", x);
    }
    if let Some(x) = state.gauge_upload_mbps {
	str += &format!(", \"upload_mbps\": {}", float_to_str(x));
    }
    if let Some(x) = state.gauge_upload_latency {
	str += &format!(", \"upload_latency\": {}", dur_to_str(Some(x)));
    }
    str += " }";
    Ok(str)
}

fn server_request(url: &[u8], content: &[u8], mut stream: &mut std::net::TcpStream, state: &Arc<Mutex<SpeedTestState>>, rx: &mpsc::Receiver<()>) -> Result<(), ErrorString> {
    let wait_duration = Duration::from_secs(30);
    let duration = Duration::from_secs(60);
    let now = Instant::now();
    let url = std::str::from_utf8(url)?;
    let urls = url.strip_prefix("/").unwrap_or(&url);
    let content = std::str::from_utf8(content)?;
//    pr!("request {urls} {content}");
    let mut _data: String = String::new();

    let (data, content_type, additional_bytes) = match urls {
	"" => (ASSET_INDEX_HTML, "text/html; charset=utf-8", 0),
	"favicon.svg" => (ASSET_FAVICON_SVG, "image/svg+xml", 0),
	"uplot.js" => (ASSET_UPLOT_JS, "text/javascript", 0),
	"uplot.css" => (ASSET_UPLOT_CSS, "text/css", 0),
	"data.js" => {
	    _data = server_get_data(&state)?;
	    (_data.as_bytes(), "text/javascript", 0)
	},
	"status" => {
	    let mut current_status = server_get_status(state)?;
	    if content.len() > 0 && current_status == content {
		while current_status == content {
		    let timeout = wait_duration.checked_sub(now.elapsed());
		    if timeout.is_none() {
			break;
		    }
		    let _ = rx.recv_timeout(timeout.unwrap_or_default());
		    current_status = server_get_status(state)?;
		}
	    }
	    _data = current_status;
	    (_data.as_bytes(), "text/plain", 0)
	},
	"start" => {
	    pr!("start");
	    let mut state = state.lock()?;
	    state.test_requested = Some(true);
	    let _ = state.to_main_sender.send(());
	    ("".as_bytes(), "text/plain", 0)
	},
	"stop" => {
	    pr!("stop");
	    state.lock()?.test_requested = Some(false);
	    ("".as_bytes(), "text/plain", 0)
	},
	"speedtest/latency.txt" => {
	    ("test=test".as_bytes(), "text/plain", 0)
	},
	"speedtest/upload.php" => ("".as_bytes(), "text/plain", 0),
	"speedtest/random350x350.jpg" => ("".as_bytes(), "text/plain", 245388),
	"speedtest/random500x500.jpg" => ("".as_bytes(), "text/plain", 505544),
	"speedtest/random750x750.jpg" => ("".as_bytes(), "text/plain", 1118012),
	"speedtest/random1000x1000.jpg" => ("".as_bytes(), "text/plain", 1986284),
	"speedtest/random1500x1500.jpg" => ("".as_bytes(), "text/plain", 4468241),
	"speedtest/random2000x2000.jpg" => ("".as_bytes(), "text/plain", 7907740),
	"speedtest/random2500x2500.jpg" => ("".as_bytes(), "text/plain", 12407926),
	"speedtest/random3000x3000.jpg" => ("".as_bytes(), "text/plain", 17816816),
	"speedtest/random3500x3500.jpg" => ("".as_bytes(), "text/plain", 24262167),
	"speedtest/random4000x4000.jpg" => ("".as_bytes(), "text/plain", 31625365),
	"speedtest/upload.php/random350x350.jpg" => ("".as_bytes(), "text/plain", 245388),
	"speedtest/upload.php/random500x500.jpg" => ("".as_bytes(), "text/plain", 505544),
	"speedtest/upload.php/random750x750.jpg" => ("".as_bytes(), "text/plain", 1118012),
	"speedtest/upload.php/random1000x1000.jpg" => ("".as_bytes(), "text/plain", 1986284),
	"speedtest/upload.php/random1500x1500.jpg" => ("".as_bytes(), "text/plain", 4468241),
	"speedtest/upload.php/random2000x2000.jpg" => ("".as_bytes(), "text/plain", 7907740),
	"speedtest/upload.php/random2500x2500.jpg" => ("".as_bytes(), "text/plain", 12407926),
	"speedtest/upload.php/random3000x3000.jpg" => ("".as_bytes(), "text/plain", 17816816),
	"speedtest/upload.php/random3500x3500.jpg" => ("".as_bytes(), "text/plain", 24262167),
	"speedtest/upload.php/random4000x4000.jpg" => ("".as_bytes(), "text/plain", 31625365),
	_ => {
	    pr!("unknown path {}", urls);
	    ("404".as_bytes(), "text/html", 0)
	},
    };

    let mut resp = format!("HTTP/1.1 200 OK\r
Content-Type: {content_type}\r
Content-Length: {}\r
\r
", data.len() + additional_bytes).as_bytes().to_vec();
    resp.extend_from_slice(data);
    let mut bytes_written = 0;

    // send post data
    while bytes_written < resp.len() {
	let timeout = duration.checked_sub(now.elapsed());
	if timeout.is_none() {
	    return Err("write timeout".into());
	}
	stream.set_write_timeout(timeout)?;
	match std::io::Write::write(&mut stream, &resp[bytes_written..]) {
	    Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
	    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
	    Err(_err) => {
		pr!("write {_err}");
		break
	    }
	    Ok(ret) => {
		if ret == 0 {
		    return Err("write closed".into());
		}
//		pr!("write {ret}");
		bytes_written += ret
	    }
	}
    }

    if additional_bytes > 0 {
//	pr!("send additional bytes {}", additional_bytes);
	let buf = [0_u8; 1024 * 8];
	bytes_written = 0;
	while bytes_written < additional_bytes {
	    let timeout = duration.checked_sub(now.elapsed());
	    if timeout.is_none() {
		return Err("write timeout".into());
	    }
	    stream.set_write_timeout(timeout)?;
	    let len = std::cmp::min(additional_bytes - bytes_written, buf.len());
	    match std::io::Write::write(&mut stream, &buf[0..len]) {
		Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
		Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
		Err(_err) => break,
		Ok(ret) => {
		    if ret == 0 {
			return Err("write closed".into());
		    }
		    bytes_written += ret
		}
	    }
	}
//	pr!("sent additional bytes {}", bytes_written);
    }
//    pr!("request done");
    Ok(())
}

fn server_connection(mut stream: std::net::TcpStream, state: &Arc<Mutex<SpeedTestState>>, rx: mpsc::Receiver<()>) -> Result<(), ErrorString> {
    pr!("new connection");
    let duration = Duration::from_secs(10);
    let mut now = Instant::now();
    let mut buf = [0_u8; 1024 * 8];
    let mut drop_buf = [0_u8; 1024 * 8];
    let mut bytes_read = 0;
    let mut hdrend_off = 0;
    let mut content_length: usize = 0;
    let mut close = false;
    loop {
//	while rx.try_recv().is_ok() {
	    // clear out status notification queue
//	}
	let timeout = duration.checked_sub(now.elapsed());
	if timeout.is_none() {
	    return Err("read timeout".into());
	}
	stream.set_read_timeout(timeout)?;
	match match bytes_read < buf.len() {
	    true => std::io::Read::read(&mut stream, &mut buf[bytes_read..]),
	    false => std::io::Read::read(&mut stream, &mut drop_buf)
	} {
	    Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
	    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
	    Err(_e) => {
//		pr!("read {e}");
		break
	    }
	    Ok(bytes) => {
		if bytes == 0 {
		    return Err("read closed".into());
		}
		bytes_read += bytes;
//		pr!("read {bytes_read}");
	    }
	}
	if hdrend_off == 0 {
	    if let Some(off) = memmem(&buf, b"\r\n\r\n") {
		hdrend_off = off;
		let hdr = &buf[0..hdrend_off + 2];
//		pr!("request: {}", std::str::from_utf8(hdr).unwrap());
		http_headers(hdr, |hdr_type, hdr_val| {
//		    pr!("hdr: {}", std::str::from_utf8(hdr_type).unwrap());
		    if hdr_type.to_ascii_lowercase() == b"connection" && hdr_val.to_ascii_lowercase() == b"close" {
			close = true;
		    }
		    if hdr_type.to_ascii_lowercase() == b"content-length" {
			content_length = std::str::from_utf8(hdr_val).unwrap_or_default().parse().unwrap_or_default();
		    }
		});
		hdrend_off += 4;
	    }
	}
	if bytes_read >= hdrend_off + content_length {
	    let hdr = &buf[0..hdrend_off];
	    let content = &buf[hdrend_off..std::cmp::min(hdrend_off + content_length, buf.len())];
	    let mut iter = hdr.split(|c| *c == b' ');
	    match iter.next() {
		Some(b"GET") | Some(b"POST") => {
		    match iter.next() {
			Some(url) => server_request(url, content, &mut stream, &state, &rx)?,
			_ => return Err("bad request url".into()),
		    }
		}
		_ => return Err("bad request method".into()),
	    }
	    if close == true {
		break;
	    }

	    if hdrend_off + content_length < buf.len() {
		buf.copy_within((hdrend_off + content_length)..bytes_read, 0);
		bytes_read -= hdrend_off + content_length;
	    }
	    else {
		bytes_read = 0;
	    }
	    hdrend_off = 0;
	    content_length = 0;
	    now = Instant::now();
	}
    }
    pr!("close connection");
    Ok(())
}

//use std::os::fd::AsRawFd;
fn server(listener: std::net::TcpListener, state: Arc<Mutex<SpeedTestState>>) {
    #[cfg(unix)]
    unsafe {
	let optval: libc::c_int = 1;
	let ret = libc::setsockopt(
//            listener.as_raw_fd(),
            std::os::fd::AsRawFd::as_raw_fd(&listener),
            libc::SOL_SOCKET,
            libc::SO_REUSEPORT,
            &optval as *const _ as *const libc::c_void,
            std::mem::size_of_val(&optval) as libc::socklen_t,
	);
	if ret != 0 {
            pr!("setsockopt failed: {}", std::io::Error::last_os_error());
	}
    }
    for stream in listener.incoming() {
	match stream {
	    Ok(s) => {
		let peer_addr = match s.peer_addr() {
		    Err(e) => {
			pr!("bad peer addr {e}");
			continue;
		    }
		    Ok(x) => x,
		};
		let state = state.clone();
		if let Err(e) = std::thread::Builder::new().name(format!("server-conn-{peer_addr}")).spawn(move || {
		    let (tx, rx) = mpsc::channel();
		    if let Ok(mut state) = state.lock() {
			state.to_conn_senders.insert(std::thread::current().id(), tx);
		    }
		    if let Err(e) = server_connection(s, &state, rx) {
			pr!("error: {e}");
		    }
		    if let Ok(mut state) = state.lock() {
			state.to_conn_senders.remove(&std::thread::current().id());
		    }
		}) {
		    pr!("thread failed {e}");
		}
	    },
	    Err(e) => pr!("connection failed {e}")
	}
    }
}

const HELP: &str = "usage: speedketchup [options]
	-h|--help: print this
	-v|--version: print version
	-i|--interval <minutes>: test interval in minutes, 10 by default
	-f|--file <filename>: file to store test results in, speedketchup-results.csv by default
	-a|--address <local_addr>: address to listen on for incoming connections, 127.0.0.1 by default
		local_addr: use 0.0.0.0 to accept connections on all addresses
	-p|--port <port>: port to listen on for incoming connections, 8080 by default
	-s|--server <server_host[:server_port]>: speedtest server to use, avoids automatic server selection if specified
		server_host: domain_name|ipv4_addr|ipv6_addr
		server_port: port number, 8080 by default
	-dd|--download-duration <seconds>: how long to test download speed, 0 disables test, 10 seconds by default
	-ud|--upload-duration <seconds>: how long to test upload speed, 0 disables test, 10 seconds by default
	-dc|--download-connections <number>: how many parallel connections to make for download, 8 connections by default
	-uc|--uplaod-connections <number>: how many parallel connections to make for upload, 8 connections by default
";
fn exit(str: &str, code: i32) -> ! {
    match code {
	0 => eprintln!("{str}"),
	_ => eprintln!("{ATTR_RED}{ATTR_BOLD}{str}{ATTR_RESET}"),
    };
    std::process::exit(code);
}

fn main() {
    /*
    match iterate_lines_in_file_backwards("speedketchup-results.csv", |line| {
	println!("{}", std::str::from_utf8(line).unwrap());
	true
    }) {
	Err(e) => exit(&e.to_string(), -1),
	Ok(()) => ()
    }
    return ();
     */
    let mut config = SpeedTestConfig {
	test_interval: 10,
	store_filename: "speedketchup-results.csv".to_string(),
	listen_port: 8080,
	listen_address: "127.0.0.1".to_string(),
	server_host: None,
	download_duration: 10,
	download_connections: 8,
	upload_duration: 10,
	upload_connections: 8,
    };

    let args = std::env::args().collect::<Vec<_>>();
    let mut it = args.iter();
    it.next();
    while let Some(arg) = it.next() {
	match arg.as_str() {
	    "-h" | "--help" => exit(HELP, 0),
	    "-v" | "--version" => exit(PKG_VERSION, 0),
	    "-i" | "--interval" => {
		config.test_interval = match it.next() {
		    Some(x) => match x.parse() {
			Ok(i) => i,
			Err(_) => exit("bad interval", -1),
		    },
		    None => exit("no interval given", -1),
		}
	    },
	    "-f" | "--file" => {
		config.store_filename = match it.next() {
		    Some(x) => x.clone(),
		    None => exit("no filename given", -1),
		}
	    },
	    "-a" | "--address" => {
		config.listen_address = match it.next() {
		    Some(x) => x.clone(),
		    None => exit("no address given", -1),
		}
	    },
	    "-p" | "--port" => {
		config.listen_port = match it.next() {
		    Some(x) => match x.parse() {
			Ok(i) => i,
			Err(_) => exit("bad port", -1),
		    },
		    None => exit("no port given", -1),
		}
	    },
	    "-s" | "--server" => {
		config.server_host = match it.next() {
		    Some(x) => {
			match x.find(":") {
			    Some(_off) => Some(x.to_string()),
			    None => Some(format!("{x}:8080"))
			}
		    }
		    None => exit("no server given", -1),
		}
	    },
	    "-dd" | "--download-duration" => {
		config.download_duration = match it.next() {
		    Some(x) => match x.parse() {
			Ok(i) => i,
			Err(_) => exit("bad duration", -1),
		    },
		    None => exit("no duration given", -1),
		}
	    },
	    "-dc" | "--download-connections" => {
		config.download_connections = match it.next() {
		    Some(x) => match x.parse() {
			Ok(i) => i,
			Err(_) => exit("bad number", -1),
		    },
		    None => exit("no number given", -1),
		}
	    },
	    "-ud" | "--upload-duration" => {
		config.upload_duration = match it.next() {
		    Some(x) => match x.parse() {
			Ok(i) => i,
			Err(_) => exit("bad duration", -1),
		    },
		    None => exit("no duration given", -1),
		}
	    },
	    "-uc" | "--upload-connections" => {
		config.upload_connections = match it.next() {
		    Some(x) => match x.parse() {
			Ok(i) => i,
			Err(_) => exit("bad number", -1),
		    },
		    None => exit("no number given", -1),
		}
	    },
	    _ => exit(&format!("unknown arg '{}'", arg), -1),
	}
    }

    std::env::set_var("RUST_BACKTRACE", "1");
    println!("speedketchup parameters (run with '-h' to see options):");
    println!("{ATTR_GREEN}test interval: {ATTR_BOLD}{}m{ATTR_RESET}", config.test_interval);
    println!("{ATTR_GREEN}results file: {ATTR_BOLD}{}{ATTR_RESET}", config.store_filename);
    println!("{ATTR_GREEN}listen address: {ATTR_BOLD}{}:{}{ATTR_RESET}",
	     config.listen_address, config.listen_port);
    println!("{ATTR_GREEN}server: {ATTR_BOLD}{}{ATTR_RESET}",
	     match &config.server_host { None => "<automatic>", Some(x) => &x });
    println!("{ATTR_GREEN}download-duration: {ATTR_BOLD}{}s{ATTR_RESET}", config.download_duration);
    println!("{ATTR_GREEN}download-connections: {ATTR_BOLD}{}{ATTR_RESET}", config.download_connections);
    println!("{ATTR_GREEN}upload-duration: {ATTR_BOLD}{}s{ATTR_RESET}", config.upload_duration);
    println!("{ATTR_GREEN}upload-connections: {ATTR_BOLD}{}{ATTR_RESET}", config.upload_connections);
    let sk_url = &format!("http://127.0.0.1:{}", config.listen_port);

    let listener = match std::net::TcpListener::bind((config.listen_address.as_str(), config.listen_port)) {
	Err(e) => exit(&format!("could not bind {}:{}: {}", config.listen_address, config.listen_port, e), -1),
	Ok(l) => l
    };
#[cfg(unix)]
    let xxx = std::process::Command::new("open").args([sk_url]).output();
#[cfg(windows)]
    let xxx = std::process::Command::new("cmd.exe").args(["/C", "start", "", sk_url]).output();
    match xxx {
	Err(e) => pr!("could not open {sk_url}: {e}"),
	Ok(_) => ()
    };
    println!("speedketchup is at {ATTR_BOLD}{sk_url}{ATTR_RESET}");

    let (tx, rx) = mpsc::channel();
    let state = Arc::new(Mutex::new(SpeedTestState {
	status: String::new(),
	idle_until: None,
	gauge_latency: None,
	gauge_download_progress: None,
	gauge_download_mbps: None,
	gauge_download_latency: None,
	gauge_upload_progress: None,
	gauge_upload_mbps: None,
	gauge_upload_latency: None,
	config: config,
//	last_result: None,
	to_conn_senders: HashMap::new(),
	to_main_sender: tx,
	test_requested: None,
    }));

    {
	let state = state.clone();
	let _ = std::thread::Builder::new().name("server".to_string()).spawn(move || {
	    server(listener, state);
	});
    }

    let mut test_interval = Duration::from_secs(0);
    loop {
	let now = Instant::now();
	let mut server_host = None;
	if let Ok(mut state) = state.lock() {
	    server_host = state.config.server_host.clone();
	    state.idle_until = None;
	    state.test_requested = None;
	}
	let result = speedtest(&server_host, &state);
	match &result {
	    Err(err) => {
		pr!("speedtest error:{}", err);
	    }
	    Ok(r) => {
//		pr!("result: {:#?}", r);
		pr!("speedtest latency:idle-{}ms/dl-{}ms/ul-{}ms download:{}Mbps upload:{}Mbps client_ip:{} isp:{} server:{}",
		    dur_to_str(r.latency_idle), dur_to_str(r.latency_download), dur_to_str(r.latency_upload), opt_float_to_str(r.download), opt_float_to_str(r.upload), r.client_public_ip, r.client_isp, r.server.host);
	    }
	};
	let mut filename = String::new();
	if let Ok(mut state) = state.lock() {
	    state.status = "save result".to_string();
	    test_interval = Duration::from_secs(state.config.test_interval * 60);
	    filename = state.config.store_filename.clone();
	    state.gauge_download_progress = None;
	    state.gauge_upload_progress = None;
            state.idle_until = Some(Instant::now() + test_interval.checked_sub(now.elapsed()).unwrap_or_default());
	}
	save_result(&filename, &result);
        let _ = set_status(&state, "idle");

	// sleep for rest of test_interval unless test is manually requested
	while let Some(dur) = test_interval.checked_sub(now.elapsed()) {
	    pr!("sleep for {:?}", dur);
	    if rx.recv_timeout(dur).is_ok() {
		if let Ok(state) = state.lock() {
		    if state.test_requested == Some(true) {
			break;
		    }
		}
	    }
	}
    }
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
