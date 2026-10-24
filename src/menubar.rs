// The menu bar icon of SpeedKetchup.app on macOS: the last result, opening
// the web page, running a test now, starting at login and the settings file.
// AppKit is called through the Objective-C runtime, no crates needed.

use super::*;
use std::ffi::{c_char, c_void};
use std::sync::OnceLock;

type Id = *mut c_void;
type Sel = *const c_void;

#[link(name = "objc")]
extern "C" {
    fn objc_getClass(name: *const c_char) -> Id;
    fn sel_registerName(name: *const c_char) -> Sel;
    fn objc_allocateClassPair(superclass: Id, name: *const c_char, extra_bytes: usize) -> Id;
    fn objc_registerClassPair(class: Id);
    fn class_addMethod(class: Id, name: Sel, imp: *const c_void, types: *const c_char) -> bool;
    fn objc_msgSend();
}

#[link(name = "AppKit", kind = "framework")]
extern "C" {
    static NSFontAttributeName: Id;
    static NSForegroundColorAttributeName: Id;
}

// SMAppService, the login item of macOS 13 and later
#[link(name = "ServiceManagement", kind = "framework")]
extern "C" {}

#[link(name = "Foundation", kind = "framework")]
extern "C" {
    static NSRunLoopCommonModes: Id;
}

// a nul terminated C string literal
macro_rules! cs {
    ($str:literal) => { concat!($str, "\0").as_ptr() as *const c_char };
}

// objc_msgSend called as a function of the exact argument and return types
macro_rules! msg {
    ($ret:ty; $obj:expr, $sel:literal $(, $arg:expr => $t:ty)*) => {{
	let f: unsafe extern "C" fn(Id, Sel $(, $t)*) -> $ret = std::mem::transmute(objc_msgSend as unsafe extern "C" fn());
	f($obj, sel_registerName(cs!($sel)) $(, $arg)*)
    }};
}

const LABEL: &str = "com.speedketchup.SpeedKetchup";

const SETTINGS: &str = "# SpeedKetchup settings, one VARIABLE=value per line, lines starting with #
# are left out, the ones below show the defaults. SpeedKetchup applies them as
# soon as this file is saved: the interval right away, the others from the
# next test. A bad value keeps the previous settings, the menu shows what is
# wrong. A new port moves the web page there, its address in the browser too.

# Minutes between the speed tests
#SPEEDKETCHUP_INTERVAL=10

# Port of the web page with the results, on 127.0.0.1
#SPEEDKETCHUP_PORT=8288

# Speedtest server host[:port] to always test against, empty picks the server
# with the lowest latency for every test
#SPEEDKETCHUP_SERVER=

# Seconds to test the download and upload speed, 0 skips the test
#SPEEDKETCHUP_DOWNLOAD_DURATION=10
#SPEEDKETCHUP_UPLOAD_DURATION=10

# Parallel connections for the download and upload tests
#SPEEDKETCHUP_DOWNLOAD_CONNECTIONS=8
#SPEEDKETCHUP_UPLOAD_CONNECTIONS=8

# InfluxDB line protocol write url to also write every result to, http only,
# eg http://influxdb:8086/api/v2/write?org=home&bucket=speedketchup, and its token
#SPEEDKETCHUP_INFLUX_URL=
#SPEEDKETCHUP_INFLUX_TOKEN=

# MQTT broker host[:port] to also publish every result to, with Home Assistant
# discovery, plain mqtt only, and its user name and password
#SPEEDKETCHUP_MQTT=
#SPEEDKETCHUP_MQTT_USER=
#SPEEDKETCHUP_MQTT_PASSWORD=
";

struct Menu {
    state: Arc<Mutex<SpeedTestState>>,
    // NSMenuItem pointers, AppKit only touches them on the main thread
    result_item: usize,
    time_item: usize,
    error_item: usize,
    login_item: usize,
}

static MENU: OnceLock<Menu> = OnceLock::new();

