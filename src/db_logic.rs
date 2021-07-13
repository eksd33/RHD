use clap::ArgMatches;
use std::io::Read;
use std::collections::HashMap;
use url::Url;
use crate::configuration::{DatabaseSettings, get_conf};
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
    let host = cli_matches.value_of("set host");

    let stdin = cli_matches.is_present("stdin");
    
    let file_to_read = match cli_matches.value_of("file"){
        Some(file)=> file,
        None => "not defined"
    };
    let mut read_contents = read_input( stdin, file_to_read).expect("Sorry there was problem processing the input: ");
    
    let parsed_urls = parse_urls(read_contents).expect("error parse_urls");
    //write_to_db(client, parsed_urls, target, host, "some note");
    
    // Display stdout as psql  
    //
    //let mut reader = client.copy_out("COPY sometarget TO stdout").unwrap();
    //let mut buf = vec![];
    //reader.read_to_end(&mut buf).unwrap();
    //println!("{}", String::from_utf8_lossy(&buf));
}



pub fn write_to_db(client: &mut Client, data: HashMap<Url, String>, target: &str, host_option: Option<&str>, note: &str){

    for (url, status) in data.iter(){
        let prep = match host_option {
            Some(host) => format!("INSERT INTO {} (host,url, status_code, note) VALUES ('{}','{}','{}','{}')", target, host, url, status, note),
            None => format!("INSERT INTO {} (host,url, status_code, note) VALUES ('{}','{}','{}','{}')", target, url.host_str().expect("error parsing host from url"), url, status, note),
        };
        // TODO impl error handle for duplicate urls (primary key)
        client.batch_execute(prep.as_str()).expect("error writing to db ");
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
    #[test]
    fn parse_the_urls_file(){
        let mut file_content = read_input(false, "/home/xd/Projects/test_file").expect("Error reading file");
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
