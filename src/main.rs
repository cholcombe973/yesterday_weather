use std::path::Path;
use std::str::FromStr;

use chrono::prelude::*;
use chrono::{DateTime, Duration, Local, NaiveDateTime};
use clap::{crate_authors, crate_version, App, AppSettings, Arg, ArgMatches};
use forecast::{ApiClient, ExcludeBlock, Lang, TimeMachineRequestBuilder, Units};
use reqwest::Client;
use rusqlite::{Connection, params};
use serde_json::Value;

fn save_to_database(data: Value, db: &Path) -> Result<(), rusqlite::Error> {
    let data = data.as_object().expect("Invalid json object returned");

    let max: f64 = match data.get("temperatureMax").unwrap().as_f64() {
        Some(value) => value,
        None => {
            println!("temperatureMax f64 parse failed");
            0.00
        }
    };
    let min: f64  = match data.get("temperatureMin").unwrap().as_f64() {
        Some(value) => value,
        None => {
            println!("mintempi f64 parse failed");
            0.00
        }
    };
    let heat_units = ((max + min) / 2_f64) - 55_f64;
    let rainfall: f64 = match data.get("precipIntensityMax").unwrap().as_f64() {
        Some(value) => value,
        None => {
            println!("rainfall f64 parse failed");
            0.00
        }
    };

    let conn = Connection::open(db)?;
    let _ = conn.execute(
        "create table if not exists heatunits (temp_min int, temp_max int,date \
         datetime,heat_units int, rainfall int, primary key (date))",
        params![],
    )?;

    conn.execute(
        "insert into heatunits values (?, ?, date('now', '-1 day'), ?, ?)",
        &[
            &max,
            &min,
            &heat_units,
            &rainfall,
        ],
    )?;
    Ok(())
}

fn get_matches<'a>() -> ArgMatches<'a> {
    App::new("Yesterdays Weather")
        .setting(AppSettings::AllowLeadingHyphen)
        .version(crate_version!())
        .author(crate_authors!())
        .about("Logs yesterdays weather to sqlite")
        .arg(
            Arg::with_name("api_key")
                .long("api_key")
                .help("The darksky api key to use")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("database")
                .default_value("heat_units.sqlite3")
                .long("database")
                .help("The sqlite database to save to")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("lat")
                .long("lat")
                .help("The latitude to check")
                .required(true)
                .takes_value(true)
                .validator(|lat| match f64::from_str(&lat) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(format!(
                        "latitude must be a floating point: {}",
                        e.to_string()
                    )),
                }),
        )
        .arg(
            Arg::with_name("long")
                .long("long")
                .help("The longitude to check")
                .required(true)
                .takes_value(true)
                .validator(|lat| match f64::from_str(&lat) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(format!(
                        "longitude must be a floating point: {}",
                        e.to_string()
                    )),
                }),
        )
        .get_matches()
}

fn get_forecast(api_key: &str, latitude: f64, longitude: f64) -> Result<Value, String> {
    let reqwest_client = Client::new();
    let api_client = ApiClient::new(&reqwest_client);
    // Find yesterday's timestamp so we can send that to darksky
    let yesterday_time: DateTime<Local> = Local::now() - Duration::hours(24);
    let native_time = NaiveDateTime::from_timestamp(yesterday_time.timestamp(), 0);
    let yesterday: DateTime<Utc> = DateTime::from_utc(native_time, Utc);

    let mut blocks = vec![
        ExcludeBlock::Alerts,
        ExcludeBlock::Currently,
        ExcludeBlock::Hourly,
        ExcludeBlock::Minutely,
    ];
    let time_machine_request =
        TimeMachineRequestBuilder::new(api_key, latitude, longitude, yesterday.timestamp() as u64)
            .exclude_block(ExcludeBlock::Hourly)
            .exclude_blocks(&mut blocks)
            .lang(Lang::English)
            .units(Units::Imperial)
            .build();
    let forecast = api_client
        .get_time_machine(time_machine_request)
        .map_err(|e| e.to_string())?;
    let res: Value = reqwest::get(forecast.url().as_str())
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())?;

    let weather = res["daily"]["data"][0].clone();
    if weather.is_null() {
        return Err(format!(
            "darksky api returned null for url: {}",
            forecast.url()
        ));
    }
    Ok(weather)
}

fn main() {
    let matches = get_matches();
    let api_key = matches.value_of("api_key").expect("Failed to get api key");
    let lat = matches.value_of("lat").expect("Failed to get latitude");
    let long = matches.value_of("long").expect("Failed to get longitude");
    let database = matches.value_of("database").unwrap();

    let lat = f64::from_str(&lat).unwrap();
    let long = f64::from_str(&long).unwrap();
    let forecast = match get_forecast(&api_key, lat, long) {
        Ok(f) => f,
        Err(e) => {
            println!("error getting forecast: {}", e);
            return;
        }
    };
    match save_to_database(forecast, &Path::new(&database)) {
        Err(e) => {
            println!("Failed to save response to database: {}", e);
            return;
        }
        _ => {}
    }
}
