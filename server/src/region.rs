use crate::database::Database;
use crate::geolocation;
use crate::id::Id;
use crate::util::fail;
use serde::Serialize;
use std::net::IpAddr;
use warp::{Reply, Rejection};

pub async fn get_regions(db: Database) -> Result<impl Reply, Rejection> {
    let conn = db.get().await.map_err(fail)?;

    #[derive(Serialize)]
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

pub async fn get_current_region(ip: Option<IpAddr>) -> Result<impl Reply, Rejection> {
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

pub async fn get_subregions(db: Database, country: Option<Id>) -> Result<impl Reply, Rejection> {
    let conn = db.get().await.map_err(fail)?;

    #[derive(Serialize)]
    struct Region {
        id: String,
        name: String,
        geometry: Option<String>,
    }

    let result: Vec<_> =
        match country {
            Some(id) => conn.query("
                SELECT id, name, geometry
                FROM region
                WHERE starts_with(id, $1 || '-')
            ", &[&id.as_str()]).await,
            None => conn.query("
                SELECT id, name, geometry
                FROM region
                WHERE id NOT LIKE '%-%'
            ", &[]).await,
        }
        .map_err(fail)?
        .iter()
        .map(|row| Region {
            id: row.get(0),
            name: row.get(1),
            geometry: row.get(2),
        })
        .collect();

    Ok(warp::reply::json(&result))
}
