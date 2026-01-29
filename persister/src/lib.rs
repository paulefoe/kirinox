use std::path::PathBuf;

use enricher::EnrichedLog;
use parser::LogStruct;

use rusqlite::{Connection, OptionalExtension, Params, Result, params};

pub struct Db {
    connection: Connection,
}

impl Db {
    pub fn new() -> Db {
        let path = PathBuf::from("./krx.db");
        if path.exists() {
            return Db {
                connection: Connection::open("krx.db").unwrap(),
            };
        };
        let con = Connection::open("krx.db").unwrap();
        con.execute(
            "CREATE TABLE IF NOT EXISTS access_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    -- request info
    remote_addr TEXT NOT NULL,
    remote_user TEXT,
    timestamp INTEGER NOT NULL,
    method TEXT NOT NULL,
    scheme TEXT NOT NULL,
    http_host TEXT NOT NULL,
    request_uri TEXT NOT NULL,
    server_protocol TEXT NOT NULL,
    status INTEGER NOT NULL,
    body_bytes_sent INTEGER NOT NULL,
    request_time REAL NOT NULL,
    upstream_response_time REAL,

    http_referer TEXT,
    http_user_agent TEXT,

    -- enrichment
    is_bot INTEGER NOT NULL CHECK (is_bot IN (0,1)),
    country TEXT,
    city TEXT,
    is_vpn INTEGER NOT NULL CHECK (is_vpn IN (0,1)),

    -- useful indexes
    created_at TEXT DEFAULT (datetime('now'))
);",
            (),
        )
        .unwrap();
        Db { connection: con }
    }

    pub fn insert_record(&self, log_struct: &LogStruct, enriched_log_struct: &EnrichedLog) {
        self.connection.execute(
            "INSERT INTO access_log (
            remote_addr, timestamp, method, scheme,
            http_host, request_uri, server_protocol, status,
            body_bytes_sent, request_time, upstream_response_time,
            http_referer, http_user_agent,
            is_bot, country, city, is_vpn
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);
                ",
            params![
                log_struct.remote_addr,
                log_struct.dt,
                log_struct.method,
                log_struct.scheme,
                log_struct.http_host,
                log_struct.request_uri,
                log_struct.server_protocol,
                log_struct.status,
                log_struct.body_bytes_sent,
                log_struct.request_time,
                log_struct.upstream_response_time,
                log_struct.http_refferer,
                log_struct.http_user_agent,
                enriched_log_struct.is_bot,
                enriched_log_struct.country,
                enriched_log_struct.city,
                enriched_log_struct.is_vpn,
            ],
        ).unwrap();
    }

    pub fn fetch_last_known_entry_date(&self) -> Option<i64> {
        let mut query = self.connection.prepare(
            "SELECT max(timestamp) from access_log;"
            ).unwrap();
        query.query_one([], |x| x.get(0)).optional().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
