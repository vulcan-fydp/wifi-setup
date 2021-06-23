#![feature(proc_macro_hygiene, decl_macro)]

use rocket::request::Form;
use rocket::response::{status, NamedFile, Redirect};
use rocket_contrib::serve::StaticFiles;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;

#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> Option<NamedFile> {
    NamedFile::open(Path::new("static/main.html")).ok()
}

fn connect_to_network(ssid: &str, pw: &str) -> std::io::Result<()> {
    let config = Command::new("wpa_passphrase").arg(ssid).arg(pw).output()?;
    let mut conf_file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("/etc/wpa_supplicant/wpa_supplicant-wlan0.conf")?;
    conf_file.write_all(&config.stdout)?;
    Command::new("wpa_cli")
        .arg("-i")
        .arg("wlan0")
        .arg("reconfigure")
        .status()?;
    Ok(())
}

#[derive(FromForm)]
struct WifiConfig {
    ssid: String,
    pw: String,
}

#[post("/connect", data = "<form>")]
fn connect(form: Form<WifiConfig>) -> status::Accepted<String> {
    let _ = connect_to_network(&form.ssid, &form.pw);
    status::Accepted(Some("Connecting".to_string()))
}

#[get("/<path..>")]
fn redirect(path: PathBuf) -> Redirect {
    Redirect::found(uri!(index))
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index, connect, redirect])
        .mount("/static", StaticFiles::from("/static"))
        .launch();
}
