use flate2::read::GzDecoder;
use std::{
    fs::{self, File, remove_file}, io::{BufRead, BufReader, Error, Read, Seek, SeekFrom, copy}, num::{ParseFloatError, ParseIntError}, os, path::PathBuf
};
use tar::Archive;

use chrono::DateTime;

#[derive(Debug)]
pub struct LogFile {
    is_created: bool,
    file_path: PathBuf,
}

#[derive(Debug)]
pub struct FileRecordingInfo {
    is_recorded: bool,
    pub start_from: u64,
    pub file_path: PathBuf,
    is_created: bool,
}

#[derive(Debug)]
pub struct LogStruct<'a> {
    pub remote_addr: &'a str,
    pub remote_user: Option<&'a str>,
    pub dt: i64,
    pub method: &'a str,
    pub scheme: &'a str,
    pub http_host: &'a str,
    pub request_uri: &'a str,
    pub server_protocol: &'a str,
    pub status: u16,
    pub body_bytes_sent: u64,
    pub request_time: f64,
    pub upstream_response_time: Option<f64>,
    pub http_refferer: Option<&'a str>,
    pub http_user_agent: &'a str,
}

pub struct Parser {
    logs_path: PathBuf,
}

impl Parser {
    pub fn new(path: &PathBuf) -> Result<Parser, &'static str> {
        if !path.exists() || !path.is_dir() {
            return Err("logs path do not exist or the path is not a dir");
        }
        Ok(Parser {
            logs_path: path.clone(),
        })
    }
    fn find_number(&self, f: &PathBuf) -> i32 {
        let file_name = f.file_name().unwrap().to_str().unwrap();
        let splitted_line: Vec<&str> = file_name.split(".").collect();
        if file_name.ends_with("gz") {
            return splitted_line[splitted_line.len() - 2]
                .parse::<i32>()
                .unwrap();
        }
        splitted_line[splitted_line.len() - 1]
            .parse::<i32>()
            .unwrap_or(0)
    }

    fn find_end_and_begining_of_file(
        &self,
        opened_file: &mut File,
        last_recorded_ts: i64,
    ) -> Result<(Option<String>, Option<String>), Error> {
        let first_line;
        //let last_line;
        let metadata = opened_file.metadata()?;
        let file_len = metadata.len();
        // small file read all of i
        if file_len < 500 {
            let mut file_contents = String::new();
            opened_file.read_to_string(&mut file_contents).unwrap();
            let mut splitted = file_contents.split('\n');
            first_line = splitted.next().map(String::from);
            let _ = splitted.next_back();
            return Ok((None, None));
            //last_line = splitted.next_back().map(String::from);
        }
        //} else {
        //    let mut end_of_the_file = [0; 200];
        //    let mut start_of_file = [0; 200];
        //    opened_file.read(&mut start_of_file)?;
        //
        //    opened_file.seek(SeekFrom::End(0))?;
        //    opened_file.read(&mut end_of_the_file).unwrap();
        //
        //    let eof_str = String::from_utf8_lossy(&end_of_the_file);
        //    let sof_str = String::from_utf8_lossy(&start_of_file);
        //    first_line = sof_str.split('\n').next().map(String::from);
        //    let mut splitted_eof = eof_str.split('\n');
        //    let _ = splitted_eof.next_back();
        //    //last_line = splitted_eof.next_back().map(String::from);
        //}
        //let pos = self.find_exact_position(opened_file, file_len, last_recorded_ts);

        Ok((None, None))
    }

    fn search_small_file(
        &self,
        reader: &mut BufReader<File>,
        path: PathBuf,
        last_recorded_ts: i64,
        is_created: bool,
    ) -> Result<FileRecordingInfo, Error> {
        let mut first_line = String::new();
        reader.read_line(&mut first_line)?;
        let first_structured_line = LogStruct::from_line(&first_line).unwrap();
        let mut first_dt = first_structured_line.dt;
        for line in reader.lines() {
            let next_line = line?;
            let next_structured_line = LogStruct::from_line(&next_line).unwrap();
            if (next_structured_line.dt >= last_recorded_ts)
                && (last_recorded_ts >= first_structured_line.dt)
            {
                println!("FOUND IT in a small file");
                return Ok(FileRecordingInfo {
                    is_recorded: false,
                    start_from: reader.stream_position()?,
                    file_path: path,
                    is_created: is_created,
                });
            }
            first_dt = next_structured_line.dt;
        }
        if first_dt <= last_recorded_ts {
            return Ok(FileRecordingInfo {
                is_recorded: true,
                start_from: 0,
                file_path: path,
                is_created: is_created,
            });
        } else {
            return Ok(FileRecordingInfo {
                is_recorded: false,
                start_from: 0,
                file_path: path,
                is_created: is_created,
            });
        }
    }

    fn find_exact_position(
        &self,
        f: &PathBuf,
        last_recorded_ts: i64,
    ) -> Result<FileRecordingInfo, Error> {
        let path = self.get_log_file(f)?;
        let opened_file = File::open(&path.file_path)?;
        let metadata = opened_file.metadata()?;
        let mut reader = BufReader::new(opened_file);
        let file_len = metadata.len();
        if file_len < 500 {
            return self.search_small_file(&mut reader, path.file_path, last_recorded_ts, path.is_created);
        }
        let mut start = 0;
        let mut end = file_len;
        let mut mid = end / 2;
        let mut first_line = String::new();
        reader.read_line(&mut first_line)?;
        reader.seek(SeekFrom::End(-200))?;

        let mut eof = String::new();
        reader.read_to_string(&mut eof)?;
        let mut splitted_end = eof.split('\n');
        splitted_end.next_back();
        let last_line = splitted_end.next_back();
        let within = self.is_log_within_file(&first_line, last_line, last_recorded_ts);
        if within == -1 {
            return Ok(FileRecordingInfo {
                is_recorded: true,
                start_from: 0,
                file_path: path.file_path,
                is_created: path.is_created,
            });
        } else if within == 0 {
            return Ok(FileRecordingInfo {
                is_recorded: false,
                start_from: 0,
                file_path: path.file_path,
                is_created: path.is_created,
            });
        }
        loop {
            let mut sof_str = String::new();
            let mut next_line = String::new();
            reader.seek(SeekFrom::Start(mid))?;

            // first line we can land in the middle of the line, so it would be incomplete log,
            // discard it;
            reader.read_line(&mut String::new())?;
            reader.read_line(&mut sof_str)?;
            reader.read_line(&mut next_line)?;
            //reader.read_line(&mut next_line)?;

            //let sof_str = String::from_utf8_lossy(&start_of_file);
            //let mut splitted = sof_str.split('\n');
            //splitted.next().unwrap();
            //let new_start = splitted.next().map(String::from).unwrap();
            println!("before");
            println!("first: {}", sof_str);
            println!("second: {}", next_line);
            let first_parsed_line = LogStruct::from_line(&sof_str).unwrap();
            let second_parsed_line = LogStruct::from_line(&next_line).unwrap();
            println!(
                "{} - {} - {}",
                first_parsed_line.dt, last_recorded_ts, second_parsed_line.dt
            );
            if (second_parsed_line.dt >= last_recorded_ts)
                && (last_recorded_ts >= first_parsed_line.dt)
            {
                println!("FOUND IT");
                return Ok(FileRecordingInfo {
                    is_recorded: false,
                    start_from: reader.stream_position()?,
                    file_path: path.file_path,
                    is_created: path.is_created,
                });
            }
            if first_parsed_line.dt < last_recorded_ts {
                start = mid;
                mid = (start + end) / 2;
            } else {
                end = mid;
                mid = (start + end) / 2;
            }
        }
    }

    fn get_log_file(&self, file_path: &PathBuf) -> Result<LogFile, Error> {
        // unarchive the file and return the path to it, if it's not an archive just return the
        // path
        if file_path.extension().unwrap() != "gz" {
            return Ok(LogFile {
                is_created: false,
                file_path: file_path.clone(),
            });
        }
        let input = BufReader::new(File::open(file_path)?);
        let new_path = file_path.with_extension("");
        let mut output = File::create(&new_path)?;
        let mut decoder = GzDecoder::new(input);
        copy(&mut decoder, &mut output)?;
        Ok(LogFile {
            is_created: true,
            file_path: new_path,
        })
    }

    fn is_log_within_file(
        &self,
        first_line: &str,
        last_line: Option<&str>,
        last_recorded_ts: i64,
    ) -> i32 {
        // 1 = log within this file
        // -1 = log is already recorded
        // 0 = log are older than this file
        let first_parsed_line = LogStruct::from_line(first_line).unwrap();
        let last_line = last_line.unwrap_or(first_line);
        let last_parsed_line = LogStruct::from_line(&last_line).expect("second line failed");
        println!(
            "{} = {} = {}",
            first_parsed_line.dt, last_recorded_ts, last_parsed_line.dt
        );
        if (first_parsed_line.dt <= last_recorded_ts && last_parsed_line.dt >= last_recorded_ts) {
            return 1;
        } else if last_parsed_line.dt < last_recorded_ts {
            return -1;
        } else {
            return 0;
        }
    }

    fn get_unrecorded_files(
        &self,
        files: &Vec<PathBuf>,
        last_recorded_ts: i64,
    ) -> Result<Vec<FileRecordingInfo>, Error> {
        let mut unrecorded_files: Vec<FileRecordingInfo> = vec![];
        for f in files {
            println!("file: {:?}", f);
            let file_recording_info = self.find_exact_position(f, last_recorded_ts)?;
            println!("file_info: {:?}", file_recording_info);
            if !file_recording_info.is_recorded {
                unrecorded_files.push(file_recording_info);
            }
        }
        Ok(unrecorded_files)
    }

    pub fn find_files(&self, last_recorded_ts: Option<i64>) -> Vec<FileRecordingInfo> {
        // [
        // "nginx-logs/paulefou/access.log.2.gz",
        // "nginx-logs/paulefou/access.log.1",
        // "nginx-logs/paulefou/access.log"
        // ]
        let mut files_list: Vec<PathBuf> = self
            .logs_path
            .read_dir()
            .unwrap()
            .map(|f| f.unwrap().path())
            .filter(|f| f.file_name().unwrap().to_str().unwrap().contains("access"))
            .collect();
        files_list.sort_by(|a, b| self.find_number(a).cmp(&self.find_number(b)));
        let last_recorded_ts = last_recorded_ts.unwrap_or(0);
        return self
            .get_unrecorded_files(&files_list, last_recorded_ts)
            .unwrap();
    }
    pub fn clean_up(&self, files: Vec<FileRecordingInfo>) -> Result<(), Error> {
        for file in files {
            if file.is_created {
                remove_file(file.file_path)?;
            }
        }
        Ok(())
    }
}

