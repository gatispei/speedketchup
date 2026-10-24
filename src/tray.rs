// The windows tray icon, Win32 and WinRT called directly.

use super::*;
use super::app::*;
use std::ffi::c_void;
use std::path::PathBuf;
use std::sync::OnceLock;

type Handle = *mut c_void;

#[repr(C)]
struct Guid(u32, u16, u16, [u8; 8]);

#[repr(C)]
struct Point {
    x: i32,
    y: i32,
}

#[repr(C)]
struct Msg {
    hwnd: Handle,
    message: u32,
    wparam: usize,
    lparam: isize,
    time: u32,
    pt: Point,
    private: u32,
}

#[repr(C)]
struct WndClassEx {
    size: u32,
    style: u32,
    wndproc: extern "system" fn(Handle, u32, usize, isize) -> isize,
    cls_extra: i32,
    wnd_extra: i32,
    instance: Handle,
    icon: Handle,
    cursor: Handle,
    background: Handle,
    menu_name: *const u16,
    class_name: *const u16,
    icon_small: Handle,
}

#[repr(C)]
struct NotifyIconData {
    size: u32,
    hwnd: Handle,
    id: u32,
    flags: u32,
    callback_message: u32,
    icon: Handle,
    tip: [u16; 128],
    state: u32,
    state_mask: u32,
    info: [u16; 256],
    version: u32,
    info_title: [u16; 64],
    info_flags: u32,
    guid: Guid,
    balloon_icon: Handle,
}

#[repr(C)]
struct MenuItemInfo {
    size: u32,
    mask: u32,
    type_: u32,
    state: u32,
    id: u32,
    submenu: Handle,
    bmp_checked: Handle,
    bmp_unchecked: Handle,
    item_data: usize,
    type_data: *mut u16,
    cch: u32,
    bmp_item: Handle,
}

#[repr(C)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[repr(C)]
struct MeasureItem {
    ctl_type: u32,
    ctl_id: u32,
    item_id: u32,
    item_width: u32,
    item_height: u32,
    item_data: usize,
}

#[repr(C)]
struct DrawItem {
    ctl_type: u32,
    ctl_id: u32,
    item_id: u32,
    item_action: u32,
    item_state: u32,
    hwnd_item: Handle,
    hdc: Handle,
    rect: Rect,
    item_data: usize,
}

#[link(name = "user32")]
extern "system" {
    fn RegisterClassExW(class: *const WndClassEx) -> u16;
    fn CreateWindowExW(ex_style: u32, class: *const u16, name: *const u16, style: u32, x: i32, y: i32, w: i32, h: i32,
		       parent: Handle, menu: Handle, instance: Handle, param: *mut c_void) -> Handle;
    fn DefWindowProcW(hwnd: Handle, msg: u32, wparam: usize, lparam: isize) -> isize;
    fn GetMessageW(msg: *mut Msg, hwnd: Handle, min: u32, max: u32) -> i32;
    fn TranslateMessage(msg: *const Msg) -> i32;
    fn DispatchMessageW(msg: *const Msg) -> isize;
    fn PostMessageW(hwnd: Handle, msg: u32, wparam: usize, lparam: isize) -> i32;
    fn RegisterWindowMessageW(name: *const u16) -> u32;
    fn FindWindowExW(parent: Handle, after: Handle, class: *const u16, name: *const u16) -> Handle;
    fn FindWindowW(class: *const u16, name: *const u16) -> Handle;
    fn SetTimer(hwnd: Handle, id: usize, ms: u32, func: *const c_void) -> usize;
    fn CreatePopupMenu() -> Handle;
    fn AppendMenuW(menu: Handle, flags: u32, id: usize, text: *const u16) -> i32;
    fn SetMenuItemInfoW(menu: Handle, item: u32, by_position: i32, info: *const MenuItemInfo) -> i32;
    fn SetMenuDefaultItem(menu: Handle, item: u32, by_position: u32) -> i32;
    fn TrackPopupMenuEx(menu: Handle, flags: u32, x: i32, y: i32, hwnd: Handle, params: *const c_void) -> i32;
    fn DestroyMenu(menu: Handle) -> i32;
    fn SetForegroundWindow(hwnd: Handle) -> i32;
    fn GetCursorPos(point: *mut Point) -> i32;
    fn InvalidateRect(hwnd: Handle, rect: *const c_void, erase: i32) -> i32;
    fn GetSystemMetrics(index: i32) -> i32;
    fn CreateIconFromResourceEx(bits: *const u8, size: u32, icon: i32, version: u32, cx: i32, cy: i32, flags: u32) -> Handle;
}

