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
        .help("Set the target name that will be used for storing the data")
        .short("t"))
    .arg(Arg::with_name("set host")
        .takes_value(true)
        .short("h")
        .help("Optional -- explicitely specifies the host that will be used instead of infering the host from the urls"))
    .arg(Arg::with_name("stdin")
        .required_unless("file")
        .long("--stdin")
        .help("When piping input with stdin, please enable this flag with --stdin"))
    .arg(Arg::with_name("file")
        .short("f")
        .required_unless("stdin")
        .takes_value(true)
        .value_name("FILE")
        .help("Specify path to the file you want to import into RHD")
        .conflicts_with("stdin"))
    )
.subcommand(SubCommand::with_name("read")
    .arg(Arg::with_name("set target")
        .short("t")
        .long("--target")
        .required_unless("list")
        .takes_value(true)
        .help("Specify the target name you want to query"))
    .arg(Arg::with_name("list")
        .long("--list")
        .required_unless("set target")
        .long_help("If used as the only flag with RHD read, you will be presented with all targets curently stored in the db. If used with -t/--target the RHD will return every entry for that target")
        .conflicts_with_all(&["set host", "status code"]))
    .arg(Arg::with_name("set host")
        .short("h")
        .long("--host")
        .multiple(true)
        .takes_value(true)
        .multiple(true)
        .help("Further modify the scope of the query -- shows only the results that have the corresponding host"))
    .arg(Arg::with_name("status code")
        .long("--status-code")
        .takes_value(true)
        .long_help("Further modify the scope of the query -- shows only the results that have the corresponding status code(s) -- you can specify multiple status codes separated by comma e.g. --status-code 200, 301")
         .multiple(true))
    .arg(Arg::with_name("path")
        .long("--path")
        .conflicts_with_all(&["path comb", "path comb strict"])
        .long_help("Displays only path part of the stored urls according to the previous query modifiers, e.g. RHD read -t 'target' --status-code 200  -- will only display 'path' portion of the urls that have the status code of 200"))
    .arg(Arg::with_name("url")
        .short("u")
        .long("--url")
        .help("Displays only url part of the stored entries according to the previous query modifiers")
        ))
.subcommand(SubCommand::with_name("mod")
    .arg(Arg::with_name("delete")
        .short("d")
        .long("--delete")
        .long_help("deletes every entry that mathes the filters applied. To see what you are deleting try 'read'ing the entries based on filter first."))
    .arg(Arg::with_name("set target")
        .short("t")
        .long("--target")
        .required_unless("list")
        .takes_value(true)
        .help("Specify the target name you want to query"))
    .arg(Arg::with_name("set host")
        .short("h")
        .long("--host")
        .multiple(true)
        .takes_value(true)
        .multiple(true)
        .help("Further modify the scope of the query - include only set hosts "))
    .arg(Arg::with_name("status code")
        .long("--status-code")
        .takes_value(true)
        .long_help("Further modify the scope of the query ")
        .multiple(true))
    .arg(Arg::with_name("id")
        .short("id")
        .takes_value(true)
        .multiple(true)
        .help("Further modify the scope of the query - include only matching id entries"))
    .arg(Arg::with_name("path comb")
        .long("--path-comb")
        .conflicts_with("delete")
        .long_help("If this flag is set, the RHD will try to combine all available paths in given target with hosts that match the specified filters"))
    );
// TODO add subcommand for "fn" like delete, path-comb, purge (ping site if they up or not
// responding and based on that purge them)  
    cli_args
}