impl<'a> LogStruct<'a> {
    pub fn from_line<'b>(line: &'b str) -> Result<LogStruct<'b>, &'static str> {
        let splitted_line: Vec<&str> = line.split("\t").collect();
        if let [remote_addr, remote_user, time_iso8601, method, scheme, http_host, request_uri, server_protocol, status, body_bytes_sent, request_time, upstream_response_time, http_refferer, http_user_agent] =
            splitted_line[..]
        {
            let dt = DateTime::parse_from_str(time_iso8601, "%Y-%m-%dT%H:%M:%S%z");
            if !dt.is_ok() {
                return Err("could not parse date");
            }
            let dt = dt.unwrap().timestamp_millis();
            let body_bytes_sent: Result<u64, ParseIntError> = body_bytes_sent.parse();
            if body_bytes_sent.is_err() {
                eprintln!("{}", body_bytes_sent.err().unwrap());
                return Err("could not parse the body_bytes_sent");
            }
            let body_bytes_sent = body_bytes_sent.unwrap();

            let status: Result<u16, ParseIntError> = status.parse();
            if status.is_err() {
                eprintln!("{}", status.err().unwrap());
                return Err("could not parse the status");
            }
            let status = status.unwrap();

            let request_time: Result<f64, ParseFloatError> = request_time.parse();
            if request_time.is_err() {
                eprintln!("{}", request_time.err().unwrap());
                return Err("could not parse the request_time");
            }
            let request_time = request_time.unwrap();

            println!("{}", upstream_response_time);
            let upstream_response_time: Result<f64, ParseFloatError> =
                upstream_response_time.parse();
            let upstream_response_time = upstream_response_time.ok();

            let mut parsed_remote_user: Option<&'b str> = None;
            if remote_user != "-" {
                parsed_remote_user = Some(remote_user)
            }

            let mut parsed_http_referer: Option<&'b str> = None;
            if http_refferer != "-" {
                parsed_http_referer = Some(http_refferer);
            }

            Ok(LogStruct {
                remote_addr,
                remote_user: parsed_remote_user,
                dt,
                method,
                scheme,
                http_host,
                request_uri,
                server_protocol,
                status,
                body_bytes_sent,
                request_time,
                upstream_response_time,
                http_refferer: parsed_http_referer,
                http_user_agent,
            })
        } else {
            println!(
                "errors here splitted line collected: \n\n{:?}\n\n\n\n\n\n",
                splitted_line
            );
            return Err("could not parse the line");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
