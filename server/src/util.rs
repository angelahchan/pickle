//! Various Warp-related utilities.

use std::fmt::Debug;
use std::net::{IpAddr, AddrParseError, SocketAddr};
use std::str::FromStr;
use warp::filters::{header, addr};
use warp::{reject, Filter, Rejection, Reply};
use std::convert::Infallible;

struct Forwarded(IpAddr);

impl FromStr for Forwarded {
    type Err = AddrParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.find(',').map(|i| &s[..i]).unwrap_or(s).trim().parse()
    }
}

pub fn with_ip() -> impl Filter<Extract = (Option<IpAddr>,), Error = Infallible> + Clone {
    header::header("x-forwarded-for")
        .map(|Forwarded(ip): Forwarded| Some(ip))
        .or(addr::remote().map(|sock: Option<SocketAddr>| sock.map(|sock| sock.ip())))
        .unify()
}

pub fn fail<T: Debug + Sized + Send + Sync + 'static>(x: T) -> Rejection {
    #[derive(Debug)]
    struct Error<T>(T);
    impl<T: Debug + Sized + Send + Sync + 'static> reject::Reject for Error<T> {}
    reject::custom(Error(x))
}

pub fn not_found() -> impl Filter<Extract = impl Reply, Error = Infallible> + Clone {
    warp::any().map(|| warp::http::StatusCode::NOT_FOUND)
}
