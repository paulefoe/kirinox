use std::time::Duration;

use parser::LogStruct;
use serde::Deserialize;
use ureq::Agent;

#[derive(Debug)]
pub struct EnrichedLog {
    pub is_bot: bool,
    pub country: String,
    pub city: String,
    pub is_vpn: bool,
}

const BOT_ASNS: [&'static str; 24] = [
    "as15169", "as8075", "as16509", "as14618", "as714", "as13238", "as38365", "as40509", "as32934",
    "as13414", "as14413", "as396982", "as209366", "as26347", "as24940", "as203020", "as45090",
    "as14061", "as16276", "as63949", "as20473", "as45102", "as398705", "as64512",
];

const BOT_USER_AGENTS: [&'static str; 54] = [
    "googlebot",
    "bingbot",
    "slurp",
    "duckduckbot",
    "baiduspider",
    "yandexbot",
    "applebot",
    "facebookexternalhit",
    "facebot",
    "twitterbot",
    "linkedinbot",
    "ahrefsbot",
    "semrushbot",
    "mj12bot",
    "dotbot",
    "rogerbot",
    "seokicks",
    "screaming frog",
    "petalbot",
    "ccbot",
    "censys",
    "shodan",
    "zgrab",
    "nmap",
    "masscan",
    "python-requests",
    "curl",
    "wget",
    "httpclient",
    "go-http-client",
    "java/",
    "libwww-perl",
    "scrapy",
    "axios",
    "node-fetch",
    "okhttp",
    "postmanruntime",
    "headlesschrome",
    "phantomjs",
    "puppeteer",
    "playwright",
    "selenium",
    "chrome-lighthouse",
    "uptimerobot",
    "statuscake",
    "pingdom",
    "newrelicpinger",
    "datadog",
    "elastic uptime",
    "monitoring",
    "bot",
    "crawler",
    "spider",
    "scanner",
];

pub struct Enricher {
    client: Agent,
}

#[derive(Deserialize, Debug)]
struct IpData {
    #[serde(rename = "as")]
    asn: String,
    asname: String,
    city: String,
    country: String,
    #[serde(rename = "countryCode")]
    country_code: String,
    hosting: bool,
    isp: String,
    lat: f32,
    lon: f32,
    org: String,
    proxy: bool,
    query: String,
    region: String,
    #[serde(rename = "regionName")]
    region_name: String,
    status: String,
    timezone: String,
    zip: String,
}

impl Enricher {
    pub fn new() -> Enricher {
        Enricher {
            client: Agent::config_builder()
                .timeout_global(Some(Duration::from_secs(5)))
                .build()
                .into(),
        }
    }

    fn fetch_ip_data(&self, ip_addr: &str) -> IpData {
        self
            .client
            .get(format!("http://ip-api.com/json/{}?fields=status,message,country,countryCode,region,regionName,city,zip,lat,lon,timezone,isp,org,as,asname,proxy,hosting,query", ip_addr))
            .call()
            .unwrap()
            .body_mut()
            .read_json::<IpData>().unwrap()
    }

    fn is_bot(&self, log_line: &LogStruct, ip_data: &IpData) -> bool {
        ip_data.hosting
            || BOT_ASNS
                .iter()
                .any(|asn| ip_data.asn.to_lowercase().contains(asn))
            || BOT_USER_AGENTS
                .iter()
                .any(|ua| log_line.http_user_agent.to_lowercase().contains(ua))
    }

    fn is_vpn(&self, ip_data: &IpData) -> bool {
        ip_data.proxy
    }

    pub fn enrich(&self, log_line: &LogStruct) -> EnrichedLog {
        let ip_data: IpData = self.fetch_ip_data(log_line.remote_addr);
        let is_bot = self.is_bot(log_line, &ip_data);
        let is_vpn = self.is_vpn(&ip_data);
        EnrichedLog {
            is_bot,
            is_vpn,
            country: ip_data.country,
            city: ip_data.city
        }

    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
