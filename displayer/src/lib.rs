use askama::Template;
use persister::Stats;
use std::fs::File;

#[derive(Template)]
#[template(path = "stats.html")]
pub struct StatsTemplate<'a> {
    active_7d: &'a str,
    active_30d: &'a str,
    active_12m: &'a str,
    active_all: &'a str,

    domain: &'a str,
    generated_at: &'a str,
    date_range: &'a str,
    total_requests: i32,
    unique_visitors: i32,
    human_requests: i32,
    human_percent: f64,
    bot_requests: i32,
    avg_response_time: f32,
    vpn_requests: i32,
    vpn_percent: f64,
    top_pages: Vec<TopPage>,
    top_countries: Vec<Country>,
    top_cities: Vec<City>,
    top_referrers: Vec<Referrer>,
}

struct TopPage {
    path: String,
    hits: i32,
    percent: f32,
}

struct Country {
    name: String,
    count: i32,
    percent: f32,
}

struct City {
    name: String,
    count: i32,
    percent: f32,
}

struct Referrer {
    url: String,
    count: i32,
    percent: f32,
}

pub struct Displayer {}

impl Displayer {
    pub fn get_template<'a>(&self, stats: Stats, host: &'a str) {
        let mut top_pages = vec![];
        let mut countries = vec![];
        let mut cities = vec![];
        let mut referrers = vec![];
        for page in stats.pages {
            top_pages.push(TopPage {
                path: page.0,
                hits: page.1,
                percent: 0.0,
            })
        }

        for country in stats.countries {
            countries.push(Country {
                name: country.0,
                count: country.1,
                percent: 0.0,
            })
        }
        for city in stats.cities {
            cities.push(City {
                name: city.0,
                count: city.1,
                percent: 0.0,
            })
        }

        for referrer in stats.referrers {
            referrers.push(Referrer {
                url: referrer.0,
                count: referrer.1,
                percent: 0.0,
            })
        }
        let res  = StatsTemplate {
            active_7d: "nah",
            active_30d: "nah",
            active_12m: "nah",
            active_all: "active",
            domain: host,
            generated_at: "today",
            date_range: "all time",
            total_requests: stats.total_requests,
            unique_visitors: stats.unique_visitors,
            human_requests: stats.human_requests,
            human_percent: f64::from(stats.total_requests) / f64::from(stats.human_requests),
            bot_requests: stats.total_requests - stats.human_requests,
            avg_response_time: stats.avg_response_time,
            vpn_requests: stats.vpn_requests,
            vpn_percent: f64::from(stats.total_requests) / f64::from(stats.vpn_requests),
            top_pages: top_pages,
            top_countries: countries,
            top_cities: cities,
            top_referrers: referrers,
        };
        let mut writer = File::create("stats.html").unwrap();
        res.write_into(&mut writer).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(4, 4);
    }
}
