#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate rocket;

use std::io;
use std::path::{Path, PathBuf};

use rocket::request::{Form, FromFormValue};
use rocket::response::NamedFile;
use rocket::http::RawStr;

#[derive(Debug, FromForm)]
struct FormInput<'r> {
    checkbox: bool,
    password: &'r RawStr,
}

#[post("/processing", data="<sink>")]
fn processing<'r>(sink: Result<Form<'r, FormInput<'r>>, Option<String>>) -> String {
    match sink {
        Ok(form) => format!("{:?}", form.get()),
        Err(Some(f)) => format!("Invalid form input: {}", f),
        Err(None) => format!("Form input was invalid UTF8."),
    }
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