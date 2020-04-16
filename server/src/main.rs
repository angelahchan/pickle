use std::env;
use std::net::IpAddr;
use std::error::Error;
use warp::{get, path, Reply, Filter, Rejection};
use database::Database;
use util::{fail, with_ip, not_found};

mod database;
mod geolocation;
mod util;

// TODO: Memoize most of the endpoints.

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize the logger.

    badlog::init_from_env("LOG_LEVEL");

    // Get the environment variables.

    let ip = env::var("IP").unwrap_or("127.0.0.1".into());
    let port = env::var("PORT").unwrap_or("8000".into());
    let database_url = env::var("DATABASE_URL")?;

    // Parse the environment variables.

    let ip: IpAddr = ip.parse()?;
    let port: u16 = port.parse()?;

    // Initialize the database pool.

    let db = Database::new(&database_url).await?;

    // Define the static files routes.

    let static_files = warp::fs::dir("static").or(warp::fs::file("static/index.html"));

    // Define the data routes.

    let data = warp::path("data").and(
        (get() .and(db.with()) .and(path!("region"))                         .and_then(get_regions)                   ).or
        (get() .and(with_ip()) .and(path!("region" / "current"))             .and_then(get_current_region)            ).or
        (get() .and(db.with()) .and(path!("region" / String))                .and_then(get_region_by_id)              ).or
        (get() .and(db.with()) .and(path!("stats" / String))                 .and_then(get_stats_by_disease)          ).or
        (get() .and(db.with()) .and(path!("stats" / String / "in" / String)) .and_then(get_stats_by_disease_in_region)).or
        (not_found())
    );

    // Start the server.

    let routes = data.or(static_files);

    Ok(warp::serve(routes).run((ip, port)).await)
}

async fn get_regions(db: Database) -> Result<impl Reply, Rejection> {
    let conn = db.get().await.map_err(fail)?;

    #[derive(serde::Serialize)]
    struct Region {
        id: String,
        name: String,
        the: bool,
    }

    let stmt = "SELECT id, name, geometry FROM region";

    let rows = conn.query(stmt, &[]).await.map_err(fail)?;

    let result = rows
        .iter()
        .map(|row| Region {
            id: row.get(0),
            name: row.get(1),
            the: row.get(2),
        })
        .collect::<Vec<_>>();

    Ok(warp::reply::json(&result))
}

async fn get_current_region(ip: Option<IpAddr>) -> Result<impl Reply, Rejection> {
    let country = match ip.and_then(|ip| geolocation::guess_by_ip(ip)) {
        Some(geolocation::Guess { country, subdivision: None }) => country,
        Some(geolocation::Guess { mut country, subdivision: Some(subdivision) }) => {
            country.push_str("-");
            country.push_str(&subdivision);
            country
        },
        None => "AU".into(),
    };
    Ok(warp::reply::json(&country))
}

async fn get_region_by_id(db: Database, id: String) -> Result<impl Reply, Rejection> {
    let conn = db.get().await.map_err(fail)?;

    #[derive(serde::Serialize)]
    struct Region {
        id: String,
        name: String,
        the: bool,
        geometry: String,
    }

    let stmt = "SELECT id, name, the, geometry FROM region WHERE id = $1";

    let row = conn
        .query_one(stmt, &[&id])
        .await
        .map_err(fail)?;

    let result = Region {
        id: row.get(0),
        name: row.get(1),
        the: row.get(2),
        geometry: row.get(3),
    };

    Ok(warp::reply::json(&result))
}

async fn get_stats_by_disease(db: Database, disease: String) -> Result<impl Reply, Rejection> {
    let conn = db.get().await.map_err(fail)?;

    #[derive(serde::Serialize)]
    struct Region {
        id: String,
        population: Option<i64>,
        cases: Option<i64>,
        deaths: Option<i64>,
        recoveries: Option<i64>,
    }

    let stmt = "
        SELECT
            region,
            (
                SELECT population
                FROM region_population
                WHERE region_population.region = disease_stats.region
                ORDER BY region_population.date DESC
                LIMIT 1
            ),
            MAX(cases),
            MAX(deaths),
            MAX(recoveries)
        FROM disease_stats
        WHERE disease = 'COVID-19'
        GROUP BY region
    ";

    let rows = conn.query(stmt, &[&disease]).await.map_err(fail)?;

    let result = rows
        .iter()
        .map(|row| Region {
            id: row.get(0),
            population: row.get(1),
            cases: row.get(2),
            deaths: row.get(3),
            recoveries: row.get(4),
        })
        .collect::<Vec<_>>();

    Ok(warp::reply::json(&result))
}

async fn get_stats_by_disease_in_region(db: Database, disease: String, region: String) -> Result<impl Reply, Rejection> {
    let conn = db.get().await.map_err(fail)?;

    #[derive(serde::Serialize)]
    struct Stat {
        date: String,
        population: Option<i64>,
        cases: Option<i64>,
        deaths: Option<i64>,
        recoveries: Option<i64>,
    }

    let stmt = "
        SELECT
            COALESCE(disease_stats.date, region_population.date),
            population,
            cases,
            deaths,
            recoveries
        FROM disease_stats
        FULL OUTER JOIN region_population
            ON region_population.date = disease_stats.date
        WHERE disease = $1
            AND disease_stats.region = $2
            AND region_population.region = $2
    ";

    let rows = conn.query(stmt, &[&disease, &region]).await.map_err(fail)?;

    let result = rows
        .iter()
        .map(|row| Stat {
            date: row.get(0),
            population: row.get(1),
            cases: row.get(2),
            deaths: row.get(3),
            recoveries: row.get(4),
        })
        .collect::<Vec<_>>();

    Ok(warp::reply::json(&result))
}
