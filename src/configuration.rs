pub fn get_conf(path: Option<&str>) -> Result<DatabaseSettings, config::ConfigError>{
    let default_path = std::env::current_dir()
    .expect("Failed to determine current directory")
    .join("configuration");
    
    let file_path = match path{
        Some(p) => std::path::Path::new(p).to_owned(),
        None => default_path
    };
    //let s = std::path::Path::new("str").to_owned();
    let mut settings = config::Config::default();
    match settings.merge(config::File::from(file_path)) {
        Ok(conf) => conf.clone().try_into(),
        Err(why) => Err(why)
    }

}

#[derive(serde::Deserialize, Debug)]
pub struct DatabaseSettings {
    pub username: String, 
    pub password: String, 
    pub port: u16, 
    pub host: String, 
    pub database_name: String,
}

impl DatabaseSettings{
    pub fn get_connection_string(&self) -> String {
        format!("host={} dbname={} user={} password={}", self.host, self.database_name, self.username, self.password)
    }
}
