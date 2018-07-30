#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate serde_json;

use std::io;
use std::path::{Path, PathBuf};

macro_rules! ok(($result:expr) => ($result.unwrap()));

use rocket::http::RawStr;
#[allow(unused_imports)]
use rocket::request::{Form, FromFormValue, Request};
use rocket::response::NamedFile;
use rocket::response::Redirect;

enum WorkType {
    FullDay,
    FreeDay,
}

pub struct DataRow<'a> {
    age: u8,
    specialization: &'a str,
    work_type: WorkType,
    active_work_hours: u8,
    email: &'a str,
    points: Vec<u64>,
}

impl<'a> From<&'a FormInput<'a>> for DataRow<'a> {
    fn from(raw_data: &'a FormInput) -> DataRow<'a> {
        let wtype = match raw_data.work_type.as_str() {
            "Полный день" => WorkType::FullDay,
            _ => WorkType::FreeDay,
        };
        let mut arrpoint = Vec::new();
        let buffer = raw_data.points.percent_decode().expect(
            "Не смог расшифровать данные в POST запросе...",
        );
        let v: serde_json::Value = serde_json::from_str(&buffer).expect(&format!(
            "Не смог принять массив: {}",
            buffer
        ));
        for i in 0..23 {
            arrpoint.push(
                v[i][1]
                    .as_u64()
                    .expect("Не смог получить данные из массива..."),
            );
        };
        let spec = raw_data.specialization.percent_decode();
        let spec = &'a spec.expect("Не смог преобразовать поле `специальность` в UTF-8");
        DataRow {
            age: raw_data.age,
            specialization: spec,
            work_type: wtype,
            active_work_hours: raw_data.active_work_hours,
            email: raw_data.email,
            points: arrpoint,
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
    let data = DataRow::from(param);
    database::store_row(data);
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
    database::create_or_open();
    rocket::ignite()
        .mount("/", routes![index, files, processing, thanks])
        .catch(catchers![not_found])
        .launch();
}

pub mod database {

    extern crate sqlite;
    use self::sqlite::{Connection, State, Type, Value};
    use super::DataRow;
    use std::fs;

    pub fn create_or_open() {
        let mut avail = false;
        if let Ok(metadata) = fs::metadata("database.sqlite") {
            let file_type = metadata.file_type();
            if (file_type.is_file()) {
                // Файл уже есть
                avail = true;
            }
        }
        let connection = ok!(sqlite::open("database.sqlite"));
        if (!avail) {
            ok!(connection.execute(
                "CREATE TABLE opinions (age INTEGER, specialization TEXT, work_type INTEGER);"
            ));
        }
    }

    pub fn store_row(data: DataRow) {
        let connection = ok!(sqlite::open("database.sqlite"));
        let statement = "INSERT INTO opinions (age, specialization, work_type) VALUES (?, ?, ?)";
        let mut statement = ok!(connection.prepare(statement));
        ok!(statement.bind(1, data.age as i64));
        ok!(statement.bind(2, data.specialization));
        ok!(statement.bind(3, data.work_type as i64));
        assert_eq!(ok!(statement.next()), State::Done);
    }
}
