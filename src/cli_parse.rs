use clap::{App, AppSettings, Arg, SubCommand};

pub fn initialize () -> App<'static, 'static>{
let mut cli_args = App::new("Rusty hunter - database")
.setting(AppSettings::SubcommandRequiredElseHelp )
.version("0.01")
.author("xD")
.arg(Arg::with_name("config")
    .short("c")
    .long("config")
    .value_name("FILE")
    .help("Set full path to custom config file")
    .takes_value(true))
.subcommand(SubCommand::with_name("write")
    .arg(Arg::with_name("set target")
        .required(true)
        .takes_value(true)
        .help("Set the target name")
        .short("t"))
    .arg(Arg::with_name("set host")
        .takes_value(true)
        .short("h")
        .help("Optional -- explicitely specifies the host"))
    .arg(Arg::with_name("stdin")
         .required_unless("file")
         .long("--stdin")
         .help("When piping input with stdin, please enable this flag with --stdin"))
    .arg(Arg::with_name("file")
         .short("f")
        .required_unless("stdin")
        .takes_value(true)
        .value_name("FILE")
        .help("Specify path to the file you want to insert into RHD")
        .conflicts_with("stdin"))
    )
.subcommand(SubCommand::with_name("read")

    
    );

    cli_args
}
