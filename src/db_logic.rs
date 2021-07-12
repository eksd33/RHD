use clap::ArgMatches;
use crate::configuration::{DatabaseSettings, get_conf};
use postgres::Client;
use crate::helpers::*;


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
    let mut read_contents = read_input( stdin, file_to_read).expect("Sorry there was problem processing the input: ");
    
    for (key, val) in parse_urls(read_contents, "host").expect("error parsing urls").iter(){
        println!("host {}, status code {}", key.host_str().unwrap(), val)
    }
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
        let existing_path:Option<&str> = Some("./conf.yaml");
        let custom = get_conf(existing_path).expect("unable to get custom conf");
        assert_eq!(custom.username, "custom_name");
    }

 }
