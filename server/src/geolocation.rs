use maxminddb::Reader;
use maxminddb::geoip2::City;
use lazy_static::lazy_static;
use std::net::IpAddr;

lazy_static! {
    static ref GEOLITE2: Reader<Vec<u8>> = Reader::open_readfile("geolite2-city.mmdb").unwrap();
}

pub struct Guess {
    /// An ISO 3166-1 alpha-2 code.
    pub country: String,
    /// The region portion of an ISO 3166-2 code.
    pub subdivision: Option<String>,
}

pub fn guess_by_ip(ip: IpAddr) -> Option<Guess> {
    let record: City = GEOLITE2.lookup(ip).ok()?;

    let country = record.country?.iso_code?;

    let subdivision = record
        .subdivisions
        .and_then(|xs| xs.into_iter().next())
        .and_then(|x| x.iso_code);

    Some(Guess { country, subdivision })
}