#[link(name = "gdi32")]
extern "system" {
    fn CreateSolidBrush(color: u32) -> Handle;
    fn GetStockObject(object: i32) -> Handle;
    fn SelectObject(hdc: Handle, object: Handle) -> Handle;
    fn DeleteObject(object: Handle) -> i32;
    fn Ellipse(hdc: Handle, left: i32, top: i32, right: i32, bottom: i32) -> i32;
}

#[link(name = "shell32")]
extern "system" {
    fn Shell_NotifyIconW(message: u32, data: *const NotifyIconData) -> i32;
    fn ShellExecuteW(hwnd: Handle, op: *const u16, file: *const u16, params: *const u16, dir: *const u16, show: i32) -> Handle;
}

#[link(name = "kernel32")]
extern "system" {
    fn GetModuleHandleW(name: *const u16) -> Handle;
    fn CreateMutexW(attributes: *const c_void, owner: i32, name: *const u16) -> Handle;
    fn GetLastError() -> u32;
    fn FindFirstChangeNotificationW(path: *const u16, subtree: i32, filter: u32) -> Handle;
    fn FindNextChangeNotification(handle: Handle) -> i32;
    fn WaitForSingleObject(handle: Handle, ms: u32) -> u32;
    fn GetCurrentPackageFamilyName(length: *mut u32, name: *mut u16) -> i32;
    fn LoadLibraryW(name: *const u16) -> Handle;
    fn GetProcAddress(module: Handle, name: *const u8) -> *const c_void;
    fn AttachConsole(process: u32) -> i32;
    fn GetStdHandle(id: u32) -> Handle;
    fn SetStdHandle(id: u32, handle: Handle) -> i32;
    fn CreateFileW(name: *const u16, access: u32, share: u32, security: *const c_void, disposition: u32, flags: u32, template: Handle) -> Handle;
    fn GetConsoleMode(handle: Handle, mode: *mut u32) -> i32;
    fn SetConsoleMode(handle: Handle, mode: u32) -> i32;
}

#[link(name = "advapi32")]
extern "system" {
    fn RegSetKeyValueW(key: isize, subkey: *const u16, name: *const u16, type_: u32, data: *const c_void, size: u32) -> i32;
    fn RegDeleteKeyValueW(key: isize, subkey: *const u16, name: *const u16) -> i32;
    fn RegGetValueW(key: isize, subkey: *const u16, name: *const u16, flags: u32, type_: *mut u32, data: *mut c_void, size: *mut u32) -> i32;
}

const WM_DESTROY: u32 = 0x0002;
const WM_COMMAND: u32 = 0x0111;
const WM_DRAWITEM: u32 = 0x002b;
const WM_MEASUREITEM: u32 = 0x002c;
const WM_TIMER: u32 = 0x0113;
const WM_CONTEXTMENU: u32 = 0x007b;
const WM_APP: u32 = 0x8000;
const WM_TRAY: u32 = WM_APP + 1;
const WM_OPEN_DASHBOARD: u32 = WM_APP + 2;
const NIN_SELECT: u32 = 0x0400;
const NIN_KEYSELECT: u32 = 0x0401;
const NIM_ADD: u32 = 0;
const NIM_MODIFY: u32 = 1;
const NIM_DELETE: u32 = 2;
const NIM_SETVERSION: u32 = 4;
const NIF_MESSAGE: u32 = 0x01;
const NIF_ICON: u32 = 0x02;
const NIF_TIP: u32 = 0x04;
const NIF_SHOWTIP: u32 = 0x80;
const MF_STRING: u32 = 0x0000;
const MF_GRAYED: u32 = 0x0001;
const MF_CHECKED: u32 = 0x0008;
const MF_SEPARATOR: u32 = 0x0800;
const MIIM_STRING: u32 = 0x0040;
const MIIM_BITMAP: u32 = 0x0080;
const TPM_RIGHTBUTTON: u32 = 0x0002;
const TPM_RETURNCMD: u32 = 0x0100;
const SM_CXSMICON: i32 = 49;
const SW_SHOWNORMAL: i32 = 1;
const HKEY_CURRENT_USER: isize = 0x80000001u32 as i32 as isize;
const REG_SZ: u32 = 1;
const RRF_RT_REG_SZ: u32 = 0x02;
const ERROR_ALREADY_EXISTS: u32 = 183;
const FILE_NOTIFY_CHANGE_FILE_NAME: u32 = 0x01;
const FILE_NOTIFY_CHANGE_LAST_WRITE: u32 = 0x10;
const INFINITE: u32 = 0xffffffff;
const HBMMENU_CALLBACK: isize = -1;
const NULL_PEN: i32 = 8;
const STD_INPUT_HANDLE: u32 = -10i32 as u32;
const STD_OUTPUT_HANDLE: u32 = -11i32 as u32;
const STD_ERROR_HANDLE: u32 = -12i32 as u32;
const GENERIC_READ: u32 = 0x80000000;
const GENERIC_WRITE: u32 = 0x40000000;
const FILE_SHARE_READ: u32 = 1;
const FILE_SHARE_WRITE: u32 = 2;
const OPEN_EXISTING: u32 = 3;
const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 0x0004;

