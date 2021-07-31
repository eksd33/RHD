use crate::configuration::{DatabaseSettings, get_conf};
use postgres::{Client, NoTls};

use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::TryRecvError;
use std::{thread, time};


use std::collections::HashMap;
use std::{iter, mem};
use std::fs::{File, read_to_string};
use std::io::{self, Read, BufRead};
use std::error;

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
        let client = Client::connect
        (&connection_string , NoTls).expect("Failed to connect to postgres");
        client
    }
    //list all tables: select tablename from pg_tables where schemaname='public';

    pub fn check_for_table (connection: &mut  Client, table : &str) -> bool {
        let table = connection.query_one("SELECT EXISTS ( SELECT FROM information_schema.tables WHERE table_schema='public' AND table_name=$1)", &[&table]).unwrap();
        let result:bool = table.get("exists");

        result
    }


    type ResultReading<T> = std::result::Result<T, Box<dyn error::Error>>;
 
    
    fn spawn_stdin_channel() -> Receiver<String> {
        let (tx, rx) = mpsc::channel::<String>();
        thread::spawn(move || loop {
            let mut buffer = String::new();
            io::stdin().read_line(&mut buffer).unwrap();
            if buffer.chars().count() < 1 { break};
            tx.send(buffer).unwrap();
        });
        rx
    }

    pub fn read_input(stdin_set: bool, file_path: &str)-> ResultReading<String>{
        let mut string_buffer = String::new();
        if stdin_set{
            let stdin_channel = spawn_stdin_channel();
            loop {
                match stdin_channel.try_recv() {
                    Ok(key)  => {
                        string_buffer.push_str(key.as_str());
                        println!("{}", key)
                    },
                    Err(TryRecvError::Empty) => {},
                    Err(TryRecvError::Disconnected) => {break}
                }
                thread::sleep(time::Duration::from_millis(20));
            }

        }else if !file_path.eq("not defined"){
            let mut file = File::open(file_path)?;
            file.read_to_string(&mut string_buffer)?;
        };
    Ok(string_buffer)
    }


    pub fn parse_urls(data: String ) -> ResultReading<HashMap<Url, String>>{
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
            let mut status_code = match itter.next(){
                Some(status)=>status.to_string(),
                None=>"Status code not found".to_string()
            };

            status_code = status_code.trim_matches(&['(', ')', '[',']'] as &[char]).to_string(); 
            let parsed = Url::parse(url).expect("error parsing url");
            urls.entry(parsed).or_insert_with_key(|_key| status_code);   
        }
        Ok(urls)
    }

    pub struct Iter<I> ( bool, iter::Peekable<I>) where I: Iterator;
    pub trait IdentifyFirstLast: Iterator + Sized{
        fn identify_first_last(self) -> Iter<Self>;
    }

    impl <I> IdentifyFirstLast for I where I:  Iterator{
        fn identify_first_last(self) -> Iter<Self> {
            Iter(true, self.peekable())
        }
    }

    impl<I> Iterator for Iter<I> where I:Iterator{
        type Item = (bool, bool, I::Item);

        fn next(&mut self) -> Option<Self::Item>{
            let first = mem::replace(&mut self.0, false);
            self.1.next().map(|i| (first, self.1.peek().is_none(), i))
        }
    }

    pub fn query_builder (target: &str, host: Vec<&str>, status_code: Vec<&str>,ids: Vec<&str>,  print_all: bool, delete: bool)-> String{
        let start:String;

        let status_code_empty = status_code.is_empty();
        let host_empty = host.is_empty();
        let ids_empty = ids.is_empty();

        if delete{
            start = format!("DELETE FROM {} WHERE ", target);
        } else {
            start = format!("SELECT * FROM {} WHERE ", target)
        };
     //   println!("{:?}{}", &status_code, &status_code[0].trim_matches(&[','] as &[char]));
        let mut status_code_query = String::new();
        let mut host_query = String::new();
        let mut id_query = String::new();

    // TODO refactor this into separate function that takes vector and identificator string 
        if !status_code_empty{
        for (_is_frst, is_last, val) in status_code.into_iter().identify_first_last(){
            let v = val.trim().trim_matches(&[','] as &[char]);
            if !is_last{
                let q = format!("status_code='{}' OR ",v );
                &status_code_query.push_str(q.as_str());
            }else{
                let q = format!("status_code='{}'",v );
                &status_code_query.push_str(q.as_str());
            } 
        }
        }
        if !host_empty{
            for (is_first, is_last, val) in host.into_iter().identify_first_last(){
            let v = val.trim().trim_matches(&[','] as &[char]);
                if !status_code_empty &&  is_first && !is_last {
                    let q = format!(" AND host='{}' OR ", v);
                    host_query.push_str(q.as_str());
                }
                else if !is_last{
                    let q = format!("host='{}' OR ", v);
                    host_query.push_str(q.as_str());
                }
                else if !status_code_empty && is_first && is_last {
                    let q = format!(" AND host='{}'", v);
                    host_query.push_str(q.as_str());
                }
                else{
                    let q = format!("host='{}'", v);
                    host_query.push_str(q.as_str());
                }

            }
        }

        if !ids_empty {
            for (is_first, is_last, val) in ids.into_iter().identify_first_last(){
                let v = val.trim().trim_matches(&[','] as &[char]);

                if (!status_code_empty || !host_empty) && is_first && !is_last{
                    let q = format!(" AND id='{}' OR ", v);
                    id_query.push_str(q.as_str());
                }
                else if !is_last{
                    let q = format!("id='{}' OR ", v);
                    id_query.push_str(q.as_str());
                }
                else if (!status_code_empty || !host_empty) && is_first && is_last {
                    let q = format!(" AND id='{}'", v);
                    id_query.push_str(q.as_str());
                }
                else {
                    let q = format!("id='{}'", v);
                    id_query.push_str(q.as_str());
                }

            }
        }

        if print_all{
        return format!("SELECT * FROM {}", target);
    }else if delete{
        return format!("{}{}{}{}", start, status_code_query, host_query, id_query)
    }else{
        return format!("{}{}{}", start, status_code_query, host_query)
    }
    
    
}

