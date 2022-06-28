//use std::marker::StructuralEq;
use std::path::Path;

use chrono::Local;
use rocket::data;
use rocket::http::Status;
use rocket::response::{content, status, Responder};
use insurance::*;
use insurance::InsuType;
use insurance::Database;

use rand::prelude::*;
use serde::Serialize;

#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> &'static str {
        "Hello, world!"
}

#[get("/?<phrase>")]
fn search(phrase: &str) -> status::Accepted<content::Json<String>> {
    let database = Database::tmp_new(Path::new("./contracts.sqlite"));
    let results = database.search(phrase);
    /*
    let mut cache: String = "{\n\"list\": [\n".to_owned();
    for i in results {
        cache.push_str("{\n");
        cache.push_str(&i.to_json());
        cache.push_str("\n},\n");
    }
    cache.push_str("]\n}");
    */
    let mut cache: Vec<serde_json::Value> = vec![];
    for i in results {
        &cache.push(serde_json::to_value(i).unwrap());
    }
    let json = serde_json::to_string(&cache).unwrap();
    println!("json: {}", &json);
    status::Accepted(Some(content::Json(format!("{json}"))))
}

#[post("/?<name>&<insutype>")]
fn new(name:&str, insutype: &str) -> status::Accepted<content::Json<String>>{
    if let Ok(res_insutype) = InsuType::try_from(insutype){
        let database = Database::tmp_new(Path::new("./contracts.sqlite"));
        println!("new Database, count: {}", database.count().unwrap());
        //let rand16 = rand::random::<u16>();
        //let rand: u64 = rand16.into();
        let thiscontract = InsuContract {id: database.count().unwrap() + 1, name: name.to_owned(), date: Local::now(), insutype: res_insutype};
        println!("{thiscontract}");
        database.write(thiscontract);
        println!("wrote to database");

        return status::Accepted(Some(content::Json(format!("{{\"name\": \"{name}\", \"insutype\": \"{insutype}\"}}"))))
    }
    status::Accepted(Some(content::Json("{{\"could not make new entry\"}}".to_owned())))
}

#[get("/")]
fn count() -> status::Accepted<content::Json<String>>{
    let database = Database::tmp_new(Path::new("./contracts.sqlite"));
    let quan = database.count().unwrap();
    status::Accepted(Some(content::Json(format!("{{\"quantity\": \"{quan}\"}}"))))
}
#[get("/")]
fn entry() -> status::Accepted<content::Html<String>>{
    status::Accepted(Some(content::Html(include_str!("../../frontend/new_entry.html").to_owned())))
}



#[launch]
fn rocket() -> _ {
        rocket::build()
             .mount("/main", routes![index])
             .mount("/", routes![index])
             .mount("/entry", routes![entry])
             .mount("/api/search/", routes![search])
             .mount("/api/new/", routes![new])
             .mount("/api/count", routes![count])
}


#[cfg(test)]
mod test{
    use std::fmt::format;

    use super::rocket;
    use rocket::local::blocking::Client;
    use rocket::http::Status;
    use rocket::response;

    #[test]
       fn index(){
        let client = Client::tracked(rocket()).expect("no valid rocket instance!");
       let mut response = client.get("/").dispatch();
       assert_eq!(response.status(), Status::Ok);
       assert_eq!(response.into_string().unwrap(), "Hello, world!")
    }

    #[test]
        fn web_new(){
        let client = Client::tracked(rocket()).expect("no valid rocket instace!");
        let uri = r#"/api/new?eintrag="Test""#.to_owned();
        let mut response = client.post("/api/new?name=Max_Mustermann&insutype=Kfz").dispatch();
        assert_eq!(response.status(), Status::Accepted);
        let x = response.into_string().unwrap();
        println!("new(): {}",&x);
        assert_eq!(&x.as_str(), &"{\"name\": \"Max_Mustermann\", \"insutype\": \"Kfz\"}");

    }
    #[test]
    fn web_count(){
        let client = Client::tracked(rocket()).expect("no valid rocket instance!");
        for i in 0..8 {
            println!("executed new()");
            web_new();
        }
        let mut response = client.get("/api/count").dispatch();
        println!("count(): {}", &response.into_string().unwrap());
    }
}
