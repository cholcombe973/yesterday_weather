#[macro_use]
extern crate clap;
extern crate reqwest;
extern crate rusqlite;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::path::Path;

use clap::{Arg, App};
use rusqlite::Connection;


#[derive(Debug, Deserialize)]
pub struct ResponseFeature {
    pub yesterday: i64,
}

#[derive(Debug, Deserialize)]
pub struct JsonResponse {
    pub version: String,
    #[serde(rename(deserialize = "termsofService"))]
    pub terms_of_service: String,
    pub features: ResponseFeature,
}

#[derive(Debug, Deserialize)]
pub struct HistoryDate {
    pub pretty: String,
    pub year: String,
    pub mon: String,
    pub mday: String,
    pub hour: String,
    pub min: String,
    pub tzname: String,
}

#[derive(Debug, Deserialize)]
pub struct Observation {
    pub date: HistoryDate,
    pub utcdate: HistoryDate,
    // Metric
    pub tempm: String,
    //Imperial
    pub tempi: String,
    pub dewptm: String,
    pub dewpti: String,
    pub hum: String,
    pub wspdm: String,
    pub wspdi: String,
    pub wgustm: String,
    pub wgusti: String,
    pub wdird: String,
    pub wdire: String,
    pub vism: String,
    pub visi: String,
    pub pressurem: String,
    pub pressurei: String,
    pub windchillm: String,
    pub windchilli: String,
    pub heatindexm: String,
    pub heatindexi: String,
    pub precipm: String,
    pub precipi: String,
    pub conds: String,
    pub icon: String,
    pub fog: String,
    pub rain: String,
    pub snow: String,
    pub hail: String,
    pub thunder: String,
    pub tornado: String,
    pub metar: String,
}

#[derive(Debug, Deserialize)]
pub struct Dailysummary {
    pub date: HistoryDate,
    pub fog: String,
    pub rain: String,
    pub snow: String,
    pub snowfallm: String,
    pub snowfalli: String,
    pub monthtodatesnowfallm: String,
    pub monthtodatesnowfalli: String,
    pub since1julsnowfallm: String,
    pub since1julsnowfalli: String,
    pub snowdepthm: String,
    pub snowdepthi: String,
    pub hail: String,
    pub thunder: String,
    pub tornado: String,
    pub meantempm: String,
    pub meantempi: String,
    pub meandewptm: String,
    pub meandewpti: String,
    pub meanpressurem: String,
    pub meanpressurei: String,
    pub meanwindspdm: String,
    pub meanwindspdi: String,
    pub meanwdire: String,
    pub meanwdird: String,
    pub meanvism: String,
    pub meanvisi: String,
    pub humidity: String,
    // Metric
    pub maxtempm: String,
    // Imperial
    pub maxtempi: String,
    pub mintempm: String,
    pub mintempi: String,
    pub maxhumidity: String,
    pub minhumidity: String,
    pub maxdewptm: String,
    pub maxdewpti: String,
    pub mindewptm: String,
    pub mindewpti: String,
    pub maxpressurem: String,
    pub maxpressurei: String,
    pub minpressurem: String,
    pub minpressurei: String,
    pub maxwspdm: String,
    pub maxwspdi: String,
    pub minwspdm: String,
    pub minwspdi: String,
    pub maxvism: String,
    pub maxvisi: String,
    pub minvism: String,
    pub minvisi: String,
    pub gdegreedays: String,
    pub heatingdegreedays: String,
    pub coolingdegreedays: String,
    pub precipm: String,
    pub precipi: String,
    pub precipsource: String,
    pub heatingdegreedaysnormal: String,
    pub monthtodateheatingdegreedays: String,
    pub monthtodateheatingdegreedaysnormal: String,
    pub since1sepheatingdegreedays: String,
    pub since1sepheatingdegreedaysnormal: String,
    pub since1julheatingdegreedays: String,
    pub since1julheatingdegreedaysnormal: String,
    pub coolingdegreedaysnormal: String,
    pub monthtodatecoolingdegreedays: String,
    pub monthtodatecoolingdegreedaysnormal: String,
    pub since1sepcoolingdegreedays: String,
    pub since1sepcoolingdegreedaysnormal: String,
    pub since1jancoolingdegreedays: String,
    pub since1jancoolingdegreedaysnormal: String,
}

#[derive(Debug, Deserialize)]
pub struct History {
    pub date: HistoryDate,
    pub utcdate: HistoryDate,
    pub observations: Vec<Observation>,
    pub dailysummary: Vec<Dailysummary>,
}

#[derive(Debug, Deserialize)]
pub struct Response {
    pub response: JsonResponse,
    pub history: History,
}

fn save_to_database(data: &mut Response, db: &Path) -> Result<(), rusqlite::Error> {
    let summary = data.history
        .dailysummary
        .pop()
        .unwrap();
    let max: i16 = match summary.maxtempi.parse() {
        Ok(value) => value,
        Err(e) => {
            println!("maxtempi i16 parse failed with error: {}", e);
            0
        }
    };
    let min: i16 = match summary.mintempi.parse() {
        Ok(value) => value,
        Err(e) => {
            println!("mintempi i16 parse failed with error: {}", e);
            0
        }
    };
    let heat_units: i16 = ((max + min) / 2) - 55;
    let rainfall: f64 = match summary.precipi.parse() {
        Ok(value) => value,
        Err(e) => {
            println!("rainfall f32 parse failed with error: {}", e);
            0.00
        }
    };

    let conn = Connection::open(db)?;
    let _ = conn.execute("create table if not exists heatunits (temp_min int, temp_max int,date \
                  datetime,heat_units int, rainfall int, primary key (date))",
                         &[])?;

    conn.execute("insert into heatunits values (?, ?, ?, ?, ?)",
                 &[&max,
                   &min,
                   &format!("{year}-{month}-{day} {hour}:{minute}",
                            year = summary.date.year,
                            month = summary.date.mon,
                            day = summary.date.mday,
                            hour = summary.date.hour,
                            minute = summary.date.min),
                   &heat_units,
                   &rainfall])?;
    Ok(())
}

fn main() {
    let matches = App::new("Yesterdays Weather")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Logs yesterdays weather to sqlite")
        .arg(Arg::with_name("api_key")
                 .short("k")
                 .help("The weather underground api key to use")
                 .required(true)
                 .takes_value(true))
        .arg(Arg::with_name("database")
                 .default_value("heat_units.sqlite3")
                 .short("d")
                 .help("The sqlite database to save to")
                 .required(true)
                 .takes_value(true))
        .get_matches();
    let api_key = matches.value_of("api_key").expect("Failed to get api key");
    let database = matches.value_of("database").unwrap();

    let mut resp = reqwest::get(&format!("https://api.wunderground.\
                                               com/api/{}/yesterday/q/OR/Fairview.json",
                                         api_key))
            .expect("Failed to query weather underground");
    if resp.status().is_success() {
        let mut yesterday_weather: Response =
            resp.json().expect("Failed to decode wunderground json response");
        //println!("Weather response: {:?}", yesterday_weather);
        match save_to_database(&mut yesterday_weather, &Path::new(&database)) {
            Err(e) => {
                println!("Failed to save response to database: {}", e);
                return;
            }
            _ => {}
        }
    } else {
        println!("Query to weather underground failed with error: {:?}",
                 resp.status().canonical_reason());
    }
}
