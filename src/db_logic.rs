use clap::ArgMatches;
use cli_grid::*;
use std::io::{self, Write, Read};
use std::collections::HashMap;
use url::Url;
use postgres::Client;
use crate::helpers::*;


pub fn write_logic(client: &mut Client,cli_matches: &ArgMatches){
    if let Some(target) = cli_matches.value_of("set target"){
        let exists = check_for_table(client, target);
// wont let me execute with $1 in client.execute -- so this is a work-aroud 
    let query = format!(
        "CREATE TABLE {} (
            id BIGSERIAL, 
            host VARCHAR(150), 
            url TEXT PRIMARY KEY NOT NULL,
            status_code VARCHAR(50),
            path TEXT
        )", target);
            
        if !exists {
            client.batch_execute(&query).expect("unable to create table");
        }
    };
    let target = cli_matches.value_of("set target").unwrap_or("not defined");

    let host = cli_matches.value_of("set host");

    let stdin = cli_matches.is_present("stdin");
    
    let file_to_read = match cli_matches.value_of("file"){
        Some(file)=> file,
        None => "not defined"
    };
    let read_contents = read_input( stdin, file_to_read).expect("Sorry there was problem processing the input: ");
    
    let parsed_urls = parse_urls(read_contents).expect("error parse_urls");
    write_to_db(client, parsed_urls, target, host);

}



pub fn write_to_db(client: &mut Client, data: HashMap<Url, String>, target: &str, host_option: Option<&str>){

    let cleanup = format!("SELECT setval(pg_get_serial_sequence('{}', 'id'), max(id)) FROM {}",target, target);
    
    for (url, status) in data.iter(){
        let prep = match host_option {
            Some(host) => format!("INSERT INTO {} (host,url, status_code, path) VALUES ('{}','{}','{}', '{}')", target, host, url, status, url.path() ),
            None => format!("INSERT INTO {} (host,url, status_code, path) VALUES ('{}','{}','{}','{}')", target, url.host_str().expect("error parsing host from url"), url, status,url.path()),
        };
        // cleanup on duplicate entries -- if the entry was duplicate on the db the cleanup query
        // will reset the sequence 

        let exec = client.batch_execute(prep.as_str());
        match exec {
            Ok(()) => (),
            Err(error) => match error.code(){
                    Some(sql_state_unqique_violation) if sql_state_unqique_violation.code() == "23505" => {client.execute(cleanup.as_str(), &[]).expect("error cleaning up");},
                    Some(other_state) => panic!("unexpected database error sql state: {:?}", other_state),
                    None => panic!()
                }
        }
    }

}

pub struct PulledData{
    host: String, 
    url: String,
    status_code: String,
    path: String, 
    } 