// running from Contents/MacOS of an .app bundle, as opposed to the plain binary
pub fn is_app() -> bool {
    std::env::current_exe().map(|p| p.to_string_lossy().contains(".app/Contents/MacOS/")).unwrap_or(false)
}

fn home() -> String {
    std::env::var("HOME").unwrap_or_default()
}

pub fn app_dir() -> String {
    format!("{}/Library/Application Support/SpeedKetchup", home())
}

fn settings_file() -> String {
    format!("{}/speedketchup.conf", app_dir())
}

fn login_plist() -> String {
    format!("{}/Library/LaunchAgents/{LABEL}.plist", home())
}

// the app keeps its results in the app directory, and a settings file to
// change the rest, the app gets no environment from a shell
pub fn app_defaults(mut config: SpeedTestConfig) -> SpeedTestConfig {
    config.store_filename = format!("{}/speedketchup-results.csv", app_dir());
    config
}

fn create_settings_file() {
    let _ = std::fs::create_dir_all(app_dir());
    let file = settings_file();
    if !std::path::Path::new(&file).exists() {
	let _ = std::fs::write(&file, SETTINGS);
    }
}

// the VARIABLE=value lines of the settings file
pub fn settings() -> Vec<(String, String)> {
    create_settings_file();
    let settings = std::fs::read_to_string(settings_file()).unwrap_or_default();
    let mut vars = vec!();
    for line in settings.lines() {
	let line = line.trim();
	if line.starts_with('#') {
	    continue;
	}
	if let Some((name, value)) = line.split_once('=') {
	    vars.push((name.trim().to_string(), value.trim().trim_matches('"').to_string()));
	}
    }
    vars
}

// what is wrong with the settings file, for the menu
static SETTINGS_ERROR: Mutex<Option<String>> = Mutex::new(None);

pub fn set_settings_error(error: Option<String>) {
    if let Ok(mut e) = SETTINGS_ERROR.lock() {
	*e = error;
    }
}

// the settings file changed, the environment still goes over it like at the
// start, a new address or port gets a new server and the old one stops
fn reload_settings(state: &Arc<Mutex<SpeedTestState>>) {
    let settings = settings();
    let var = |name: &str| std::env::var(name).ok().or_else(|| settings.iter().find(|(n, _)| n == name).map(|(_, v)| v.clone()));
    let mut config = app_defaults(default_config());
    let mut result = config_from_vars(&mut config, &var).and_then(|_| check_config(&config));
    let old = match state.lock() {
	Ok(state) => state.config.clone(),
	Err(_) => return,
    };
    let mut listener = None;
    if result.is_ok() && (&config.listen_address, config.listen_port) != (&old.listen_address, old.listen_port) {
	match std::net::TcpListener::bind((config.listen_address.as_str(), config.listen_port)) {
	    Ok(l) => listener = Some(l),
	    Err(e) => result = Err(format!("cannot listen on {}:{}: {e}", config.listen_address, config.listen_port)),
	}
    }
    if let Err(e) = &result {
	pr!("{}: {e}, keeping the previous settings", settings_file());
    }
    set_settings_error(result.as_ref().err().cloned());
    if result.is_err() || config == old {
	return;
    }
    if let Ok(mut state) = state.lock() {
	state.config = config;
	// wakes the test loop to wait for the new interval
	let _ = state.to_main_sender.send(());
    }
    if let Some(listener) = listener {
	start_server(listener, state);
	let ip = old.listen_address.parse().ok().filter(|ip: &std::net::IpAddr| !ip.is_unspecified());
	let ip = ip.unwrap_or(std::net::Ipv4Addr::LOCALHOST.into());
	let _ = std::net::TcpStream::connect_timeout(&(ip, old.listen_port).into(), Duration::from_secs(1));
    }
    pr!("reloaded {}", settings_file());
}

fn vnode_event(fd: i32, fflags: u32) -> libc::kevent {
    libc::kevent {
	ident: fd as usize,
	filter: libc::EVFILT_VNODE,
	flags: libc::EV_ADD | libc::EV_CLEAR,
	fflags: fflags,
	data: 0,
	udata: std::ptr::null_mut(),
    }
}

