#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate rocket;

use std::io;
use std::path::{Path, PathBuf};

#[allow(unused_imports)]
use rocket::request::{Form, FromFormValue};
use rocket::response::Redirect;
use rocket::response::NamedFile;
use rocket::http::RawStr;

#[derive(Debug, FromForm)]
struct FormInput<'r> {
    age: u8,
    work_type: &'r RawStr,
    active_work_hours: u8,
    emailaddr: &'r RawStr,
}

#[post("/processing", data="<param_form>")]
fn processing<'r>(param_form: Form<'r, FormInput<'r>>) -> Result<Redirect, String> {

    let param = param_form.get();
    if let Err(e) = param.age {
        return Err(format!("Age is invalid: {}", e));
    }

    println!("{}", param.age);
    Ok(Redirect::to("/thanks"))
}

#[get("/")]
fn index() -> io::Result<NamedFile> {
    NamedFile::open("static/index.html")
}

#[get("/<file..>")]
fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).ok()
}

fn main() {
    rocket::ignite().mount("/", routes![index, files, processing]).launch();
}