const ID_HEADER: usize = 1;
const ID_RESULT: usize = 2;
const ID_TIME: usize = 3;
const ID_ERROR: usize = 4;
const ID_DASHBOARD: usize = 10;
const ID_TEST: usize = 11;
const ID_LOGIN: usize = 12;
const ID_SETTINGS: usize = 13;
const ID_QUIT: usize = 14;

const CLASS: &str = "SpeedKetchupTray";
const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const STARTUP_TASK: &str = "SpeedKetchupStartup";

const ICON: &[u8] = include_bytes!("../windows/SpeedKetchup.ico");

fn wide(str: &str) -> Vec<u16> {
    str.encode_utf16().chain(std::iter::once(0)).collect()
}

// the parent terminal's console, for the handles that are not redirected
fn attach_console() -> bool {
    unsafe {
	// ATTACH_PARENT_PROCESS
	if AttachConsole(0xffffffff) == 0 {
	    return false;
	}
	for (id, name) in [(STD_INPUT_HANDLE, "CONIN$"), (STD_OUTPUT_HANDLE, "CONOUT$"), (STD_ERROR_HANDLE, "CONOUT$")] {
	    let mut handle = GetStdHandle(id);
	    if handle.is_null() || handle as isize == -1 {
		handle = CreateFileW(wide(name).as_ptr(), GENERIC_READ | GENERIC_WRITE, FILE_SHARE_READ | FILE_SHARE_WRITE,
				     std::ptr::null(), OPEN_EXISTING, 0, std::ptr::null_mut());
		SetStdHandle(id, handle);
	    }
	    let mut mode = 0;
	    if id != STD_INPUT_HANDLE && GetConsoleMode(handle, &mut mode) != 0 {
		SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
	    }
	}
    }
    true
}

// the tray icon without a terminal and options, or with --tray, one at a time
pub fn is_app() -> bool {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let console = attach_console();
    let tray = args.iter().any(|a| a == "--tray" || a == "--login") || (!console && args.is_empty());
    if !tray {
	return false;
    }
    unsafe {
	let user32 = LoadLibraryW(wide("user32.dll").as_ptr());
	let set_awareness = GetProcAddress(user32, b"SetProcessDpiAwarenessContext\0".as_ptr());
	if !set_awareness.is_null() {
	    let set_awareness: extern "system" fn(isize) -> i32 = std::mem::transmute(set_awareness);
	    // DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2
	    set_awareness(-4);
	}
	CreateMutexW(std::ptr::null(), 0, wide("Local\\SpeedKetchup").as_ptr());
	if GetLastError() == ERROR_ALREADY_EXISTS {
	    // HWND_MESSAGE
	    let hwnd = FindWindowExW(-3isize as Handle, std::ptr::null_mut(), wide(CLASS).as_ptr(), std::ptr::null());
	    if !hwnd.is_null() {
		PostMessageW(hwnd, WM_OPEN_DASHBOARD, 0, 0);
	    }
	    std::process::exit(0);
	}
    }
    true
}

fn package_family_name() -> Option<String> {
    let mut name = [0u16; 256];
    let mut length = name.len() as u32;
    match unsafe { GetCurrentPackageFamilyName(&mut length, name.as_mut_ptr()) } {
	0 => Some(String::from_utf16_lossy(&name[..length.saturating_sub(1) as usize])),
	_ => None,
    }
}

