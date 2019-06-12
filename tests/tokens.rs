extern crate cookie;
extern crate headers;
extern crate http;
extern crate hyper;
extern crate hyper_serde;
extern crate mime;
extern crate serde;
extern crate serde_test;
extern crate time;

use cookie::Cookie;
use http::header::{self, HeaderMap, HeaderValue};
use headers::ContentType;
use http::StatusCode;
use hyper::Method;
use hyper_serde::{De, Ser, deserialize};
use serde::Deserialize;
use serde_test::{Deserializer, Token, assert_ser_tokens};
use std::fmt::Debug;
use time::Duration;

#[test]
fn test_content_type() {
    let content_type = ContentType::from("Application/Json".parse::<mime::Mime>().unwrap());
    let tokens = &[Token::Str("application/json")];

    assert_ser_tokens(&Ser::new(&content_type), tokens);
    assert_de_tokens(&content_type, tokens);
}

#[test]
fn test_cookie() {
    // Unfortunately we have to do the to_string().parse() dance here to avoid the object being a
    // string with a bunch of indices in it which apparently is different from the exact same
    // cookie but parsed as a bunch of strings.
    let cookie: Cookie = Cookie::build("Hello", "World!")
        .max_age(Duration::seconds(42))
        .domain("servo.org")
        .path("/")
        .secure(true)
        .http_only(false)
        .finish().to_string().parse().unwrap();

    let tokens = &[Token::Str("Hello=World!; Secure; Path=/; Domain=servo.org; Max-Age=42")];

    assert_ser_tokens(&Ser::new(&cookie), tokens);
    assert_de_tokens(&cookie, tokens);
}

#[test]
fn test_headers_empty() {
    let headers = HeaderMap::new();

    let tokens = &[Token::Map { len: Some(0) }, Token::MapEnd];

    assert_ser_tokens(&Ser::new(&headers), tokens);
    assert_de_tokens(&headers, tokens);
}

#[test]
fn test_headers_not_empty() {
    let mut headers = HeaderMap::new();
    headers.insert(header::HOST, HeaderValue::from_static("baguette"));

    // In http, HeaderMap is internally a HashMap and thus testing this
    // with multiple headers is non-deterministic.

    let tokens = &[Token::Map{ len: Some(1) },
                   Token::Str("host"),
                   Token::Seq{ len: Some(1) },
                   Token::Bytes(b"baguette"),
                   Token::SeqEnd,
                   Token::MapEnd];

    assert_ser_tokens(&Ser::new(&headers), tokens);
    assert_de_tokens(&headers, tokens);

    let pretty = &[Token::Map{ len: Some(1) },
                   Token::Str("host"),
                   Token::Seq{ len: Some(1) },
                   Token::Str("baguette"),
                   Token::SeqEnd,
                   Token::MapEnd];

    assert_ser_tokens(&Ser::new_pretty(&headers), pretty);
    assert_de_tokens(&headers, pretty);
}

#[test]
fn test_method() {
    let method = Method::PUT;
    let tokens = &[Token::Str("PUT")];

    assert_ser_tokens(&Ser::new(&method), tokens);
    assert_de_tokens(&method, tokens);
}

#[test]
fn test_raw_status() {
    let raw_status = StatusCode::from_u16(200).unwrap();
    let tokens = &[Token::U16(200)];

    assert_ser_tokens(&Ser::new(&raw_status), tokens);
    assert_de_tokens(&raw_status, tokens);
}

#[test]
fn test_tm() {
    use time::strptime;

    let time = strptime("2017-02-22T12:03:31Z", "%Y-%m-%dT%H:%M:%SZ").unwrap();
    let tokens = &[Token::Str("2017-02-22T12:03:31Z")];

    assert_ser_tokens(&Ser::new(&time), tokens);
    assert_de_tokens(&time, tokens);
}

pub fn assert_de_tokens<T>(value: &T, tokens: &[Token])
    where T: Debug + PartialEq,
          for<'de> De<T>: Deserialize<'de>,
{
    let mut deserializer = Deserializer::new(&tokens);
    let v = deserialize::<T, _>(&mut deserializer).unwrap();
    assert_eq!(&v, value);
    assert_eq!(deserializer.next_token_opt(), None);
}
