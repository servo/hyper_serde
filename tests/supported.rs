extern crate cookie;
extern crate hyper;
extern crate hyper_serde;
extern crate serde;

use cookie::Cookie;
use hyper::header::{ContentType, Headers};
use hyper::http::RawStatus;
use hyper::method::Method;
use hyper_serde::{De, Ser, Serde};
use serde::{Deserialize, Serialize};

fn is_supported<T>()
    where De<T>: Deserialize,
          for<'a> Ser<'a, T>: Serialize,
          Serde<T>: Deserialize + Serialize
{
}

#[test]
fn supported() {
    is_supported::<Cookie>();
    is_supported::<ContentType>();
    is_supported::<Headers>();
    is_supported::<Method>();
    is_supported::<RawStatus>();
}
