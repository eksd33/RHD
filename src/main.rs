use postgres::{Client, NoTls};
use RHD::helpers::{db_connect, check_for_table};
use RHD::db_logic::{write_logic, read_logic, mod_logic};
use RHD::cli_parse::initialize;

fn main(){

// initialize the clap parsing get the matches, get the optional value of the config 
    let cli_args = initialize();
    let cli_args = cli_args.get_matches();
    let conf = cli_args.value_of("config");
    
    let mut client = db_connect(conf);

// proceed based on the subcommands used 
    match cli_args.subcommand(){
        ("read",Some(read))=>{read_logic(&mut client, read)},
        ("write", Some(write))=>{write_logic(&mut client,write)},
        ("mod", Some(mod_match))=>{mod_logic(&mut client,mod_match)},
        _ => {}
    };

}

