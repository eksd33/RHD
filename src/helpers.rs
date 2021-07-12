
use crate::configuration::{DatabaseSettings, get_conf};
use postgres::{Client, NoTls};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read};
use std::fmt;
use std::error;
use std::ops::Add;
use url::{Url, ParseError};

pub fn db_connect (c:Option<&str>) -> Client {
    let configuration = get_conf(c).unwrap_or_else(|_|{
        DatabaseSettings{
            password: "password".to_string(),
            port : 5432,
            host : "localhost".to_string(),
            username : "rhd_admin".to_string(),
            database_name : "rhd".to_string(),                                                           }
        });
    let connection_string = configuration.get_connection_string();
    let mut client = Client::connect
    (&connection_string , NoTls).expect("Failed to connect to postgres");
    client
}

//resets bigserial after drop: SELECT pg_catalog.setval(pg_get_serial_sequence('table_name', 'id'), (SELECT MAX(id) FROM table_name)+1);
//list all tables: select tablename from pg_tables where schemaname='public';

pub fn check_for_table (connection: &mut  Client, table : &str) -> bool {
    let table = connection.query_one("SELECT EXISTS ( SELECT FROM information_schema.tables WHERE table_schema='public' AND table_name=$1)", &[&table]).unwrap();
    let result:bool = table.get("exists");

    result
}


type ResultReading<T> = std::result::Result<T, Box<dyn error::Error>>;
 

pub fn read_input(stdin_set: bool, file_path: &str)-> ResultReading<String>{
    let mut string_buffer = String::new();
    if stdin_set{
       io::stdin().read_to_string(&mut string_buffer)?; 
    }else if !file_path.eq("not defined"){
        let mut file = File::open(file_path)?;
        file.read_to_string(&mut string_buffer)?;
    };
Ok(string_buffer)
}


pub fn parse_urls(data: String, host: &str ) -> ResultReading<HashMap<Url, String>>{
    let mut urls = HashMap::new();
    let parsed_data: Vec<&str> = data.lines().collect();
    let data_vec = parsed_data.iter().map(|c| c.trim().to_string()).collect::< Vec<String> >();
    for mut line in data_vec{
        if line.starts_with("http://") | line.starts_with("https://"){
            ()
        }else{
            let tmp = format!("https://{}", line);
            line = tmp;
        }
        let mut itter = line.trim().split_whitespace();
        
        let url = itter.next().unwrap();
        let status_code = match itter.next(){
            Some(status)=>status.to_string(),
            None=>"Status code not found".to_string()
        };

        let parsed = Url::parse(url).expect("error parsing url");
        urls.entry(parsed).or_insert_with_key(|key| status_code);
    }
    Ok(urls)
}

