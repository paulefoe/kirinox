use kirinox::ArgsConfig;
use kirinox::read_logs;
use std::{env, process};


fn main() {
    let config = ArgsConfig::from_env(env::args()).unwrap_or_else(|err| {
        eprintln!("There was an error loading the config: {err}");
        process::exit(1);
    });
    read_logs(&config.nginx_log_path).unwrap();
}
