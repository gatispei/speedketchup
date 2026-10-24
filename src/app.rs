// The desktop app shared by macOS and windows.

use super::*;
use std::path::PathBuf;

pub const SETTINGS: &str = "# SpeedKetchup settings, one VARIABLE=value per line, lines starting with #
# are left out, the ones below show the defaults. SpeedKetchup applies them as
# soon as this file is saved: the interval right away, the others from the
# next test. A bad value is left at its default, the menu shows what is
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

pub fn settings_file() -> PathBuf {
    ui::app_dir().join("speedketchup.conf")
}

fn app_defaults(mut config: SpeedTestConfig) -> SpeedTestConfig {
    config.store_filename = ui::app_dir().join("speedketchup-results.csv").to_string_lossy().to_string();
    config
}

// a bad value is left at its default, the first one is shown in the menu
pub fn app_config() -> SpeedTestConfig {
    let settings = settings();
    let var = |name: &str| setting(name, &settings);
    let mut config = app_defaults(default_config());
    let mut result = config_from_vars(&mut config, &var);
    if let Err(e) = check_config(&config) {
	config.influx_url = None;
	result = result.and(Err(e));
    }
    if let Err(e) = &result {
	pr!("{}: {e}, using the default", settings_file().display());
    }
    set_settings_error(result.err());
    config
}

pub fn create_settings_file() {
    let _ = std::fs::create_dir_all(ui::app_dir());
    let file = settings_file();
    if !file.exists() {
	// notepad before windows 10 wants crlf
	let text = if cfg!(windows) { SETTINGS.replace('\n', "\r\n") } else { SETTINGS.to_string() };
	let _ = std::fs::write(&file, text);
    }
}

pub fn settings() -> Vec<(String, String)> {
    create_settings_file();
    let settings = std::fs::read_to_string(settings_file()).unwrap_or_default();
    let mut vars = vec!();
    for line in settings.lines() {
	// notepad's byte order mark
	let line = line.trim().trim_start_matches('\u{feff}');
	if line.starts_with('#') {
	    continue;
	}
	if let Some((name, value)) = line.split_once('=') {
	    vars.push((name.trim().to_string(), value.trim().trim_matches('"').to_string()));
	}
    }
    vars
}

static SETTINGS_ERROR: Mutex<Option<String>> = Mutex::new(None);

fn set_settings_error(error: Option<String>) {
    if let Ok(mut e) = SETTINGS_ERROR.lock() {
	*e = error;
    }
}

static PORT_ERROR: Mutex<Option<String>> = Mutex::new(None);

pub fn error_line() -> Option<String> {
    let error = SETTINGS_ERROR.lock().ok().and_then(|e| e.clone()).map(|e| format!("Settings: {e}"));
    error.or_else(|| PORT_ERROR.lock().ok().and_then(|e| e.clone()))
}

pub fn listening(state: &Arc<Mutex<SpeedTestState>>) -> bool {
    state.lock().map(|s| s.serving.is_some()).unwrap_or(false)
}

pub fn dashboard_url(state: &Arc<Mutex<SpeedTestState>>) -> Option<String> {
    let state = state.lock().ok()?;
    state.serving.as_ref().map(|(_, port)| format!("http://127.0.0.1:{port}"))
}

// without the port there is no web page, until retry_listen gets it
fn listen(state: &Arc<Mutex<SpeedTestState>>) {
    let (address, port, serving) = match state.lock() {
	Ok(s) => (s.config.listen_address.clone(), s.config.listen_port, s.serving.clone()),
	Err(_) => return,
    };
    if serving.as_ref() == Some(&(address.clone(), port)) {
	return;
    }
    let error = match std::net::TcpListener::bind((address.as_str(), port)) {
	Ok(listener) => {
	    start_server(listener, state);
	    None
	},
	Err(_) => {
	    if let Ok(mut s) = state.lock() {
		s.server_id += 1;
		s.serving = None;
	    }
	    Some(format!("Port {port} is in use by another program"))
	},
    };
    // a connection wakes the old server to stop
    if let Some((old_address, old_port)) = serving {
	let ip = old_address.parse().ok().filter(|ip: &std::net::IpAddr| !ip.is_unspecified());
	let ip = ip.unwrap_or(std::net::Ipv4Addr::LOCALHOST.into());
	let _ = std::net::TcpStream::connect_timeout(&(ip, old_port).into(), Duration::from_secs(1));
    }
    if let Ok(mut e) = PORT_ERROR.lock() {
	if *e != error {
	    pr!("{}", error.as_deref().unwrap_or(&format!("listening on {address}:{port}")));
	    *e = error;
	}
    }
}

pub fn retry_listen(state: Arc<Mutex<SpeedTestState>>) {
    listen(&state);
    let _ = std::thread::Builder::new().name("listen".to_string()).spawn(move || {
	loop {
	    std::thread::sleep(Duration::from_secs(30));
	    listen(&state);
	}
    });
}

fn reload_settings(state: &Arc<Mutex<SpeedTestState>>) {
    let config = app_config();
    if let Ok(mut state) = state.lock() {
	if config == state.config {
	    return;
	}
	state.config = config;
	// the test loop waits for the new interval
	let _ = state.to_main_sender.send(());
    }
    listen(state);
    pr!("reloaded {}", settings_file().display());
}

pub fn settings_changed(state: &Arc<Mutex<SpeedTestState>>, last: &mut Option<Vec<u8>>) {
    let now = std::fs::read(settings_file()).ok();
    if now.is_some() && now != *last {
	*last = now;
	reload_settings(state);
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

pub fn status_lines(state: &Arc<Mutex<SpeedTestState>>) -> (String, String) {
    let state = match state.lock() {
	Ok(s) => s,
	Err(_) => return (String::new(), String::new()),
    };
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
	let secs = until.saturating_duration_since(Instant::now()).as_secs();
	when += &format!("{}next in {:02}:{:02}", if when.len() > 0 { ", " } else { "Next test " }, secs / 60, secs % 60);
    }
    (result, when)
}
