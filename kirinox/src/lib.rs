use std::{env::Args, fs::File};
use std::fs;
use std::io::{BufRead, BufReader, Error, Seek, SeekFrom};
use std::path::PathBuf;
use parser::{self, LogStruct, Parser};
use enricher::Enricher;
use persister::Db;

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
    let enricher = Enricher::new();
    let persister = Db::new();
    let parser = Parser::new(log_path).unwrap();
    let last_recorded_ts = persister.fetch_last_known_entry_date();
    let files = parser.find_files(last_recorded_ts);
    for file in files.iter().rev() {
        let opened_file = File::open(&file.file_path)?;
        let mut line_results: Vec<String> = vec![];
        let mut reader = BufReader::new(opened_file);
        let mut buf = String::new();
        while reader.read_line(&mut buf)? != 0 {
            if file.start_from != 0 {
                if reader.stream_position()? >= file.start_from {
                    break;
                }
            }
            println!("current file: {:?}, \ncurrent line: {}", file.file_path, buf);
            line_results.push(buf.clone());
            buf.clear();
        }
        println!("line restults are {:?}", line_results);
        for line in line_results.iter().rev() {
            if let Ok(log_struct) = LogStruct::from_line(line) {
                let enriched_log = enricher.enrich(&log_struct);
                persister.insert_record(&log_struct, &enriched_log);
            }
        }
    }
    parser.clean_up(files)?;
    Ok(10)
}
