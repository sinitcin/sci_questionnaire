#![feature(plugin, custom_derive, decl_macro)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rocket_contrib;
extern crate serde_json;
#[macro_use] 
extern crate serde_derive;

use std::io;
use std::path::{Path, PathBuf};

macro_rules! ok(($result:expr) => ($result.unwrap()));

use rocket::http::RawStr;
#[allow(unused_imports)]
use rocket::request::{Form, FromFormValue, Request};
use rocket::response::NamedFile;
use rocket::response::Redirect;
use rocket_contrib::Template;


#[derive(Serialize)]
struct TemplateContext {
    participants: String,
}

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
    points: &'a str,
}

impl<'a> From<&'a FormInput<'a>> for DataRow<'a> {
    fn from(raw_data: &'a FormInput) -> DataRow<'a> {
        let wtype = match raw_data.work_type.as_str() {
            "Полный день" => WorkType::FullDay,
            _ => WorkType::FreeDay,
        };
        let buffer = raw_data.points.percent_decode().expect(
            "Не смог расшифровать данные в POST запросе...",
        );
        let _v: serde_json::Value = serde_json::from_str(&buffer).expect(&format!(
            "Не смог принять массив: {}",
            buffer
        ));
        DataRow {
            age: raw_data.age,
            specialization: raw_data.specialization,
            work_type: wtype,
            active_work_hours: raw_data.active_work_hours,
            email: raw_data.email,
            points: raw_data.points,
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
fn index() -> Template {

    let count = format!("{}", database::number_of_participants());
    let end_with = match count.chars().last() {
        Some('2') | 
        Some('3') | 
        Some('4') => "человека",
        _ => "человек",
    };

    let context = TemplateContext {
        participants: format!("{} {}", count, end_with) 
    };

    Template::render("index", &context)
}

#[get("/<file..>")]
fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).ok()
}

#[post("/processing", data = "<param_form>")]
fn processing<'r>(param_form: Form<'r, FormInput<'r>>) -> Result<Redirect, String> {
    let param = param_form.get();
    let decoded_mail = &*param.email.percent_decode().expect("Не смог расшифровать <email> из запроса");
    let decoded_mail = &RawStr::from_str(decoded_mail);
    let decoded_specialization = &*param.specialization.percent_decode().expect("Не смог расшифровать <специальность> из запроса");
    let decoded_specialization = &RawStr::from_str(decoded_specialization);
    let decoded_points = &*param.points.percent_decode().expect("");
    let decoded_points = &RawStr::from_str(decoded_points);
    let param2 = &FormInput {
        specialization: decoded_specialization,
        email: decoded_mail,
        points: decoded_points,
        ..*param
    };
    let data = DataRow::from(param2);
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
    database::create();
    rocket::ignite()
        .mount("/", routes![index, files, processing, thanks])
        .attach(Template::fairing())
        .catch(catchers![not_found])
        .launch();
}

pub mod database {

    extern crate sqlite;
    use self::sqlite::State;
    use super::DataRow;
    use std::path::Path;

    /// Путь к БД 
    fn get_db_path() -> String {
        if cfg!(windows) {
            "database.sqlite".to_owned()
        } else {
            "/storage/database.sqlite".to_owned()
        }
    }

    /// Создание новой БД
    pub fn create() {
        if !Path::new(&get_db_path()).exists() {
            let connection = ok!(sqlite::open(get_db_path()));
            ok!(connection.execute(
                "CREATE TABLE opinions (age INTEGER, specialization TEXT, work_type INTEGER, active_work_hours INTEGER, email TEXT, points TEXT);"
            ));
        }
    }

    /// Записать в БД данные нового человека
    pub fn store_row(data: DataRow) {
        let connection = ok!(sqlite::open(get_db_path()));
        let statement = "INSERT INTO opinions (age, specialization, work_type, active_work_hours, email, points) VALUES (?, ?, ?, ?, ?, ?)";
        let mut statement = ok!(connection.prepare(statement));
        ok!(statement.bind(1, data.age as i64));
        ok!(statement.bind(2, data.specialization));
        ok!(statement.bind(3, data.work_type as i64));
        ok!(statement.bind(4, data.active_work_hours as i64));
        ok!(statement.bind(5, data.email));
        ok!(statement.bind(6, data.points));
        assert_eq!(ok!(statement.next()), State::Done);
    }

    /// Количество людей участвовавших в опросе
    pub fn number_of_participants() -> u64 {
        let connection = ok!(sqlite::open(get_db_path()));
        let mut statement = ok!(connection.prepare("SELECT COUNT(*) FROM opinions"));
        let _ = ok!(statement.next()); 
        ok!(statement.read::<i64>(0)) as u64
    }
}
