use postgres::{Client, NoTls};
use RHD::configuration::{DatabaseSettings, get_conf};
use clap::{Arg, App};

fn main(){
    let cli_args = App::new("Rusty hunter - database")
        .version("0.01")
        .author("xD")
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("FILE")
            .help("Set full path to custom config file")
            .takes_value(true))
        .get_matches();
    let config = cli_args.value_of("config");

    
}

fn db_connect (c:Option<&str>) {
    let configuration = get_conf(c).unwrap_or_else(|_|{
        DatabaseSettings{
            password: "password".to_string(),
            port : 5432,
            host : "localhost".to_string(),
            username : "rhd_admin".to_string(),
            database_name : "rhd".to_string(),                                                           }
        });
    let connection_string = configuration.get_connection();
    let mut client = Client::connect
    (&connection_string , NoTls).expect("Failed to connect to postgres");
}

