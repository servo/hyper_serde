extern crate cookie;
extern crate hyper;
extern crate hyper_serde;
#[macro_use]
extern crate mime;
extern crate serde;
extern crate serde_test;
extern crate time;

use cookie::Cookie;
use hyper::header::{ContentType, Headers};
use hyper::http::RawStatus;
use hyper::method::Method;
use hyper_serde::{De, Ser, deserialize};
use serde::Deserialize;
use serde_test::{Deserializer, Token, assert_ser_tokens};
use std::fmt::Debug;
use time::Duration;

#[test]
fn test_content_type() {
    let content_type = ContentType(mime!(Application / Json));
    let tokens = &[Token::Str("application/json")];

    assert_ser_tokens(&Ser::new(&content_type), tokens);
    assert_de_tokens(&content_type, tokens);
}

#[test]
fn test_cookie() {
    let cookie = Cookie::build("Hello", "World!")
        .max_age(Duration::seconds(42))
        .domain("servo.org")
        .path("/")
        .secure(true)
        .http_only(false)
        .finish();

    let tokens = &[Token::Str("Hello=World!; Secure; Path=/; \
                               Domain=servo.org; Max-Age=42")];

    assert_ser_tokens(&Ser::new(&cookie), tokens);
    assert_de_tokens(&cookie, tokens);
}

#[test]
fn test_headers_empty() {
    let headers = Headers::new();

    let tokens = &[Token::MapStart(Some(0)), Token::MapEnd];

    assert_ser_tokens(&Ser::new(&headers), tokens);
    assert_de_tokens(&headers, tokens);
}

#[test]
fn test_headers_not_empty() {
    use hyper::header::Host;

    let mut headers = Headers::new();
    headers.set(Host {
        hostname: "baguette".to_owned(),
        port: None,
    });

    // In Hyper 0.9, Headers is internally a HashMap and thus testing this
    // with multiple headers is non-deterministic.

    let tokens = &[Token::MapStart(Some(1)),
                   Token::MapSep,
                   Token::Str("Host"),
                   Token::SeqStart(Some(1)),
                   Token::SeqSep,
                   Token::Bytes(b"baguette"),
                   Token::SeqEnd,
                   Token::MapEnd];

    assert_ser_tokens(&Ser::new(&headers), tokens);
    assert_de_tokens(&headers, tokens);

    let pretty = &[Token::MapStart(Some(1)),
                   Token::MapSep,
                   Token::Str("Host"),
                   Token::SeqStart(Some(1)),
                   Token::SeqSep,
                   Token::Str("baguette"),
                   Token::SeqEnd,
                   Token::MapEnd];

    assert_ser_tokens(&Ser::new_pretty(&headers), pretty);
    assert_de_tokens(&headers, pretty);
}

#[test]
fn test_method() {
    let method = Method::Put;
    let tokens = &[Token::Str("PUT")];

    assert_ser_tokens(&Ser::new(&method), tokens);
    assert_de_tokens(&method, tokens);
}

#[test]
fn test_raw_status() {
    use std::borrow::Cow;

    let raw_status = RawStatus(200, Cow::Borrowed("OK"));
    let tokens = &[Token::TupleStart(2),
                   Token::TupleSep,
                   Token::U16(200),
                   Token::TupleSep,
                   Token::Str("OK"),
                   Token::TupleEnd];

    assert_ser_tokens(&Ser::new(&raw_status), tokens);
    assert_de_tokens(&raw_status, tokens);
}

pub fn assert_de_tokens<T>(value: &T, tokens: &[Token<'static>])
    where T: Debug + PartialEq,
          De<T>: Deserialize,
{
    let mut deserializer = Deserializer::new(tokens.iter().cloned());
    let v = deserialize::<T, _>(&mut deserializer);
    assert_eq!(v.as_ref(), Ok(value));
    assert_eq!(deserializer.next_token(), None);
}