// applies the settings file whenever it changes, watched with kqueue: most
// editors save to a new file and rename it over the old one, so the
// directory is watched too and the file opened again after every change
pub fn watch_settings(state: Arc<Mutex<SpeedTestState>>) {
    let _ = std::thread::Builder::new().name("settings".to_string()).spawn(move || unsafe {
	let (dir, file) = match (std::ffi::CString::new(app_dir()), std::ffi::CString::new(settings_file())) {
	    (Ok(dir), Ok(file)) => (dir, file),
	    _ => return,
	};
	let kq = libc::kqueue();
	let dir_fd = libc::open(dir.as_ptr(), libc::O_EVTONLY | libc::O_CLOEXEC);
	if kq < 0 || dir_fd < 0 {
	    pr!("cannot watch {}", app_dir());
	    return;
	}
	let mut last = std::fs::read(settings_file()).ok();
	loop {
	    let file_fd = libc::open(file.as_ptr(), libc::O_EVTONLY | libc::O_CLOEXEC);
	    let mut changes = vec!(vnode_event(dir_fd, libc::NOTE_WRITE));
	    if file_fd >= 0 {
		changes.push(vnode_event(file_fd, libc::NOTE_WRITE | libc::NOTE_EXTEND | libc::NOTE_DELETE | libc::NOTE_RENAME));
	    }
	    let mut events: [libc::kevent; 2] = std::mem::zeroed();
	    let n = libc::kevent(kq, changes.as_ptr(), changes.len() as i32, events.as_mut_ptr(), 2, std::ptr::null());
	    // a save can be several writes and a rename, let it finish
	    std::thread::sleep(Duration::from_millis(200));
	    // closing it takes its event out of the queue too
	    if file_fd >= 0 {
		libc::close(file_fd);
	    }
	    if n < 0 {
		std::thread::sleep(Duration::from_secs(1));
		continue;
	    }
	    let now = std::fs::read(settings_file()).ok();
	    if now.is_some() && now != last {
		last = now;
		reload_settings(&state);
	    }
	}
    });
}

// the app itself as the login item, macOS 13 and later, None before
fn login_service() -> Option<Id> {
    unsafe {
	let class = objc_getClass(cs!("SMAppService"));
	match class.is_null() {
	    true => None,
	    false => Some(msg![Id; class, "mainAppService"]),
	}
    }
}

// in the App Sandbox, as from the Mac App Store, only SMAppService works
fn sandboxed() -> bool {
    std::env::var_os("APP_SANDBOX_CONTAINER_ID").is_some()
}

fn login_enabled() -> bool {
    match login_service() {
	// SMAppServiceStatusEnabled
	Some(service) if sandboxed() || !std::path::Path::new(&login_plist()).exists() => unsafe {
	    msg![isize; service, "status"] == 1
	},
	_ => std::path::Path::new(&login_plist()).exists(),
    }
}

fn set_login(enable: bool) {
    if let Some(service) = login_service() {
	let sel = match enable {
	    true => "registerAndReturnError:",
	    false => "unregisterAndReturnError:",
	};
	let ok = unsafe {
	    match enable {
		true => msg![bool; service, "registerAndReturnError:", std::ptr::null_mut::<Id>() => *mut Id],
		false => msg![bool; service, "unregisterAndReturnError:", std::ptr::null_mut::<Id>() => *mut Id],
	    }
	};
	if ok || sandboxed() {
	    if !ok {
		pr!("SMAppService {sel} failed");
	    }
	    let _ = std::fs::remove_file(login_plist());
	    return;
	}
	pr!("SMAppService {sel} failed, using a LaunchAgent");
    }
    match enable {
	true => write_login_plist(),
	false => {
	    let _ = std::fs::remove_file(login_plist());
	}
    }
}