// a package's AppData is redirected, notepad needs the real path
pub fn app_dir() -> PathBuf {
    let env = |name| PathBuf::from(std::env::var_os(name).unwrap_or_default());
    match package_family_name() {
	Some(family) => env("LOCALAPPDATA").join("Packages").join(family).join("LocalCache\\Roaming\\SpeedKetchup"),
	None => env("APPDATA").join("SpeedKetchup"),
    }
}

pub fn open_url(url: &str) {
    unsafe {
	ShellExecuteW(std::ptr::null_mut(), wide("open").as_ptr(), wide(url).as_ptr(), std::ptr::null(), std::ptr::null(), SW_SHOWNORMAL);
    }
}

fn edit_settings() {
    create_settings_file();
    let file = format!("\"{}\"", settings_file().display());
    unsafe {
	ShellExecuteW(std::ptr::null_mut(), wide("open").as_ptr(), wide("notepad.exe").as_ptr(), wide(&file).as_ptr(), std::ptr::null(), SW_SHOWNORMAL);
    }
}

// the directory, as editors may save by renaming
pub fn watch_settings(state: Arc<Mutex<SpeedTestState>>) {
    let _ = std::thread::Builder::new().name("settings".to_string()).spawn(move || unsafe {
	let dir = wide(&app_dir().to_string_lossy());
	let handle = FindFirstChangeNotificationW(dir.as_ptr(), 0, FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_LAST_WRITE);
	// INVALID_HANDLE_VALUE
	if handle as isize == -1 {
	    pr!("cannot watch {}", app_dir().display());
	    return;
	}
	let mut last = std::fs::read(settings_file()).ok();
	loop {
	    WaitForSingleObject(handle, INFINITE);
	    std::thread::sleep(Duration::from_millis(200));
	    settings_changed(&state, &mut last);
	    if FindNextChangeNotification(handle) == 0 {
		return;
	    }
	}
    });
}

// combase is loaded at run time, it is not there before windows 10
mod winrt {
    use super::*;

    type HString = *mut c_void;

    pub const ISTARTUP_TASK_STATICS: Guid = Guid(0xee5b60bd, 0xa148, 0x41a7, [0xb2, 0x6e, 0xe8, 0xb8, 0x8a, 0x1e, 0x62, 0xf8]);
    pub const IAPP_INSTANCE_STATICS: Guid = Guid(0x9d11e77f, 0x9ea6, 0x47af, [0xa6, 0xec, 0x46, 0x78, 0x4c, 0x5b, 0xa2, 0x54]);
    const IASYNC_INFO: Guid = Guid(0x00000036, 0x0000, 0x0000, [0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46]);

    pub struct Combase {
	init: extern "system" fn(i32) -> i32,
	create_string: extern "system" fn(*const u16, u32, *mut HString) -> i32,
	delete_string: extern "system" fn(HString) -> i32,
	get_factory: extern "system" fn(HString, *const Guid, *mut Handle) -> i32,
    }