pub fn read_logic(client: &mut Client, cli_matches: &ArgMatches){
    let target =  cli_matches.value_of("set target").unwrap_or("not defined");

    if !check_for_table(client, target){
        println!("Sorry that target is not in the database. Try running: rhd read --list ");
        return 
    }
    let host = match cli_matches.values_of("set host"){
        Some(v) => v.collect(),
        None => Vec::new(),
    };

    
    let vec_status_code = match cli_matches.values_of("status code"){
        Some(v) => v.collect(), 
        None => Vec::new(),
    };

    let list_present = cli_matches.is_present("list");
    let path_present = cli_matches.is_present("path");
    let url_present = cli_matches.is_present("url");
    let host_present = cli_matches.is_present("set host");
    let target_present = cli_matches.is_present("set target");

    if url_present && !list_present && !host_present && vec_status_code.is_empty(){
        println!(" Url flag has to be used with at least one other flag: --list -h/--host, --status-code");
        return
    }

    let query = query_builder(target, host, vec_status_code, list_present );
    let path_url_host_are_off = !path_present && !url_present && !host_present;

    let mut grid_cli_vec = vec![
        Row::new(vec![
        Cell::new("Host".into(), 1),
        Cell::new("Url".into(), 1),
        Cell::new("Status code".into(), 1),
        Cell::new("Path".into(), 1),
        ])
    ];
    let mut max_width_row = 1;

//
//
// TODO REPLACE THIS IF ELSE MESS WITH SOME SWITCH / MATCH GUARD; 
//
//
    if list_present  && path_url_host_are_off && !target_present{
    // could replace this if with query.eq("SELECT * FROM not defined");
        for row in client.query("SELECT tablename FROM pg_tables WHERE schemaname='public'", &[]).unwrap(){
            let mut row_t:String = row.get(0);
            row_t.push_str("\n");
            io::stdout().write_all(row_t.as_bytes()).expect("Failed writing stdout");}
    }else if list_present && url_present {
        let url_query = format!("SELECT url FROM {}", target);
        for row in client.query(url_query.as_str(),&[]).expect("Failed querying the database"){
            let mut row_u: String = row.get(0);
            row_u.push_str("\n");
            io::stdout().write_all(row_u.as_bytes()).expect("Failed writing stdout");
        } 
    
    }else {
        println!("{}", query);
        for row in &client.query(query.as_str(), &[]).unwrap(){
            let row_data = PulledData {
                host: row.get(1),
                url: row.get(2),
                status_code: row.get(3),
                path: row.get(4)
            };
            if !path_present && !url_present {
                let len_of_url = row_data.url.chars().count();
                
                let grid_row = Row::new(vec![
                    Cell::new(row_data.host.into(), 1),
                    Cell::new(row_data.url.into(), 1),
                    Cell::new(row_data.status_code.into(), 1),
                    Cell::new(row_data.path.into(), 1),
                ]);

                if len_of_url > max_width_row { max_width_row = len_of_url};
                grid_cli_vec.push(grid_row);
            }else if path_present && !url_present {
                
            }else if url_present && !list_present {
                let mut filtered_row_u:String = row.get("url");
                filtered_row_u.push_str("\n");
                io::stdout().write_all(filtered_row_u.as_bytes()).expect("Failed writing stdout");
            }
        }
    }
    
    if grid_cli_vec.len() > 1 {

    let grid = Grid::builder(grid_cli_vec)
        .default_h_align(HAlign::Center)
        .default_blank_char(' ')
        .column_width(max_width_row)
        .build();
    println!("{}", grid);

    }
    
}


#[cfg(test)]
mod tests{
    use super::*;
    use crate::configuration::{get_conf, DatabaseSettings};
    #[test]
    fn test_postgres_connection(){
        use std::time::Duration;

        let conf = Some("not_existing_config");
        let mut client = db_connect(conf);
        let duration = Duration::new(15,0);
        assert!(client.is_valid(duration).is_ok());
    }
    #[test]
    fn custom_config(){
        let existing_path:Option<&str> = Some("./conf.yaml");
        let custom = get_conf(existing_path).expect("unable to get custom conf");
        assert_eq!(custom.username, "custom_name");
    }
    #[test]
    fn parse_the_urls_file(){
        let file_content = read_input(false, "/home/xd/Projects/test_file").expect("Error reading file");
        let p_urls = parse_urls(file_content).expect("Error parsing urls");
        let key_should_exist = Url::parse("https://qstr.googl.t").expect("error parsing url in test");
        assert!(p_urls.contains_key(&key_should_exist))
    }
    // If the name is not self explanatory -> checks for table it it doesn't exists, it creates one
    // and then it drops the table at the end of the test
    #[test]
    fn crate_table_if_not_existing_then_drop(){
        let mut client = db_connect(Some("not_existing_config"));
        let not_existing_table = "this_table_should_not_exist";
        let exists = check_for_table(&mut client, not_existing_table);
        assert!(!exists);
        let query = format!(
        "CREATE TABLE {} (
            id BIGSERIAL, 
            host VARCHAR(150), 
            url TEXT PRIMARY KEY NOT NULL,
            status_code VARCHAR(3),
            note TEXT
        )", not_existing_table);
        if !exists {
            client.batch_execute(&query).expect("unable to create table");
        }
        let should_exist_now = check_for_table(&mut client, not_existing_table);
        assert!(should_exist_now);

        let drop_prep = format!("DROP TABLE {}", &not_existing_table);
        client.execute(drop_prep.as_str(), &[]).expect("error droping table");

        let table_deleted = check_for_table(&mut client, not_existing_table);
        assert!(!table_deleted)
        
    }
 }
