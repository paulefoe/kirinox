use std::env::Args;
use std::fs;
use std::io::Error;
use std::path::PathBuf;
use parser::{self, LogStruct};

#[derive(Debug)]
pub struct ArgsConfig {
    pub nginx_log_path: PathBuf,
    pub analytics_output_html: PathBuf,
}

impl ArgsConfig {
    pub fn from_env(mut args: Args) -> Result<Self, &'static str> {
        if args.len() < 3 {
            return Err("not enough arguments");
        }
        let _ = args.next();
        let logs_path = PathBuf::from(args.next().unwrap());
        if !logs_path.exists() {
            return Err("no logs were found at the provided path");
        }
        Ok(ArgsConfig {
            nginx_log_path: logs_path,
            analytics_output_html: PathBuf::from(args.next().unwrap()),
        })
    }
}

pub fn read_logs(log_path: &PathBuf) -> Result<i32, Error> {
    let contents = fs::read_to_string(log_path)?;
    for line in contents.split("\n") {
        let log_struct = LogStruct::from_line(line);
        println!("{:?}", log_struct);
    }
    Ok(10)
}