    fn combase() -> Option<&'static Combase> {
	static COMBASE: OnceLock<Option<Combase>> = OnceLock::new();
	COMBASE.get_or_init(|| unsafe {
	    let module = LoadLibraryW(wide("combase.dll").as_ptr());
	    if module.is_null() {
		return None;
	    }
	    let get = |name: &[u8]| GetProcAddress(module, name.as_ptr());
	    let functions = [get(b"RoInitialize\0"), get(b"WindowsCreateString\0"), get(b"WindowsDeleteString\0"), get(b"RoGetActivationFactory\0")];
	    if functions.iter().any(|f| f.is_null()) {
		return None;
	    }
	    Some(Combase {
		init: std::mem::transmute(functions[0]),
		create_string: std::mem::transmute(functions[1]),
		delete_string: std::mem::transmute(functions[2]),
		get_factory: std::mem::transmute(functions[3]),
	    })
	}).as_ref()
    }

    pub unsafe fn method<F: Copy>(object: Handle, slot: usize) -> F {
	let vtable = *(object as *const *const usize);
	std::mem::transmute_copy(&*vtable.add(slot))
    }

    pub unsafe fn release(object: Handle) {
	if !object.is_null() {
	    method::<extern "system" fn(Handle) -> u32>(object, 2)(object);
	}
    }

    pub fn run<T: Send + 'static>(f: impl FnOnce(&'static Combase) -> Option<T> + Send + 'static) -> Option<T> {
	let combase = combase()?;
	std::thread::spawn(move || {
	    // RO_INIT_MULTITHREADED
	    (combase.init)(1);
	    f(combase)
	}).join().ok().flatten()
    }

    pub unsafe fn factory(combase: &Combase, class: &str, iid: &Guid) -> Option<Handle> {
	let name = wide(class);
	let mut hstring = std::ptr::null_mut();
	(combase.create_string)(name.as_ptr(), (name.len() - 1) as u32, &mut hstring);
	let mut factory = std::ptr::null_mut();
	let hr = (combase.get_factory)(hstring, iid, &mut factory);
	(combase.delete_string)(hstring);
	if hr < 0 || factory.is_null() { None } else { Some(factory) }
    }

    pub unsafe fn hstring(combase: &Combase, str: &str) -> HString {
	let str = wide(str);
	let mut hstring = std::ptr::null_mut();
	(combase.create_string)(str.as_ptr(), (str.len() - 1) as u32, &mut hstring);
	hstring
    }

    pub unsafe fn delete(combase: &Combase, hstring: HString) {
	(combase.delete_string)(hstring);
    }

    pub unsafe fn wait(operation: Handle) -> bool {
	let mut info = std::ptr::null_mut();
	let query: extern "system" fn(Handle, *const Guid, *mut Handle) -> i32 = method(operation, 0);
	if query(operation, &IASYNC_INFO, &mut info) < 0 {
	    return false;
	}
	let mut status = 0;
	for _ in 0..500 {
	    method::<extern "system" fn(Handle, *mut i32) -> i32>(info, 7)(info, &mut status);
	    if status != 0 {
		break;
	    }
	    std::thread::sleep(Duration::from_millis(10));
	}
	release(info);
	// AsyncStatus Completed
	status == 1
    }
}

fn with_startup_task<T: Send + 'static>(f: impl FnOnce(Handle) -> Option<T> + Send + 'static) -> Option<T> {
    package_family_name()?;
    winrt::run(move |combase| unsafe {
	let statics = winrt::factory(combase, "Windows.ApplicationModel.StartupTask", &winrt::ISTARTUP_TASK_STATICS)?;
	let id = winrt::hstring(combase, STARTUP_TASK);
	let mut operation = std::ptr::null_mut();
	winrt::method::<extern "system" fn(Handle, *mut c_void, *mut Handle) -> i32>(statics, 7)(statics, id, &mut operation);
	winrt::delete(combase, id);
	winrt::release(statics);
	if operation.is_null() {
	    return None;
	}
	let mut task = std::ptr::null_mut();
	if winrt::wait(operation) {
	    winrt::method::<extern "system" fn(Handle, *mut Handle) -> i32>(operation, 8)(operation, &mut task);
	}
	winrt::release(operation);
	if task.is_null() {
	    return None;
	}
	let result = f(task);
	winrt::release(task);
	result
    })
}

fn startup_task_state() -> Option<i32> {
    with_startup_task(|task| unsafe {
	let mut state = 0;
	winrt::method::<extern "system" fn(Handle, *mut i32) -> i32>(task, 8)(task, &mut state);
	Some(state)
    })
}

fn login_enabled() -> bool {
    if package_family_name().is_some() {
	return matches!(startup_task_state(), Some(2) | Some(4));
    }
    let mut size = 0;
    unsafe {
	RegGetValueW(HKEY_CURRENT_USER, wide(RUN_KEY).as_ptr(), wide("SpeedKetchup").as_ptr(), RRF_RT_REG_SZ,
		     std::ptr::null_mut(), std::ptr::null_mut(), &mut size) == 0
    }
}

