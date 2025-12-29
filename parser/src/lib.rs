use std::{
    io::ErrorKind,
    num::{ParseFloatError, ParseIntError},
};

use chrono::{DateTime, FixedOffset};

#[derive(Debug)]
pub struct LogStruct<'a> {
    pub remote_addr: &'a str,
    pub remote_user: Option<&'a str>,
    pub dt: DateTime<FixedOffset>,
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
            let dt = dt.unwrap();
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
            println!("{:?}", splitted_line);
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
