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
                    Some(sql_state_unqique_violation) if sql_state_unqique_violation.code() == "23505" => {
                        client.execute(cleanup.as_str(), &[]).expect("error cleaning up");
                        
                        let probe = format!("SELECT status_code FROM {} WHERE url='{}'", target, url);
                        let status_check = client.query_one(probe.as_str(),&[]).expect("error fetching status_code check");
                        let old_status: String = status_check.get("status_code");
                        
                        if !old_status.eq(status){
                            let update_query = format!("UPDATE {} SET status_code='{}' WHERE url='{}'", target, status, url);
                            client.execute(update_query.as_str(), &[]).expect("error updating database");

                        }
                    },
                    Some(other_state) => panic!("unexpected database error sql state: {:?}", other_state),
                    None => panic!()
                }
        }
    }

}

    pub struct PulledData{
    id: i64,
    host: String, 
    url: String,
    status_code: String,
    path: String, 
    } 

pub fn read_logic(client: &mut Client, cli_matches: &ArgMatches){
    let target =  cli_matches.value_of("set target").unwrap_or("not defined");
    
    let list_present = cli_matches.is_present("list");
    let path_present = cli_matches.is_present("path");
    let url_present = cli_matches.is_present("url");
    let status_code_present = cli_matches.is_present("status code");
    let host_present = cli_matches.is_present("set host");
    let target_present = cli_matches.is_present("set target");
    let path_host_status_are_off = !path_present && !host_present && !status_code_present;
    // the print all could be separated from the query_builder function call ... is it woth it tho? 
    let print_all = path_host_status_are_off && target_present && !url_present;

    
    if host_present && !check_for_table(client, target){
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
    

    if url_present && !list_present && !host_present && vec_status_code.is_empty(){
        println!(" Url flag has to be used with at least one other flag: --list -h/--host, --status-code");
        return
    };  

    let query = query_builder(target, host, vec_status_code,Vec::new(), print_all, false);

    //println!("{}", query);

    let mut grid_cli_vec = vec![
        Row::new(vec![
        Cell::new("Id".into(),1),
        Cell::new("Host".into(), 1),
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
    if list_present {
        if path_host_status_are_off && !target_present && !url_present{
        // could replace this if with query.eq("SELECT * FROM not defined");
            for row in client.query("SELECT tablename FROM pg_tables WHERE schemaname='public'", &[]).unwrap(){
                let mut row_t:String = row.get(0);
                row_t.push_str("\n");
                io::stdout().write_all(row_t.as_bytes()).expect("Failed writing stdout");}
        }
        else if url_present {
            let url_query = format!("SELECT url FROM {}", target);
            
            for row in client.query(url_query.as_str(),&[]).expect("Failed querying the database"){
                let mut row_u: String = row.get(0);
                row_u.push_str("\n");
                io::stdout().write_all(row_u.as_bytes()).expect("Failed writing stdout");
            }
        // TODO Could implement only when the --list is present the ID will be shown 
        }
        else if path_present{
            let path_query = format!("SELECT path FROM {}", target);

            for row in client.query(path_query.as_str(), &[]).expect("Failed querying database"){
                let mut row_p:String = row.get(0);
                row_p.push_str("\n");
                io::stdout().write_all(row_p.as_bytes()).expect("Failed writing to stdout");
            }
        }
        else if status_code_present {
            let status_url_query = format!("SELECT url, status_code FROM {}", target);

            for row in client.query(status_url_query.as_str(), &[]).expect("Failed querying database"){
                let row_u:String = row.get(0);
                let row_s:String = row.get(1);
                let mut status_url_line = format!("{}    [{}]", row_u, row_s);
                status_url_line.push_str("\n");
                io::stdout().write_all(status_url_line.as_bytes()).expect("Failed writing to stdout");
            }
        }
    }
    else {
        for row in &client.query(query.as_str(), &[]).unwrap(){
            let row_data = PulledData {
                id: row.get(0),
                host: row.get(1),
                url: row.get(2),
                status_code: row.get(3),
                path: row.get(4)
            };
            if !path_present && !url_present {
                let len_of_url = row_data.host.chars().count();
                
                let grid_row = Row::new(vec![
                    Cell::new(row_data.id.to_string(),1),
                    Cell::new(row_data.host.into(), 1),
                    Cell::new(row_data.status_code.into(), 1),
                    Cell::new(row_data.path.into(), 1),
                ]);

                if len_of_url > max_width_row { max_width_row = len_of_url};
                grid_cli_vec.push(grid_row);
            }
            else if path_present && !url_present {
                let mut filtered_row_p: String = row_data.path;
                filtered_row_p.push_str("\n");
                io::stdout().write_all(filtered_row_p.as_bytes()).expect("Failed writing stdout");
            }
            else if url_present && !path_present {
                let mut filtered_row_u:String = row_data.url;
                filtered_row_u.push_str("\n");
                io::stdout().write_all(filtered_row_u.as_bytes()).expect("Failed writing stdout");
            }
        }
    }
    
    if grid_cli_vec.len() > 1 {
    if max_width_row < 21 {max_width_row = 21;};
    let grid = Grid::builder(grid_cli_vec)
        .default_h_align(HAlign::Center)
        .default_blank_char(' ')
        .column_width(max_width_row)
        .padding_size(3)
        .build();
    println!("{}", grid);

    }
    
}

pub fn mod_logic(client: &mut Client, cli_matches: &ArgMatches){
    let target = cli_matches.value_of("set target").expect("Target name has to be present");
    
    if !check_for_table(client,target){
        println!("Sorry that target is not in the database. Try running: rhd read --list ");
        return
    }
    
    let host = match cli_matches.values_of("set host"){
        Some(v) => v.collect(),
        None => Vec::new(),
    };

    let status_code = match cli_matches.values_of("status code"){
        Some(v) => v.collect(),
        None => Vec::new(),
    };

    let ids = match cli_matches.values_of("id"){
        Some(v) => v.collect(),
        None => Vec::new(),
    };

    let delete_present = cli_matches.is_present("delete");
    let path_comb_present = cli_matches.is_present("path comb");
    let host_status_code_ids_notpresent =  !cli_matches.is_present("set host") && !cli_matches.is_present("status code") && !cli_matches.is_present("id");

    if delete_present && !host_status_code_ids_notpresent {
        let query = query_builder(target, host, status_code, ids, false,delete_present);

        let row_modified = client.execute(query.as_str(), &[]).expect("Error modifying database");

        println!("Number of row modified: {}",row_modified);
    }
    if delete_present && host_status_code_ids_notpresent{
        println!("Please confirm the decision to delete the {} target by by typing: y/Y or: yes/YES", target);
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input).expect("Invalid UTF-8 data");
        
        let user_agrees = user_input.trim().eq_ignore_ascii_case("yes") || user_input.trim().eq_ignore_ascii_case("y");
    
        if user_input.is_empty(){
            println!("If you wish to delete the target you have to confirm the decision by either typing: y/Y  or: yes/YES");
            return
        }

        if !user_agrees{
            println!("Aborting");
            return 
        }

        let delete_target = format!("DROP TABLE {}", target);
        client.execute(delete_target.as_str(), &[]).expect("Error deleting target");
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
