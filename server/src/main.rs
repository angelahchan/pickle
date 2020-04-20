use chrono::NaiveDate;
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
        (get() .and(db.with()) .and(path!("region"))             .and_then(get_regions)       ).or
        (get() .and(with_ip()) .and(path!("region" / "current")) .and_then(get_current_region)).or
        (get() .and(db.with()) .and(path!("region" / Id))        .and_then(get_region_by_id)  ).or

        (get() .and(db.with()) .and(path!("disease"))                  .and_then(get_diseases)               ).or
        (get() .and(db.with()) .and(path!("disease" / Id))             .and_then(get_disease_by_id)          ).or
        (get() .and(db.with()) .and(path!("disease" / Id / "in" / Id)) .and_then(get_disease_by_id_in_region)).or

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
    }

    let stmt = "SELECT id, name FROM region";

    let rows = conn.query(stmt, &[]).await.map_err(fail)?;

    let result = rows
        .iter()
        .map(|row| Region {
            id: row.get(0),
            name: row.get(1),
        })
        .collect::<Vec<_>>();

    Ok(warp::reply::json(&result))
}

async fn get_current_region(ip: Option<IpAddr>) -> Result<impl Reply, Rejection> {
    const DEFAULT_REGION: &'static str = "AU";

    let country = match ip.and_then(|ip| geolocation::guess_by_ip(ip)) {
        Some(geolocation::Guess { country, subdivision: None }) => country,
        Some(geolocation::Guess { mut country, subdivision: Some(subdivision) }) => {
            country.push_str("-");
            country.push_str(&subdivision);
            country
        },
        None => DEFAULT_REGION.into(),
    };

    Ok(warp::reply::json(&country))
}

async fn get_region_by_id(db: Database, Id(id): Id) -> Result<impl Reply, Rejection> {
    let conn = db.get().await.map_err(fail)?;

    #[derive(serde::Serialize)]
    struct Region {
        id: String,
        name: String,
        geometry: Option<String>,
    }

    let stmt = "SELECT id, name, geometry FROM region WHERE id = $1";

    let row = conn
        .query_one(stmt, &[&id])
        .await
        .map_err(fail)?;

    let result = Region {
        id: row.get(0),
        name: row.get(1),
        geometry: row.get(2),
    };

    Ok(warp::reply::json(&result))
}

async fn get_diseases(db: Database) -> Result<impl Reply, Rejection> {
    let conn = db.get().await.map_err(fail)?;

    #[derive(serde::Serialize)]
    struct Disease {
        id: String,
        name: String,
        long_name: String,
        popularity: f32,
    }

    let stmt = "SELECT id, name, long_name, popularity FROM disease";

    let rows = conn.query(stmt, &[]).await.map_err(fail)?;

    let result = rows
        .iter()
        .map(|row| Disease {
            id: row.get(0),
            name: row.get(1),
            long_name: row.get(2),
            popularity: row.get(3),
        })
        .collect::<Vec<_>>();

    Ok(warp::reply::json(&result))
}

async fn get_disease_by_id(db: Database, Id(id): Id) -> Result<impl Reply, Rejection> {
    let conn = db.get().await.map_err(fail)?;

    #[derive(serde::Serialize)]
    struct Disease {
        id: String,
        name: String,
        long_name: String,
        description: String,
        reinfectable: bool,
        popularity: f32,
        stats: Vec<Stat>,
    }

    #[derive(serde::Serialize)]
    struct Stat {
        region: String,
        cases: Option<i64>,
        deaths: Option<i64>,
        recoveries: Option<i64>,
        population: Option<i64>,
    }

    let stats = conn
        .query("
            SELECT
                region,
                MAX(cases),
                MAX(deaths),
                MAX(recoveries),
                (
                    SELECT population
                    FROM region_population
                    WHERE
                        region_population.region = disease_stats.region AND
                        population IS NOT NULL
                    ORDER BY region_population.date DESC
                    LIMIT 1
                )
            FROM disease_stats
            WHERE disease = $1
            GROUP BY region
        ", &[&id])
        .await
        .map_err(fail)?
        .iter()
        .map(|row| Stat {
            region: row.get(0),
            cases: row.get(1),
            deaths: row.get(2),
            recoveries: row.get(3),
            population: row.get(4),
        })
        .collect();

    let row = conn
        .query_one("
            SELECT name, long_name, description, reinfectable, popularity
            FROM disease
            WHERE id = $1
        ", &[&id])
        .await
        .map_err(fail)?;

    let result = Disease {
        id,
        name: row.get(0),
        long_name: row.get(1),
        description: row.get(2),
        reinfectable: row.get(3),
        popularity: row.get(4),
        stats,
    };

    Ok(warp::reply::json(&result))
}

async fn get_disease_by_id_in_region(db: Database, Id(id): Id, Id(region): Id) -> Result<impl Reply, Rejection> {
    let conn = db.get().await.map_err(fail)?;

    #[derive(serde::Serialize)]
    struct Disease {
        id: String,
        links: Vec<Link>,
        stats: Vec<Stat>,
        population: Vec<Population>,
    }

    #[derive(serde::Serialize)]
    struct Link {
        uri: String,
        description: String,
    }

    #[derive(serde::Serialize)]
    struct Stat {
        date: NaiveDate,
        cases: Option<i64>,
        deaths: Option<i64>,
        recoveries: Option<i64>,
    }

    #[derive(serde::Serialize)]
    struct Population {
        date: NaiveDate,
        population: Option<i64>,
    }

    let links = conn
        .query("
            SELECT uri, description
            FROM disease_link
            WHERE
                disease = $1 AND
                (region IS NULL OR region = $2 OR starts_with($2, region || '-'))
        ", &[&id, &region])
        .await
        .map_err(fail)?
        .iter()
        .map(|row| Link {
            uri: row.get(0),
            description: row.get(1),
        })
        .collect();

    let stats: Vec<_> = conn
        .query("
            SELECT date, cases, deaths, recoveries
            FROM disease_stats
            WHERE
                disease = $1 AND
                region = $2
        ", &[&id, &region])
        .await
        .map_err(fail)?
        .iter()
        .map(|row| Stat {
            date: row.get(0),
            cases: row.get(1),
            deaths: row.get(2),
            recoveries: row.get(3),
        })
        .collect();

    let population = if let (Some(min), Some(max)) = (
        stats.iter().map(|x| x.date).min(),
        stats.iter().map(|x| x.date).max(),
    ) {
        conn
            .query("
                SELECT date, population
                FROM region_population
                WHERE
                    region = $1 AND
                    (
                        ($2 <= date AND date <= $3) OR
                        (date < $2 AND date >= ALL(SELECT date FROM region_population WHERE region = $1 AND date < $2)) OR
                        (date > $3 AND date <= ALL(SELECT date FROM region_population WHERE region = $1 AND date > $3))
                    )
            ", &[&region, &min, &max])
            .await
            .map_err(fail)?
            .iter()
            .map(|row| Population {
                date: row.get(0),
                population: row.get(1),
            })
            .collect()
    } else {
        vec![]
    };

    let result = Disease { id, links, stats, population };

    Ok(warp::reply::json(&result))
}

/// An identifier, converted into uppercase.
struct Id(pub String);

impl std::str::FromStr for Id {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_uppercase()))
    }
}
