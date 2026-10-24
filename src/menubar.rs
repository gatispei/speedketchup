// The macOS menu bar icon, AppKit called through the Objective-C runtime.

use super::*;
use super::app::*;
use std::path::PathBuf;
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

struct Menu {
    state: Arc<Mutex<SpeedTestState>>,
    // NSMenuItem pointers, AppKit only touches them on the main thread
    result_item: usize,
    time_item: usize,
    error_item: usize,
    dashboard_item: usize,
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

pub fn app_dir() -> PathBuf {
    PathBuf::from(home()).join("Library/Application Support/SpeedKetchup")
}

fn login_plist() -> String {
    format!("{}/Library/LaunchAgents/{LABEL}.plist", home())
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
	let (dir, file) = match (std::ffi::CString::new(app_dir().to_string_lossy().as_bytes()), std::ffi::CString::new(settings_file().to_string_lossy().as_bytes())) {
	    (Ok(dir), Ok(file)) => (dir, file),
	    _ => return,
	};
	let kq = libc::kqueue();
	let dir_fd = libc::open(dir.as_ptr(), libc::O_EVTONLY | libc::O_CLOEXEC);
	if kq < 0 || dir_fd < 0 {
	    pr!("cannot watch {}", app_dir().display());
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
	    settings_changed(&state, &mut last);
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
    let item = add_item(menu, "", None, target, "");
    let font = msg![Id; objc_getClass(cs!("NSFont")), "boldSystemFontOfSize:", 0.0 => f64];
    set_styled_title(item, "SpeedKetchup", font, msg![Id; objc_getClass(cs!("NSColor")), "labelColor"]);
}

// a title in font and color, a disabled item keeps the color instead of
// being grayed out
unsafe fn set_styled_title(item: Id, title: &str, font: Id, color: Id) {
    let (values, keys) = ([font, color], [NSFontAttributeName, NSForegroundColorAttributeName]);
    let attributes = msg![Id; objc_getClass(cs!("NSDictionary")), "dictionaryWithObjects:forKeys:count:",
			  values.as_ptr() => *const Id, keys.as_ptr() => *const Id, 2 => usize];
    let title = ns_string(title);
    let alloc = msg![Id; objc_getClass(cs!("NSAttributedString")), "alloc"];
    let styled = msg![Id; alloc, "initWithString:attributes:", title => Id, attributes => Id];
    msg![(); item, "setAttributedTitle:", styled => Id];
    msg![(); styled, "release"];
    msg![(); title, "release"];
}

unsafe fn add_separator(menu: Id) {
    let item = msg![Id; objc_getClass(cs!("NSMenuItem")), "separatorItem"];
    msg![(); menu, "addItem:", item => Id];
}

// the settings instead while another program has the port
extern "C" fn open_dashboard(this: Id, cmd: Sel, sender: Id) {
    if let Some(m) = MENU.get() {
	match dashboard_url(&m.state) {
	    Some(url) => open_url(&url),
	    None => edit_settings(this, cmd, sender),
	}
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
	let (path, bundle) = (ns_string(&settings_file().to_string_lossy()), ns_string("com.apple.TextEdit"));
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
    // without the web page the menu says why
    let listening = MENU.get().map(|m| listening(&m.state)).unwrap_or(false);
    if !at_login && listening {
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
	let error = error_line();
	let font = msg![Id; objc_getClass(cs!("NSFont")), "menuFontOfSize:", 0.0 => f64];
	set_styled_title(m.error_item as Id, error.as_deref().unwrap_or(""), font, msg![Id; objc_getClass(cs!("NSColor")), "systemRedColor"]);
	msg![(); m.error_item as Id, "setHidden:", error.is_none() => bool];
	// without the web page there is no dashboard to open
	msg![(); m.dashboard_item as Id, "setEnabled:", listening(&m.state) => bool];
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
	let dashboard_item = add_item(menu, "Open Dashboard", sel(cs!("openDashboard:")), target, "o");
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
	    dashboard_item: dashboard_item as usize,
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