fn set_login(enable: bool) {
    if package_family_name().is_some() {
	// only the user can turn it on again after turning it off in settings
	if enable && startup_task_state() == Some(1) {
	    open_url("ms-settings:startupapps");
	    return;
	}
	with_startup_task(move |task| unsafe {
	    match enable {
		true => {
		    let mut operation = std::ptr::null_mut();
		    winrt::method::<extern "system" fn(Handle, *mut Handle) -> i32>(task, 6)(task, &mut operation);
		    if !operation.is_null() {
			winrt::wait(operation);
			winrt::release(operation);
		    }
		},
		false => {
		    winrt::method::<extern "system" fn(Handle) -> i32>(task, 7)(task);
		},
	    }
	    Some(())
	});
	return;
    }
    unsafe {
	let (key, name) = (wide(RUN_KEY), wide("SpeedKetchup"));
	if enable {
	    let exe = std::env::current_exe().map(|p| p.display().to_string()).unwrap_or_default();
	    let command = wide(&format!("\"{exe}\" --login"));
	    RegSetKeyValueW(HKEY_CURRENT_USER, key.as_ptr(), name.as_ptr(), REG_SZ, command.as_ptr() as *const c_void, (command.len() * 2) as u32);
	} else {
	    RegDeleteKeyValueW(HKEY_CURRENT_USER, key.as_ptr(), name.as_ptr());
	}
    }
}

pub fn at_login() -> bool {
    if std::env::args().any(|a| a == "--login") {
	return true;
    }
    package_family_name().is_some() && winrt::run(|combase| unsafe {
	let statics = winrt::factory(combase, "Windows.ApplicationModel.AppInstance", &winrt::IAPP_INSTANCE_STATICS)?;
	let mut args = std::ptr::null_mut();
	winrt::method::<extern "system" fn(Handle, *mut Handle) -> i32>(statics, 7)(statics, &mut args);
	winrt::release(statics);
	if args.is_null() {
	    return None;
	}
	let mut kind = 0;
	winrt::method::<extern "system" fn(Handle, *mut i32) -> i32>(args, 6)(args, &mut kind);
	winrt::release(args);
	// ActivationKind StartupTask
	Some(kind == 1020)
    }).unwrap_or(false)
}

fn load_icon(size: i32) -> Handle {
    let u16_at = |i: usize| u16::from_le_bytes([ICON[i], ICON[i + 1]]) as usize;
    let u32_at = |i: usize| u32::from_le_bytes([ICON[i], ICON[i + 1], ICON[i + 2], ICON[i + 3]]) as usize;
    let entries = (0..u16_at(4)).map(|i| {
	let entry = 6 + 16 * i;
	let width = if ICON[entry] == 0 { 256 } else { ICON[entry] as i32 };
	(width, u32_at(entry + 12), u32_at(entry + 8))
    });
    let best = entries.min_by_key(|(width, _, _)| if *width >= size { width - size } else { 1000 + size - width });
    match best {
	Some((_, offset, length)) => unsafe {
	    CreateIconFromResourceEx(ICON[offset..offset + length].as_ptr(), length as u32, 1, 0x00030000, size, size, 0)
	},
	None => std::ptr::null_mut(),
    }
}

struct Tray {
    state: Arc<Mutex<SpeedTestState>>,
    taskbar_created: u32,
    menu: AtomicUsize,
}

static TRAY: OnceLock<Tray> = OnceLock::new();

fn tooltip() -> String {
    let tray = match TRAY.get() {
	Some(t) => t,
	None => return String::new(),
    };
    let (result, when) = status_lines(&tray.state);
    let mut tip = format!("SpeedKetchup\n{result}\n{when}");
    if let Some(error) = error_line() {
	tip = format!("SpeedKetchup\n{error}\n{result}");
    }
    tip
}

fn notify_data(hwnd: Handle) -> NotifyIconData {
    let mut data: NotifyIconData = unsafe { std::mem::zeroed() };
    data.size = std::mem::size_of::<NotifyIconData>() as u32;
    data.hwnd = hwnd;
    data.id = 1;
    data
}

fn set_tip(data: &mut NotifyIconData, tip: &str) {
    for (i, c) in tip.encode_utf16().take(data.tip.len() - 1).enumerate() {
	data.tip[i] = c;
    }
}

fn add_icon(hwnd: Handle) {
    let mut data = notify_data(hwnd);
    data.flags = NIF_MESSAGE | NIF_ICON | NIF_TIP | NIF_SHOWTIP;
    data.callback_message = WM_TRAY;
    data.icon = load_icon(unsafe { GetSystemMetrics(SM_CXSMICON) });
    set_tip(&mut data, &tooltip());
    unsafe {
	Shell_NotifyIconW(NIM_ADD, &data);
	// NOTIFYICON_VERSION_4
	data.version = 4;
	Shell_NotifyIconW(NIM_SETVERSION, &data);
    }
}

