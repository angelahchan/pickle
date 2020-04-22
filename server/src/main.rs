use database::Database;
use id::Id;
use std::env;
use std::error::Error;
use std::net::IpAddr;
use util::{with_ip, not_found};
use warp::{get, path, Filter};

mod database;
mod disease;
mod geolocation;
mod id;
mod region;
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
        (get() .and(db.with()) .and(path!("region"))                     .and_then(region::get_regions)                      ).or
        (get() .and(with_ip()) .and(path!("region" / "current"))         .and_then(region::get_current_region)               ).or
        (get() .and(db.with()) .and(path!("region" / "subregions" / Id)) .and_then(|x, y| region::get_subregions(x, Some(y)))).or
        (get() .and(db.with()) .and(path!("region" / "subregions"))      .and_then(|x| region::get_subregions(x, None))      ).or

        (get() .and(db.with()) .and(path!("disease"))                           .and_then(disease::get_diseases)               ).or
        (get() .and(db.with()) .and(path!("disease" / Id))                      .and_then(disease::get_disease_by_id)          ).or
        (get() .and(db.with()) .and(path!("disease" / Id / "in" / Id))          .and_then(disease::get_disease_by_id_in_region)).or
        (get() .and(db.with()) .and(path!("disease" / Id / "in" / Id / "news")) .and_then(disease::get_news)                   ).or

        (not_found())
    );

    // Start the server.

    let routes = data.or(static_files).with(warp::compression::gzip());

    Ok(warp::serve(routes).run((ip, port)).await)
}
