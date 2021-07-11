use std::fs::File;
use std::io::{self, Read};
use std::fmt;
use std::error;
use url::{Url, ParseError};
use postgres::Client;

#[derive (Debug, Clone)]
struct ReadingError;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;
 
impl fmt::Display for ReadingError{
    fn fmt(&self, f: &mut fmt::Formatter)-> fmt::Result{
        write!(f, "error processing input")}

}

impl error::Error for ReadingError{}

pub fn read_input(stdin_set: bool, file_path: &str)-> Result<String>{
    let mut string_buffer = String::new();
    if stdin_set{
       io::stdin().read_to_string(&mut string_buffer)?; 
    }else if !file_path.eq("not defined"){
        let mut file = File::open(file_path)?;
        file.read_to_string(&mut string_buffer)?;
    };
Ok(string_buffer)
}


pub fn write_to_db(client: &mut Client, data: String, target: &str, host: &str ) -> Result<()>{
    for line in data.lines().into_iter(){
        
        let mut url = line.clone().trim().to_string();
       
        if line.starts_with("https://") | line.starts_with("http://"){
            ()
        }else{ 
            url = format!("https://{}", url );
        };

        let mut itter = url.split_whitespace();
        
        let url = itter.next().unwrap();
        let status_code = match itter.next(){
            Some(status)=>status,
            None=>"Status code not found"
        };

        let parsed = Url::parse(url).expect("error parsing url");
        println!("host: {}, status code: {}", parsed.host_str().expect("error parsing host"), status_code)
    }
    Ok(())
}
