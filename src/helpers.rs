use crate::configuration::{DatabaseSettings, get_conf};
use crate::db_logic::{write_to_db, read_input};
use clap::ArgMatches;
use postgres::{NoTls,Client};

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


pub fn match_write(client: &mut Client,cli_matches: &ArgMatches){
    if let Some(target) = cli_matches.value_of("set target"){
        let exists = check_for_table(client, target);
// wont let me execute with $1 in client.execute -- so this is a work-aroud 
        let query = format!(
        "CREATE TABLE {} (
            id BIGSERIAL, 
            host VARCHAR(150), 
            url TEXT PRIMARY KEY NOT NULL,
            status_code VARCHAR(3),
            note TEXT
        )", target);
        if !exists {
            client.batch_execute(&query).expect("unable to create table");
        }

    //maybe create a table just for url paths as primary key and foreign key of host in main target
    //table 
    };
    let target = match cli_matches.value_of("set target"){
        Some(target) => target,
        None => "not defined"
    };
    let host = match cli_matches.value_of("set host"){
        Some(host)=> host,
        None => "not defined"
    };

    let stdin = cli_matches.is_present("stdin");
    
    let file_to_read = match cli_matches.value_of("file"){
        Some(file)=> file,
        None => "not defined"
    };
    let read_contents = read_input( stdin, file_to_read).expect("Sorry there was problem processing the input: ");

    write_to_db(client, read_contents, target, host).expect("fucked up writing to db");   
}

#[cfg(test)]
mod tests{
    use super::*;
    
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
        let existing_path:Option<&str> = Some("./configuration.yaml");
        let custom = get_conf(existing_path).expect("unable to get custom conf");
        assert_eq!(custom.username, "custom_name");
    }

 }
