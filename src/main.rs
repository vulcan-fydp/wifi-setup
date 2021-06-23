#![feature(proc_macro_hygiene, decl_macro)]

use rocket::request::Form;
use rocket::response::{status, NamedFile, Redirect};
use rocket_contrib::serve::StaticFiles;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;
use std::process::Command;

#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> Option<NamedFile> {
    NamedFile::open(Path::new("static/main.html")).ok()
}

#[get("/demo")]
fn demo() -> Option<NamedFile> {
    NamedFile::open(Path::new("static/demo.html")).ok()
}

#[post("/switch_connect")]
fn switch_connect() -> status::Accepted<String> {
    status::Accepted(Some("Connecting".to_string()))
}

#[post("/press_a")]
fn press_a() -> status::Accepted<String> {
    status::Accepted(Some("Pressing A".to_string()))
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
fn connect(form: Form<WifiConfig>) -> Redirect {
    let _ = connect_to_network(&form.ssid, &form.pw);
    Redirect::found(uri!(demo))
}

#[catch(404)]
fn redirect() -> Redirect {
    Redirect::found(uri!(index))
}

fn main() {
    rocket::ignite()
        .register(catchers![redirect])
        .mount("/", routes![index, demo, switch_connect, press_a, connect])
        .mount(
            "/static",
            StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/static")),
        )
        .launch();
}
