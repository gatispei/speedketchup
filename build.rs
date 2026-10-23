// Minifies the inline scripts of asset/index.html with esbuild into OUT_DIR,
// the page is embedded from there. Without esbuild the page goes in as is,
// unless SPEEDKETCHUP_MINIFY=1 asks for it, as the release builds do. ESBUILD
// is the command to run, esbuild by default.

use std::io::Write;
use std::process::{Command, Stdio};

fn esbuild_minify(js: &str, esbuild: &str) -> Result<String, String> {
    let mut cmd = esbuild.split_whitespace();
    let mut child = Command::new(cmd.next().ok_or("ESBUILD is empty")?)
	.args(cmd)
	.args(["--minify", "--log-level=error"])
	.stdin(Stdio::piped())
	.stdout(Stdio::piped())
	.stderr(Stdio::piped())
	.spawn()
	.map_err(|e| format!("cannot run {esbuild}: {e}"))?;
    // esbuild reads all of stdin before writing anything, so this does not block
    child.stdin.take().ok_or("no stdin")?.write_all(js.as_bytes()).map_err(|e| e.to_string())?;
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    if !out.status.success() {
	return Err(format!("{esbuild} failed: {}", String::from_utf8_lossy(&out.stderr).trim()));
    }
    String::from_utf8(out.stdout).map_err(|e| e.to_string())
}

fn minify_scripts(html: &str, esbuild: &str) -> Result<String, String> {
    let mut out = String::new();
    let mut rest = html;
    while let Some(start) = rest.find("<script") {
	let tag_end = start + rest[start..].find('>').ok_or("unterminated <script")? + 1;
	let end = tag_end + rest[tag_end..].find("</script>").ok_or("no </script>")?;
	let tag = &rest[start..tag_end];
	let body = &rest[tag_end..end];
	out += &rest[..tag_end];
	if tag.contains("src=") || body.trim().is_empty() {
	    out += body;
	} else {
	    out += &esbuild_minify(body, esbuild)?;
	}
	rest = &rest[end..];
    }
    out += rest;
    Ok(out)
}

fn main() {
    println!("cargo:rerun-if-changed=asset/index.html");
    println!("cargo:rerun-if-env-changed=ESBUILD");
    println!("cargo:rerun-if-env-changed=SPEEDKETCHUP_MINIFY");

    let html = std::fs::read_to_string("asset/index.html").expect("cannot read asset/index.html");
    let esbuild = std::env::var("ESBUILD").unwrap_or("esbuild".to_string());
    let required = std::env::var("SPEEDKETCHUP_MINIFY").as_deref() == Ok("1");
    let page = match minify_scripts(&html, &esbuild) {
	Ok(page) => page,
	Err(e) if required => panic!("cannot minify asset/index.html: {e}"),
	Err(e) => {
	    println!("cargo:warning=asset/index.html is embedded unminified: {e}");
	    html
	}
    };
    let out_dir = std::env::var("OUT_DIR").expect("no OUT_DIR");
    std::fs::write(format!("{out_dir}/index.html"), page).expect("cannot write index.html");
}
