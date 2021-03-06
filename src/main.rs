#![feature(proc_macro_hygiene, decl_macro)]
#![feature(bool_to_option)]

use reqwest;
use rocket::request::Form;
use rocket::response::Redirect;
use rocket_contrib::json::Json;
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::process::Command;
use regex::Regex;
use lazy_static::lazy_static;
use anyhow::Result;

#[macro_use]
extern crate rocket;

fn get_ssids() -> Result<Vec<String>> {
    lazy_static! {
        static ref SSID_REGEX: Regex = Regex::new("SSID:\"(.+)\"").unwrap();
    }
    let mut ssids = vec![];
    let scan_out = Command::new("iwlist").arg("wlan0").arg("scan").output()?;
    let text = String::from_utf8(scan_out.stdout)?;
    for cap in SSID_REGEX.captures_iter(&text) {
        if let Some(m) = cap.get(1) {
            ssids.push(m.as_str().to_owned());
        }
    }
    Ok(ssids)
}

#[derive(Serialize)]
struct Ssids {
    ssids: Vec<String>,
}

#[get("/scan_ssids")]
fn scan_ssids() -> Json<Ssids> {
    let ssids = get_ssids().unwrap_or(vec![]);
    Json(Ssids { ssids })
}

#[get("/")]
fn ssids() -> Result<Template> {
    let ssids = get_ssids().unwrap_or(vec![]);
    let context: HashMap<&str, Vec<String>> = [("ssids", ssids)].iter().cloned().collect();
    Ok(Template::render("ssid-list", &context))
}

#[get("/ssid/<ssid>")]
fn ssid(ssid: String) -> Template {
    let context: HashMap<&str, String> = [("ssid", ssid)].iter().cloned().collect();
    Template::render("ssid", &context)
}

fn connect_to_network(ssid: &str, pw: &str) -> Result<()> {
    let config = Command::new("wpa_passphrase").arg(ssid).arg(pw).output()?;
    config.status.success().then_some(0).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            "wpa_passphrase exited with error",
        )
    })?;
    let mut conf_file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("/etc/wpa_supplicant/wpa_supplicant.conf")?;
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
fn connect(form: Form<WifiConfig>) -> Template {
    let _ = connect_to_network(&form.ssid, &form.pw);
    let context: HashMap<&str, String> = [("ssid", form.ssid.clone())].iter().cloned().collect();
    Template::render("connecting", &context)
}

#[derive(Serialize)]
struct IsConnected {
    connected: bool,
}

#[get("/is_connected")]
fn is_connected() -> Json<IsConnected> {
    let check_url = "http://clients3.google.com/generate_204";
    let connected = match reqwest::blocking::get(check_url) {
        Err(_) => false,
        Ok(s) => s.status().is_success(),
    };

    Json(IsConnected { connected })
}

#[catch(404)]
fn redirect() -> Redirect {
    Redirect::found(uri!(ssids))
}

fn main() {
    let static_dir = match option_env!("WIFI_SETUP_STATIC_DIR") {
        Some(dir) => dir,
        None => concat!(env!("CARGO_MANIFEST_DIR"), "/static"),
    };

    rocket::ignite()
        .attach(Template::fairing())
        .register(catchers![redirect])
        .mount("/", routes![ssids, ssid, connect, is_connected, scan_ssids])
        .mount("/static", StaticFiles::from(static_dir))
        .launch();
}