fn xml_escape(str: &str) -> String {
    str.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

// a LaunchAgent starts the app at login, SPEEDKETCHUP_LOGIN keeps it from
// opening the web page then
fn write_login_plist() {
    let exe = std::env::current_exe().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
    let plist = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>Label</key>
	<string>{LABEL}</string>
	<key>ProgramArguments</key>
	<array>
		<string>{}</string>
	</array>
	<key>EnvironmentVariables</key>
	<dict>
		<key>SPEEDKETCHUP_LOGIN</key>
		<string>1</string>
	</dict>
	<key>RunAtLoad</key>
	<true/>
	<key>LimitLoadToSessionType</key>
	<string>Aqua</string>
	<key>ProcessType</key>
	<string>Interactive</string>
</dict>
</plist>
"#, xml_escape(&exe));
    let _ = std::fs::create_dir_all(format!("{}/Library/LaunchAgents", home()));
    if let Err(e) = std::fs::write(login_plist(), plist) {
	pr!("cannot write {}: {e}", login_plist());
    }
}

fn ago(secs: u64) -> String {
    match secs {
	0..=59 => "just now".to_string(),
	60..=3599 => format!("{} min ago", secs / 60),
	3600..=86399 => format!("{} h ago", secs / 3600),
	_ => format!("{} days ago", secs / 86400),
    }
}

fn speeds_line(download: Option<f32>, upload: Option<f32>, latency: Option<Duration>) -> String {
    let mut parts = vec!();
    if let Some(x) = download {
	parts.push(format!("↓ {}", float_to_str(x)));
    }
    if let Some(x) = upload {
	parts.push(format!("↑ {}", float_to_str(x)));
    }
    let mut line = parts.join("  ");
    if parts.len() > 0 {
	line += " Mbit/s";
    }
    if let Some(x) = latency {
	line += &format!("  ·  {} ms", dur_to_str(Some(x)));
    }
    line
}

// the two lines at the top of the menu, the result and when it was
fn status_lines(state: &Arc<Mutex<SpeedTestState>>) -> (String, String) {
    let state = match state.lock() {
	Ok(s) => s,
	Err(_) => return (String::new(), String::new()),
    };
    // while testing, the live gauges in place of the last result
    if state.status.starts_with("test") || state.status == "get provider info" {
	let phase = match state.status.as_str() {
	    "test latency" => "Testing latency…",
	    "test download" => "Testing download…",
	    "test wait" | "test upload" => "Testing upload…",
	    _ => "Finding a server…",
	};
	let line = speeds_line(state.gauge_download_mbps, state.gauge_upload_mbps, state.gauge_latency);
	return (if line.len() > 0 { line } else { "Testing…".to_string() }, phase.to_string());
    }
    let result = match &state.last_result {
	None => "No results yet".to_string(),
	Some(r) => speeds_line(r.download, r.upload, r.latency_idle),
    };
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
    let mut when = match (state.last_test_time, &state.last_error) {
	(None, _) => String::new(),
	(Some(t), None) => format!("Last test {}", ago(now.saturating_sub(t))),
	(Some(t), Some(e)) => format!("Last test failed {}: {e}", ago(now.saturating_sub(t))),
    };
    if let Some(until) = state.idle_until {
	// minutes:seconds like the web page, the open menu counts it down
	let secs = until.saturating_duration_since(Instant::now()).as_secs();
	when += &format!("{}next in {:02}:{:02}", if when.len() > 0 { ", " } else { "Next test " }, secs / 60, secs % 60);
    }
    (result, when)
}

unsafe fn ns_string(str: &str) -> Id {
    let alloc = msg![Id; objc_getClass(cs!("NSString")), "alloc"];
    msg![Id; alloc, "initWithBytes:length:encoding:", str.as_ptr() => *const u8, str.len() => usize, 4 => usize]
}

unsafe fn set_title(item: Id, title: &str) {
    let title = ns_string(title);
    msg![(); item, "setTitle:", title => Id];
    msg![(); title, "release"];
}

unsafe fn add_item(menu: Id, title: &str, action: Option<Sel>, target: Id, key: &str) -> Id {
    let alloc = msg![Id; objc_getClass(cs!("NSMenuItem")), "alloc"];
    let (title, key) = (ns_string(title), ns_string(key));
    let item = msg![Id; alloc, "initWithTitle:action:keyEquivalent:", title => Id, action.unwrap_or(std::ptr::null()) => Sel, key => Id];
    msg![(); title, "release"];
    msg![(); key, "release"];
    match action {
	Some(_) => msg![(); item, "setTarget:", target => Id],
	None => msg![(); item, "setEnabled:", false => bool],
    }
    msg![(); menu, "addItem:", item => Id];
    item
}

// the first line of the menu, the app name in bold, an explicit color keeps
// it from being grayed out as a disabled item
unsafe fn add_header(menu: Id, target: Id) {
    let item = add_item(menu, "SpeedKetchup", None, target, "");
    let font = msg![Id; objc_getClass(cs!("NSFont")), "boldSystemFontOfSize:", 0.0 => f64];
    let color = msg![Id; objc_getClass(cs!("NSColor")), "labelColor"];
    let (values, keys) = ([font, color], [NSFontAttributeName, NSForegroundColorAttributeName]);
    let attributes = msg![Id; objc_getClass(cs!("NSDictionary")), "dictionaryWithObjects:forKeys:count:",
			  values.as_ptr() => *const Id, keys.as_ptr() => *const Id, 2 => usize];
    let title = msg![Id; item, "title"];
    let alloc = msg![Id; objc_getClass(cs!("NSAttributedString")), "alloc"];
    let bold = msg![Id; alloc, "initWithString:attributes:", title => Id, attributes => Id];
    msg![(); item, "setAttributedTitle:", bold => Id];
    msg![(); bold, "release"];
}

unsafe fn add_separator(menu: Id) {
    let item = msg![Id; objc_getClass(cs!("NSMenuItem")), "separatorItem"];
    msg![(); menu, "addItem:", item => Id];
}

extern "C" fn open_dashboard(_this: Id, _cmd: Sel, _sender: Id) {
    if let Some(m) = MENU.get() {
	let port = m.state.lock().map(|s| s.config.listen_port).unwrap_or_default();
	open_url(&format!("http://127.0.0.1:{port}"));
    }
}

// NSApplicationDelegate, opening the app again while it runs, in the Finder
// or with open, comes here instead of starting another one
extern "C" fn should_handle_reopen(this: Id, cmd: Sel, app: Id, _has_visible_windows: bool) -> bool {
    open_dashboard(this, cmd, app);
    false
}

extern "C" fn run_test(_this: Id, _cmd: Sel, _sender: Id) {
    if let Some(m) = MENU.get() {
	request_test(&m.state);
    }
}

extern "C" fn toggle_login(_this: Id, _cmd: Sel, _sender: Id) {
    set_login(!login_enabled());
}

// in TextEdit, NSWorkspace lets it open the file in the sandbox container
extern "C" fn edit_settings(_this: Id, _cmd: Sel, _sender: Id) {
    create_settings_file();
    unsafe {
	let workspace = msg![Id; objc_getClass(cs!("NSWorkspace")), "sharedWorkspace"];
	let (path, bundle) = (ns_string(&settings_file()), ns_string("com.apple.TextEdit"));
	let url = msg![Id; objc_getClass(cs!("NSURL")), "fileURLWithPath:", path => Id];
	let textedit = msg![Id; workspace, "URLForApplicationWithBundleIdentifier:", bundle => Id];
	let urls = msg![Id; objc_getClass(cs!("NSArray")), "arrayWithObject:", url => Id];
	let config = msg![Id; objc_getClass(cs!("NSWorkspaceOpenConfiguration")), "configuration"];
	if textedit.is_null() {
	    msg![bool; workspace, "openURL:", url => Id];
	} else {
	    msg![(); workspace, "openURLs:withApplicationAtURL:configuration:completionHandler:",
		 urls => Id, textedit => Id, config => Id, std::ptr::null_mut::<c_void>() => Id];
	}
	msg![(); path, "release"];
	msg![(); bundle, "release"];
    }
}

// the web page in the default browser, NSWorkspace works in the sandbox too
pub fn open_url(url: &str) {
    unsafe {
	let str = ns_string(url);
	let url = msg![Id; objc_getClass(cs!("NSURL")), "URLWithString:", str => Id];
	if !url.is_null() {
	    msg![bool; msg![Id; objc_getClass(cs!("NSWorkspace")), "sharedWorkspace"], "openURL:", url => Id];
	}
	msg![(); str, "release"];
    }
}

// NSApplicationDelegate, opens the web page unless started at login, by the
// login item or by the LaunchAgent with SPEEDKETCHUP_LOGIN
extern "C" fn will_finish_launching(this: Id, cmd: Sel, notification: Id) {
    let mut at_login = std::env::var_os("SPEEDKETCHUP_LOGIN").is_some();
    unsafe {
	let manager = msg![Id; objc_getClass(cs!("NSAppleEventManager")), "sharedAppleEventManager"];
	let event = msg![Id; manager, "currentAppleEvent"];
	if !event.is_null() {
	    // keyAEPropData is keyAELaunchedAsLogInItem
	    let data = msg![Id; event, "paramDescriptorForKeyword:", u32::from_be_bytes(*b"prdt") => u32];
	    at_login |= !data.is_null() && msg![u32; data, "enumCodeValue"] == u32::from_be_bytes(*b"lgit");
	}
    }
    if !at_login {
	open_dashboard(this, cmd, notification);
    }
}

// NSTimer refreshing the open menu every second, main thread only
static TIMER: AtomicUsize = AtomicUsize::new(0);

// NSMenuDelegate, the open menu tracks events in its own run loop mode, a
// timer in the common modes keeps firing there
extern "C" fn menu_will_open(this: Id, _cmd: Sel, _menu: Id) {
    unsafe {
	let timer = msg![Id; objc_getClass(cs!("NSTimer")), "timerWithTimeInterval:target:selector:userInfo:repeats:",
			 1.0 => f64, this => Id, sel_registerName(cs!("menuNeedsUpdate:")) => Sel, std::ptr::null_mut() => Id, true => bool];
	let run_loop = msg![Id; objc_getClass(cs!("NSRunLoop")), "mainRunLoop"];
	msg![(); run_loop, "addTimer:forMode:", timer => Id, NSRunLoopCommonModes => Id];
	TIMER.store(timer as usize, Ordering::Relaxed);
    }
}

extern "C" fn menu_did_close(_this: Id, _cmd: Sel, _menu: Id) {
    let timer = TIMER.swap(0, Ordering::Relaxed) as Id;
    if !timer.is_null() {
	unsafe { msg![(); timer, "invalidate"] };
    }
}

// NSMenuDelegate, fills in the menu right before it shows, and the timer
// calls it while the menu is open
extern "C" fn menu_needs_update(_this: Id, _cmd: Sel, _menu: Id) {
    let m = match MENU.get() {
	Some(m) => m,
	None => return,
    };
    let (result, when) = status_lines(&m.state);
    unsafe {
	set_title(m.result_item as Id, &result);
	set_title(m.time_item as Id, &when);
	let error = SETTINGS_ERROR.lock().ok().and_then(|e| e.clone());
	set_title(m.error_item as Id, &format!("Settings: {}", error.as_deref().unwrap_or("")));
	msg![(); m.error_item as Id, "setHidden:", error.is_none() => bool];
	msg![(); m.login_item as Id, "setState:", login_enabled() as isize => isize];
    }
}

unsafe fn target_class() -> Id {
    let class = objc_allocateClassPair(objc_getClass(cs!("NSObject")), cs!("SpeedKetchupMenuTarget"), 0);
    for (name, imp) in [
	(cs!("openDashboard:"), open_dashboard as *const c_void),
	(cs!("runTest:"), run_test as *const c_void),
	(cs!("toggleLogin:"), toggle_login as *const c_void),
	(cs!("editSettings:"), edit_settings as *const c_void),
	(cs!("menuNeedsUpdate:"), menu_needs_update as *const c_void),
	(cs!("menuWillOpen:"), menu_will_open as *const c_void),
	(cs!("menuDidClose:"), menu_did_close as *const c_void),
    ] {
	class_addMethod(class, sel_registerName(name), imp, cs!("v@:@"));
    }
    class_addMethod(class, sel_registerName(cs!("applicationWillFinishLaunching:")),
		    will_finish_launching as *const c_void, cs!("v@:@"));
    class_addMethod(class, sel_registerName(cs!("applicationShouldHandleReopen:hasVisibleWindows:")),
		    should_handle_reopen as *const c_void, cs!("c@:@c"));
    objc_registerClassPair(class);
    class
}

// runs the AppKit event loop on the main thread, never returns
pub fn run(state: Arc<Mutex<SpeedTestState>>) -> ! {
    // a LaunchAgent of an earlier version moves to SMAppService, or points at
    // wherever the app is now, it may have moved
    if std::path::Path::new(&login_plist()).exists() {
	set_login(true);
    }
    unsafe {
	let app = msg![Id; objc_getClass(cs!("NSApplication")), "sharedApplication"];
	// an accessory app has no Dock icon and no menu of its own
	msg![(); app, "setActivationPolicy:", 1 => isize];
	let target = msg![Id; msg![Id; target_class(), "alloc"], "init"];
	msg![(); app, "setDelegate:", target => Id];
	let sel = |name: *const c_char| Some(sel_registerName(name));

	let menu = msg![Id; msg![Id; objc_getClass(cs!("NSMenu")), "alloc"], "init"];
	msg![(); menu, "setAutoenablesItems:", false => bool];
	msg![(); menu, "setDelegate:", target => Id];
	add_header(menu, target);
	add_separator(menu);
	let result_item = add_item(menu, "", None, target, "");
	let time_item = add_item(menu, "", None, target, "");
	let error_item = add_item(menu, "", None, target, "");
	add_separator(menu);
	add_item(menu, "Open Dashboard", sel(cs!("openDashboard:")), target, "o");
	add_item(menu, "Run Test Now", sel(cs!("runTest:")), target, "r");
	add_separator(menu);
	let login_item = add_item(menu, "Start at Login", sel(cs!("toggleLogin:")), target, "");
	// sandboxed before macOS 13 there is no way to start at login
	msg![(); login_item, "setHidden:", sandboxed() && login_service().is_none() => bool];
	add_item(menu, "Edit Settings…", sel(cs!("editSettings:")), target, ",");
	add_separator(menu);
	add_item(menu, "Quit SpeedKetchup", sel(cs!("terminate:")), app, "q");

	let _ = MENU.set(Menu {
	    state: state,
	    result_item: result_item as usize,
	    time_item: time_item as usize,
	    error_item: error_item as usize,
	    login_item: login_item as usize,
	});

	let bar = msg![Id; objc_getClass(cs!("NSStatusBar")), "systemStatusBar"];
	// NSVariableStatusItemLength
	let item = msg![Id; bar, "statusItemWithLength:", -1.0 => f64];
	msg![Id; item, "retain"];
	let button = msg![Id; item, "button"];
	let (symbol, description) = (ns_string("speedometer"), ns_string("SpeedKetchup"));
	let image = msg![Id; objc_getClass(cs!("NSImage")), "imageWithSystemSymbolName:accessibilityDescription:",
			 symbol => Id, description => Id];
	if image.is_null() {
	    set_title(button, "SK");
	} else {
	    msg![(); image, "setTemplate:", true => bool];
	    msg![(); button, "setImage:", image => Id];
	}
	let tip = ns_string("SpeedKetchup");
	msg![(); button, "setToolTip:", tip => Id];
	msg![(); tip, "release"];
	msg![(); item, "setMenu:", menu => Id];

	msg![(); app, "run"];
    }
    std::process::exit(0)
}
