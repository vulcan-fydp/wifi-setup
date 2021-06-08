#![feature(proc_macro_hygiene, decl_macro)]

use rocket::request::Form;
use rocket::response::{status, NamedFile, Redirect};
use std::path::{Path, PathBuf};

#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> Option<NamedFile> {
    NamedFile::open(Path::new("static/main.html")).ok()
}

#[derive(FromForm)]
struct WifiConfig {
    ssid: String,
    pw: String,
}

#[post("/connect", data = "<form>")]
fn connect(form: Form<WifiConfig>) -> status::Accepted<String> {
    println!("ssid: {} pw: {}", form.ssid, form.pw);
    status::Accepted(Some("Connecting".to_string()))
}

#[get("/<path..>")]
fn redirect(path: PathBuf) -> Redirect {
    Redirect::found(uri!(index))
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index, connect, redirect])
        .launch();
}