fn update(hwnd: Handle, menu: Handle) {
    let tray = match TRAY.get() {
	Some(t) => t,
	None => return,
    };
    let mut data = notify_data(hwnd);
    data.flags = NIF_TIP | NIF_SHOWTIP;
    set_tip(&mut data, &tooltip());
    unsafe {
	Shell_NotifyIconW(NIM_MODIFY, &data);
    }
    if menu.is_null() {
	return;
    }
    let (result, when) = status_lines(&tray.state);
    unsafe {
	for (id, text) in [(ID_RESULT, result), (ID_TIME, when)] {
	    let mut text = wide(&text);
	    let mut info: MenuItemInfo = std::mem::zeroed();
	    info.size = std::mem::size_of::<MenuItemInfo>() as u32;
	    info.mask = MIIM_STRING;
	    info.type_data = text.as_mut_ptr();
	    SetMenuItemInfoW(menu, id as u32, 0, &info);
	}
	// "#32768" is the class of menus
	let window = FindWindowW(wide("#32768").as_ptr(), std::ptr::null());
	if !window.is_null() {
	    InvalidateRect(window, std::ptr::null(), 1);
	}
    }
}

fn show_menu(hwnd: Handle, x: i32, y: i32) {
    let tray = match TRAY.get() {
	Some(t) => t,
	None => return,
    };
    let (result, when) = status_lines(&tray.state);
    let error = error_line();
    let listening = listening(&tray.state);
    unsafe {
	let menu = CreatePopupMenu();
	let add = |flags: u32, id: usize, text: &str| AppendMenuW(menu, flags, id, wide(text).as_ptr());
	// the default item is bold
	add(MF_STRING | MF_GRAYED, ID_HEADER, "SpeedKetchup");
	SetMenuDefaultItem(menu, ID_HEADER as u32, 0);
	AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());
	add(MF_STRING | MF_GRAYED, ID_RESULT, &result);
	add(MF_STRING | MF_GRAYED, ID_TIME, &when);
	if let Some(error) = &error {
	    // enabled, as a disabled item's bitmap is grayed
	    add(MF_STRING, ID_ERROR, error);
	    let mut info: MenuItemInfo = std::mem::zeroed();
	    info.size = std::mem::size_of::<MenuItemInfo>() as u32;
	    info.mask = MIIM_BITMAP;
	    info.bmp_item = HBMMENU_CALLBACK as Handle;
	    SetMenuItemInfoW(menu, ID_ERROR as u32, 0, &info);
	}
	AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());
	add(MF_STRING | if listening { 0 } else { MF_GRAYED }, ID_DASHBOARD, "Open Dashboard");
	add(MF_STRING, ID_TEST, "Run Test Now");
	AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());
	add(MF_STRING | if login_enabled() { MF_CHECKED } else { 0 }, ID_LOGIN, "Start at Login");
	add(MF_STRING, ID_SETTINGS, "Edit Settings…");
	AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());
	add(MF_STRING, ID_QUIT, "Quit SpeedKetchup");

	// else the menu does not close when clicking elsewhere
	SetForegroundWindow(hwnd);
	tray.menu.store(menu as usize, Ordering::Relaxed);
	let command = TrackPopupMenuEx(menu, TPM_RIGHTBUTTON | TPM_RETURNCMD, x, y, hwnd, std::ptr::null());
	tray.menu.store(0, Ordering::Relaxed);
	DestroyMenu(menu);
	if command > 0 {
	    PostMessageW(hwnd, WM_COMMAND, command as usize, 0);
	}
    }
}

fn open_dashboard() {
    if let Some(tray) = TRAY.get() {
	match dashboard_url(&tray.state) {
	    Some(url) => open_url(&url),
	    None => edit_settings(),
	}
    }
}

