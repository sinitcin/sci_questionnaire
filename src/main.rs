#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate rocket;

use std::io;
use std::path::{Path, PathBuf};

use rocket::http::RawStr;
#[allow(unused_imports)]
use rocket::request::{Form, FromFormValue, Request};
use rocket::response::NamedFile;
use rocket::response::Redirect;

enum WorkType {
    FullDay,
    FreeDay,
}

struct DataRow<'a> {
    age: u8,
    specialization: &'a str,
    work_type: WorkType,
    active_work_hours: u8,
    email: &'a str,
    points: Vec<(u8, u8)>,
}

impl<'a> From<&'a FormInput<'a>> for DataRow<'a> {
    fn from(raw_data: &'a FormInput) -> DataRow<'a> {

    // Массив
    // [["00:00",0],["01:00",0],["02:00",0],["03:00",0],["04:00",0],["05:00",0],["06:00",0],["07:00",1],["08:00",2]]
        // 
        let wtype = match raw_data.work_type.as_str() {
            "Полный день" => WorkType::FullDay,
            _ => WorkType::FreeDay,
        };
        
        DataRow {
            age: raw_data.age,
            specialization: raw_data.specialization,
            work_type: wtype,
            active_work_hours: raw_data.active_work_hours,
            email: raw_data.email,
            points: vec![(0, 0)]
            // points: Vec<(u8, u8)>,
        }
    }
}

#[derive(Debug, FromForm)]
struct FormInput<'r> {
    age: u8,
    specialization: &'r RawStr,
    work_type: &'r RawStr,
    active_work_hours: u8,
    email: &'r RawStr,
    points: &'r RawStr,
    submit: &'r RawStr,
}

#[get("/")]
fn index() -> io::Result<NamedFile> {
    NamedFile::open("static/index.html")
}

#[get("/<file..>")]
fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).ok()
}

#[post("/processing", data = "<param_form>")]
fn processing<'r>(param_form: Form<'r, FormInput<'r>>) -> Result<Redirect, String> {

    let param = param_form.get();
    println!("{}", param.specialization);
    Ok(Redirect::to("/thanks"))
}

#[get("/thanks")]
fn thanks() -> io::Result<NamedFile> {
    NamedFile::open("static/thanks.html")
}

#[catch(404)]
fn not_found(_req: &Request) -> io::Result<NamedFile> {
    NamedFile::open("static/404.html")
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index, files, processing, thanks])
        .catch(catchers![not_found])
        .launch();
}
