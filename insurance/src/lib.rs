#[macro_use] extern crate rocket;

#[cfg(test)]
mod tests {
    //use chrono::{Local};
    use chrono::prelude::*;
    //use chrono::offset::{Utc, Local};
    use chrono::NaiveDateTime;
    use super::rocket;
    use rocket::local::blocking::Client;
    use crate::*;
    use fs::{remove_file, remove_dir};


    #[test]
    fn display_InsuContract(){
        let contract = InsuContract {
            id: 1,
            name: "Max Mustermann".to_owned(),
            date: Local.ymd(2000, 1, 1).and_hms(0, 0, 0),
            insutype: InsuType::Kfz,
        };
        println!("{contract}");
        assert_eq!(contract.to_string(), format!("(id: 1, name: Max Mustermann, date: {}, insutype: Kfz)", Local.ymd(2000, 1, 1).and_hms(0, 0, 0)));
    }

    #[test]
    fn database_new(){
        if Path::new("./testres/folder").exists(){
            remove_dir(Path::new("./testres/folder"));    
        }
        Database::tmp_new(Path::new("./testres/folder/testdatabase.sqlite"));
        println!("new 1");
        assert!(Path::new("./testres/folder/testdatabase.sqlite").exists());
        let base = Database::tmp_new(Path::new("./testres/folder/testdatabase.sqlite"));
        println!("new 2");
        assert!(Path::new("./testres/folder/testdatabase.sqlite").exists())
    }
    #[test]
    fn database_readwrite(){
        let database = Database::tmp_new(Path::new("./testres/data.sqlite"));
        let utc_date = NaiveDateTime::from_timestamp(20000000000, 0);
        let date: DateTime<Local> = Local.from_utc_datetime(&utc_date);

        //database.write(InsuContract { id: 1, name: "TEst".to_owned(), date, insutype: InsuType::Kfz});
        
        for i in 1..100 {
          database.write(InsuContract { id: i, name: "Max Musterman".to_owned(), date, insutype: InsuType::Reise })
        }
        
        let contracts = &mut database.read().unwrap().into_iter();

        for i in 1..100 {
            if let Some(contract) = contracts.next(){
                println!("database_readwrite(): {}", contract);
                let compare = InsuContract{ id: i, name: "Max Musterman".to_owned(), date, insutype: InsuType::Reise };

                assert_eq!(contract.to_string(), compare.to_string());
            }
        }
              

    }

    #[test]
    fn database_count(){
        let database = Database::tmp_new(Path::new("./testres/database_count.sqlite"));
        let utc_date = NaiveDateTime::from_timestamp(20000000000, 0);
        let date: DateTime<Local> = Local.from_utc_datetime(&utc_date);
        
        for i in 0..20 {
          database.write(InsuContract { id: i, name: "Max Musterman".to_owned(), date, insutype: InsuType::Reise })
        }
        assert_eq!(database.count().unwrap(), 20);
        
         
    }

    #[test]
    fn database_search() {
        let database = Database::tmp_new(Path::new("./testres/search.sqlite"));
        let utc_date = NaiveDateTime::from_timestamp(20000000000, 0);
        let date: DateTime<Local> = Local.from_utc_datetime(&utc_date);

        for i in 0..20{
            database.write(InsuContract { id: i, name: "Max Mustermann".to_owned(), date, insutype: InsuType::Kfz });
        }

        for i in 20..25{
            database.write(InsuContract { id: i, name: "Alex Mustermann".to_owned(), date, insutype: InsuType::Reise });
        }

        for i in 25..45{
            database.write(InsuContract { id: i, name: "Max Mustermann".to_owned(), date, insutype: InsuType::Kfz });
        }
        let mut compare: Vec<InsuContract> = Vec::new();
        
        for i in 20..25{
            compare.push(InsuContract { id: i, name: "Alex Mustermann".to_owned(), date, insutype: InsuType::Reise });
        }
        let search = database.search("Alex");
        for i in 0..compare.len(){
            assert_eq!(compare[i], search[i]);
        }

    }

}


use chrono::prelude::*;
use chrono::{NaiveDate, NaiveDateTime, DateTime};
use rusqlite::{Connection, params};
use serde::ser::SerializeStruct;
use std::marker::PhantomData;
use std::{fmt, path::Path, fs};
extern crate chrono;
use serde::{Deserialize, Serialize};
use fs::create_dir_all;

#[derive(Debug)]
pub struct InsuContract{
    pub id: u64,
    pub name: String,
    pub date: DateTime<Local>,
    pub insutype: InsuType,
}
impl PartialEq for InsuContract {
    fn eq(&self, other: &Self) -> bool {
        if self.id != other.id{
            return false
        }
        if self.name != other.name{
            return false;
        }
        if self.date != other.date{
            return false;
        }
        if self.insutype != other.insutype{
            return false;
        }
        true
    }
    
}

