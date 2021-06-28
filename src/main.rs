#![feature(proc_macro_hygiene, decl_macro)]

use controller_emulator::controller::ns_procon;
use controller_emulator::controller::Controller;
use controller_emulator::usb_gadget;
use controller_emulator::usb_gadget::ns_procon::ns_procons;

use rocket::request::Form;
use rocket::response::{status, NamedFile, Redirect};
use rocket::State;
use rocket_contrib::serve::StaticFiles;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;
use std::process::Command;
use std::sync::Mutex;
use std::thread::sleep;
use std::time::Duration;

#[macro_use]
extern crate rocket;

struct Demo {
    controller: Mutex<ns_procon::NsProcon>,
}

#[get("/")]
fn index() -> Option<NamedFile> {
    NamedFile::open(Path::new("static/main.html")).ok()
}

#[get("/demo")]
fn demo() -> Option<NamedFile> {
    NamedFile::open(Path::new("static/demo.html")).ok()
}

#[post("/switch_connect")]
fn switch_connect(state: State<Demo>) -> status::Accepted<String> {
    let _ = usb_gadget::reset("procons");
    let mut controller = state.controller.lock().unwrap();
    controller
        .start_comms()
        .expect("Couldn't start communicating");
    status::Accepted(Some("Connecting".to_string()))
}

#[post("/press_a")]
fn press_a(state: State<Demo>) -> status::Accepted<String> {
    let mut controller = state.controller.lock().unwrap();
    controller.press(ns_procon::inputs::BUTTON_A);
    sleep(Duration::from_millis(100));
    controller.release(ns_procon::inputs::BUTTON_A);
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
    let procons = ns_procons();
    procons
        .create_config("procons")
        .expect("Could not create configuration");

    let procon_1 = ns_procon::NsProcon::create("/dev/hidg0");

    rocket::ignite()
        .register(catchers![redirect])
        .manage(Demo {
            controller: Mutex::new(procon_1),
        })
        .mount("/", routes![index, demo, switch_connect, press_a, connect])
        .mount(
            "/static",
            StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/static")),
        )
        .launch();
}