#[cfg(test)]
    mod tests{
        use super::*;
        
        #[test]
        fn query_building(){
            let target = "test_target";
            let host = vec!["test_host"];
            let status_code = vec!["303,","404"];

            let query = query_builder(target, host, status_code.clone(),Vec::new(), false, false);

            assert_eq!(query, String::from("SELECT * FROM test_target WHERE status_code='303' OR status_code='404' AND host='test_host'"));
            let query = query_builder(target, Vec::new(),  status_code.clone(),Vec::new(),false,false);

            assert_eq!(query, String::from("SELECT * FROM test_target WHERE status_code='303' OR status_code='404'"));

            let host = vec!["test_host_one,", "test_host_two"];
            let query = query_builder(target, host.clone(), Vec::new(),Vec::new(), false, false);
            assert_eq!(query, String::from("SELECT * FROM test_target WHERE host='test_host_one' OR host='test_host_two'"));
            
            let status_code = vec!["303,", "404", "200"];
            let host = vec!["test_host_one," , " test_host_two", "test_host_three"];
            let query = query_builder(target, host, status_code,Vec::new(), false,false);
            assert_eq!(query, String::from("SELECT * FROM test_target WHERE status_code='303' OR status_code='404' OR status_code='200' AND host='test_host_one' OR host='test_host_two' OR host='test_host_three'"))
        }
        #[test]
        fn drop_query_building(){
        let target = "test_target";
        let host = vec!["test_host", "test_host_one"];
        let status_code = vec!["303","404", "200"];
        let ids = vec!["1", "2", "3"];
        let query = query_builder(target, host.clone(), status_code, ids, false,true);

        assert_eq!(query, String::from("DELETE FROM test_target WHERE status_code='303' OR status_code='404' OR status_code='200' AND host='test_host' OR host='test_host_one' AND id='1' OR id='2' OR id='3'"));

        let ids = vec!["1","2,"];
        let query = query_builder(target, host.clone(), Vec::new(), ids, false, true);
        assert_eq!(query, String::from("DELETE FROM test_target WHERE host='test_host' OR host='test_host_one' AND id='1' OR id='2'"));

        let ids = vec!["1"];
        let query = query_builder(target, Vec::new() , Vec::new(), ids, false,true);
        assert_eq!(query, String::from("DELETE FROM test_target WHERE id='1'"));
        }
    }