impl serde::ser::Serialize for InsuContract {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
            S: serde::Serializer {
        let mut s = serializer.serialize_struct("InsuContract", 4)?;
        s.serialize_field("id", &self.id)?;
        s.serialize_field("name", &self.name)?;
        s.serialize_field("date", &self.date.to_rfc3339())?;
        s.serialize_field("insutype", &self.insutype)?;
        s.end()
    }
}

impl InsuContract {
    pub fn to_json(&self) -> String{
        format!(r#"
    "id": "{}",
    "name": "{}",
    "date": "{}",
    "insutype":"{}""#, self.id, self.name, self.date, self.insutype)
    }
}

impl fmt::Display for InsuContract{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(id: {}, name: {}, date: {}, insutype: {})", self.id, self.name, self.date, self.insutype)
    }
}

#[derive(std::cmp::PartialEq, Debug, Deserialize, Serialize)]
pub enum InsuType{
    Kfz,
    Hausrat,
    Reise,
}
pub enum ConvertErr{
    CouldnotConvert
}



impl fmt::Display for InsuType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let insutype = match &self{
            InsuType::Kfz => "Kfz",
            InsuType::Hausrat => "Hausrat",
            InsuType::Reise => "Reise",
        };
        write!(f, "{insutype}")
    }
}

impl TryFrom<&str> for InsuType{
    type Error = String;
    fn try_from(phrase: &str) -> Result<Self, Self::Error>{
        match phrase{
            "Kfz" => Ok(InsuType::Kfz),
            "Hausrat" => Ok(InsuType::Hausrat),
            "Reise" => Ok(InsuType::Reise),
            _ => Err("Could not parse phrase".to_owned()),
        }
    }
}

pub struct Database{
    connection: Connection,
}



impl Database {
    pub fn tmp_new(path: &Path)-> Database{
        match path.parent(){
            Some(parent) => { create_dir_all(parent).unwrap()},
            None => {}
        }
        let connection = Connection::open(path).unwrap();
        connection.execute(
            "create table if not exists contracts (
            id  INTEGER PRIMARY KEY,
            name TEXT,
            date INTEGER,
            insutype TEXT
            )", []).unwrap();
        Database { connection }
    }

    pub fn write(&self, ncontract: InsuContract){
        self.connection.execute(
            "INSERT INTO contracts (id, name, date, insutype) VALUES (?1, ?2, ?3, ?4)",
            params![ncontract.id, ncontract.name, ncontract.date.timestamp(), ncontract.insutype.to_string()]
            ).unwrap();
    }
    pub fn read(&self) -> Option<Vec<InsuContract>>{
        let mut prep = self.connection.prepare("SELECT id, name, date, insutype FROM contracts").unwrap();
        let coniter = prep.query_map([], |row| {
            let utc_date = NaiveDateTime::from_timestamp(row.get(2).unwrap(), 0);
            let date: DateTime<Local> = Local.from_utc_datetime(&utc_date);
            let insutypestr: String = row.get(3).unwrap();
            let insutype = InsuType::try_from(insutypestr.as_str()).unwrap();
            Ok(InsuContract{
                id: row.get(0).unwrap(),
                name: row.get(1).unwrap(),
                date,
                insutype,
                }
            )
        }).unwrap();
        
        let mut contracts: Vec<InsuContract> = vec![];
        for contract_res in coniter{
            if let Ok(contract) = contract_res{
                contracts.push(contract);
            }
        }
        Some(contracts)
    }
    pub fn count(&self)-> Option<u64>{
       let mut com = self.connection.prepare("SELECT COUNT(*) FROM contracts").unwrap();
       let answ = com.query_map([], |row |{
           let quan: u64 = row.get(0).unwrap();
           Ok(quan)
       }).unwrap();
       let mut com: Option<u64> = None;
       for i in answ{
            com = Some(i.unwrap());
       }
       com
    }
    pub fn search(&self, phrase: &str)-> Vec<InsuContract>{
        let cmd = format!("SELECT * FROM contracts WHERE name like '%{phrase}%'");
        println!("{}", &cmd);
        let mut req = self.connection.prepare(&cmd).unwrap();
        let response = req.query_map([], |row| {
            let utc_date = NaiveDateTime::from_timestamp(row.get(2).unwrap(), 0);
            let date: DateTime<Local> = Local.from_utc_datetime(&utc_date);
            let insutypestr: String = row.get(3).unwrap();
            let insutype = InsuType::try_from(insutypestr.as_str()).unwrap();
            Ok(InsuContract{
                id: row.get(0).unwrap(),
                name: row.get(1).unwrap(),
                date,
                insutype,
                }
            )
        }).unwrap();
        
        let mut contracts: Vec<InsuContract> = vec![];
        for contract in response.flatten(){
                contracts.push(contract);
        }
        contracts
        
    }

}

pub enum DatabaseErr {
    Pathnotfound,
    CouldnotcreateDatabase
}