extern "system" fn wndproc(hwnd: Handle, msg: u32, wparam: usize, lparam: isize) -> isize {
    let tray = TRAY.get();
    match msg {
	WM_TRAY => {
	    let event = (lparam & 0xffff) as u32;
	    if event == WM_CONTEXTMENU || event == NIN_SELECT || event == NIN_KEYSELECT {
		let (mut x, mut y) = ((wparam & 0xffff) as i16 as i32, ((wparam >> 16) & 0xffff) as i16 as i32);
		if event == NIN_SELECT {
		    let mut point = Point { x: 0, y: 0 };
		    if unsafe { GetCursorPos(&mut point) } != 0 {
			(x, y) = (point.x, point.y);
		    }
		}
		show_menu(hwnd, x, y);
	    }
	    0
	},
	WM_COMMAND => {
	    match wparam & 0xffff {
		ID_DASHBOARD => open_dashboard(),
		ID_TEST => if let Some(t) = tray { request_test(&t.state) },
		ID_LOGIN => set_login(!login_enabled()),
		ID_SETTINGS | ID_ERROR => edit_settings(),
		ID_QUIT => {
		    unsafe { Shell_NotifyIconW(NIM_DELETE, &notify_data(hwnd)) };
		    std::process::exit(0);
		},
		_ => (),
	    }
	    0
	},
	WM_TIMER => {
	    update(hwnd, tray.map(|t| t.menu.load(Ordering::Relaxed)).unwrap_or(0) as Handle);
	    0
	},
	WM_MEASUREITEM => {
	    let item = unsafe { &mut *(lparam as *mut MeasureItem) };
	    let size = unsafe { GetSystemMetrics(SM_CXSMICON) } as u32;
	    (item.item_width, item.item_height) = (size, size);
	    1
	},
	WM_DRAWITEM => {
	    let item = unsafe { &*(lparam as *const DrawItem) };
	    let r = &item.rect;
	    let size = (r.bottom - r.top).min(r.right - r.left) * 6 / 10;
	    let (x, y) = ((r.left + r.right - size) / 2, (r.top + r.bottom - size) / 2);
	    unsafe {
		// COLORREF of #e0322b
		let brush = CreateSolidBrush(0x002b32e0);
		let old_brush = SelectObject(item.hdc, brush);
		let old_pen = SelectObject(item.hdc, GetStockObject(NULL_PEN));
		Ellipse(item.hdc, x, y, x + size + 1, y + size + 1);
		SelectObject(item.hdc, old_pen);
		SelectObject(item.hdc, old_brush);
		DeleteObject(brush);
	    }
	    1
	},
	WM_OPEN_DASHBOARD => {
	    open_dashboard();
	    0
	},
	WM_DESTROY => {
	    unsafe { Shell_NotifyIconW(NIM_DELETE, &notify_data(hwnd)) };
	    std::process::exit(0);
	},
	// explorer restarted
	_ if tray.map(|t| t.taskbar_created == msg).unwrap_or(false) => {
	    add_icon(hwnd);
	    0
	},
	_ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

pub fn run(state: Arc<Mutex<SpeedTestState>>) -> ! {
    unsafe {
	let instance = GetModuleHandleW(std::ptr::null());
	let class_name = wide(CLASS);
	let class = WndClassEx {
	    size: std::mem::size_of::<WndClassEx>() as u32,
	    style: 0,
	    wndproc: wndproc,
	    cls_extra: 0,
	    wnd_extra: 0,
	    instance: instance,
	    icon: std::ptr::null_mut(),
	    cursor: std::ptr::null_mut(),
	    background: std::ptr::null_mut(),
	    menu_name: std::ptr::null(),
	    class_name: class_name.as_ptr(),
	    icon_small: std::ptr::null_mut(),
	};
	if RegisterClassExW(&class) == 0 {
	    pr!("RegisterClassExW failed: {}", GetLastError());
	}
	// HWND_MESSAGE
	let hwnd = CreateWindowExW(0, class_name.as_ptr(), wide("SpeedKetchup").as_ptr(), 0, 0, 0, 0, 0,
				   -3isize as Handle, std::ptr::null_mut(), instance, std::ptr::null_mut());
	if hwnd.is_null() {
	    pr!("CreateWindowExW failed: {}", GetLastError());
	}
	let _ = TRAY.set(Tray {
	    state: state.clone(),
	    taskbar_created: RegisterWindowMessageW(wide("TaskbarCreated").as_ptr()),
	    menu: AtomicUsize::new(0),
	});
	add_icon(hwnd);
	SetTimer(hwnd, 1, 1000, std::ptr::null());

	if !at_login() && listening(&state) {
	    open_dashboard();
	}

	let mut msg: Msg = std::mem::zeroed();
	while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
	    TranslateMessage(&msg);
	    DispatchMessageW(&msg);
	}
    }
    std::process::exit(0)
}
