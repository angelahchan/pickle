use chrono::NaiveDate;
use crate::database::Database;
use crate::id::Id;
use serde::Serialize;
use crate::util::fail;
use warp::{Reply, Rejection};

pub async fn get_diseases(db: Database) -> Result<impl Reply, Rejection> {
    let conn = db.get().await.map_err(fail)?;

    #[derive(Serialize)]
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

pub async fn get_disease_by_id(db: Database, id: Id) -> Result<impl Reply, Rejection> {
    let conn = db.get().await.map_err(fail)?;

    #[derive(Serialize)]
    struct Disease {
        id: String,
        name: String,
        long_name: String,
        description: String,
        reinfectable: bool,
        popularity: f32,
        stats: Vec<Stat>,
    }

    #[derive(Serialize)]
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
        ", &[&id.as_str()])
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
        ", &[&id.as_str()])
        .await
        .map_err(fail)?;

    let result = Disease {
        id: id.into(),
        name: row.get(0),
        long_name: row.get(1),
        description: row.get(2),
        reinfectable: row.get(3),
        popularity: row.get(4),
        stats,
    };

    Ok(warp::reply::json(&result))
}

pub async fn get_disease_by_id_in_region(db: Database, id: Id, region: Id) -> Result<impl Reply, Rejection> {
    let conn = db.get().await.map_err(fail)?;

    #[derive(Serialize)]
    struct Disease {
        id: String,
        links: Vec<Link>,
        stats: Vec<Stat>,
        population: Vec<Population>,
    }

    #[derive(Serialize)]
    struct Link {
        uri: String,
        description: String,
    }

    #[derive(Serialize)]
    struct Stat {
        date: NaiveDate,
        cases: Option<i64>,
        deaths: Option<i64>,
        recoveries: Option<i64>,
    }

    #[derive(Serialize)]
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
        ", &[&id.as_str(), &region.as_str()])
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
        ", &[&id.as_str(), &region.as_str()])
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
            ", &[&region.as_str(), &min, &max])
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

    let result = Disease { id: id.into(), links, stats, population };

    Ok(warp::reply::json(&result))
}
