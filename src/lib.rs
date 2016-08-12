/*!

This crate provides wrappers and convenience functions to make Hyper and
Serde work hand in hand.

Currently, only two types are supported: `hyper::header::Headers`
and `hyper::method::Method`.

# How do I use a data type with a `Headers` member with Serde?

Use the serde attributes `deserialize_with` and `serialize_with`.

```
struct MyStruct {
    #[serde(deserialize_with = "hyper_serde::deserialize",
            serialize_with = "hyper_serde::serialize")]
    headers: Headers, 
}
```

# How do I encode a `Headers` value with `serde_json::to_string`?

Use the `Ser` wrapper.

```
serde_json::to_string(&Ser::new(&headers))
```

# How do I decode a `Method` value with `serde_json::parse`?

Use the `De` wrapper.

```
serde_json::parse::<De<Method>>("\"PUT\"").map(De::into_inner)
```

*/

#[deny(missing_docs)]
#[deny(unsafe_code)]

extern crate hyper;
extern crate serde;

use hyper::header::Headers;
use hyper::method::Method;
use serde::{Deserialize, Deserializer, Error, Serialize, Serializer};
use serde::de::{MapVisitor, Visitor};

#[cfg(test)] extern crate serde_test;
#[cfg(test)] use serde_test::{Token, assert_de_tokens, assert_ser_tokens};

/// Deserialises a `T` value with a given deserializer.
///
/// This is useful to deserialize Hyper types used in structure fields or
/// tuple members with `#[serde(deserialize_with = "hyper_serde::deserialize")]`.
pub fn deserialize<T, D>(deserializer: &mut D) -> Result<T, D::Error>
    where D: Deserializer, De<T>: Deserialize
{
    De::deserialize(deserializer).map(De::into_inner)
}

/// A wrapper to deserialize Hyper types.
///A
/// This is useful with functions such as `serde_json::from_str`.
///
/// Values of this type can only be obtained through
/// the `serde::Deserialize` trait.
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct De<T>(T);

impl<T> De<T> where De<T>: Deserialize {
    /// Consumes this wrapper, returning the deserialized value.
    #[inline(always)]
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl Deserialize for De<Headers> {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        struct HeadersVisitor;

        impl Visitor for HeadersVisitor {
            type Value = De<Headers>;

            fn visit_unit<E>(&mut self) -> Result<Self::Value, E>
                where E: Error
            {
                Ok(De(Headers::new()))
            }

            fn visit_map<V>(&mut self, mut visitor: V)
                            -> Result<Self::Value, V::Error>
                where V: MapVisitor
            {
                let mut headers = Headers::new();
                while let Some((key, value)) = try!(visitor.visit::<String, _>()) {
                    headers.set_raw(key, value);
                }
                try!(visitor.end());
                Ok(De(headers))
            }
        }

        deserializer.deserialize_map(HeadersVisitor)
    }
}

impl Deserialize for De<Method> {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        try!(String::deserialize(deserializer))
            .parse::<Method>().map(De)
            .map_err(|err| Error::invalid_value(&err.to_string()))
    }
}

/// Serialises `value` with a given serializer.
///
/// This is useful to serialize Hyper types used in structure fields or
/// tuple members with `#[serde(serialize_with = "hyper_serde::serialize")]`.
pub fn serialize<T, S>(value: &T, serializer: &mut S) -> Result<(), S::Error>
    where S: Serializer, for<'a> Ser<'a, T>: Serialize
{
    Ser::new(value).serialize(serializer)
}

/// A wrapper to serialize Hyper types.
///
/// This is useful with functions such as `serde_json::to_string`.
/// 
/// Values of this type can only be passed to the `serde::Serialize` trait.
#[derive(Debug)]
pub struct Ser<'a, T: 'a>(&'a T);

impl<'a, T> Ser<'a, T> where Ser<'a, T>: serde::Serialize {
    #[inline(always)]
    pub fn new(value: &'a T) -> Self {
        Ser(value)
    }
}

impl<'a> Serialize for Ser<'a, Headers> {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer
    {
        let mut map_state = try!(serializer.serialize_map(Some(self.0.len())));
        for header in self.0.iter() {
            let name = header.name();
            let value = self.0.get_raw(name).unwrap();
            try!(serializer.serialize_map_key(&mut map_state, name));
            try!(serializer.serialize_map_value(&mut map_state, value));
        }
        serializer.serialize_map_end(map_state)
    }
}

impl<'a> Serialize for Ser<'a, Method> {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer
    {
        Serialize::serialize(self.0.as_ref(), serializer)
    }
}

#[test]
fn test_headers_empty() {
    let headers = Headers::new();

    let tokens = &[
        Token::MapStart(Some(0)),
        Token::MapEnd,
    ];

    assert_ser_tokens(&Ser::new(&headers), tokens);
    assert_de_tokens(&De(headers), tokens);
}

#[test]
fn test_headers_not_empty() {
    use hyper::header::ContentLength;

    let mut headers = Headers::new();
    headers.set(ContentLength(15));

    // In Hyper 0.9, Headers is internally a HashMap and thus testing this
    // with multiple headers is non-deterministic.

    let tokens = &[
        Token::MapStart(Some(1)),
            Token::MapSep,
                Token::Str("Host"),
                Token::SeqStart(Some(1)),
                    Token::SeqSep,
                        Token::SeqStart(Some(8)),
                            Token::SeqSep, Token::U8(98),  // 'b'
                            Token::SeqSep, Token::U8(97),  // 'a'
                            Token::SeqSep, Token::U8(103), // 'g'
                            Token::SeqSep, Token::U8(117), // 'u'
                            Token::SeqSep, Token::U8(101), // 'e'
                            Token::SeqSep, Token::U8(116), // 't'
                            Token::SeqSep, Token::U8(116), // 't'
                            Token::SeqSep, Token::U8(101), // 'e'
                    Token::SeqEnd,
                Token::SeqEnd,
        Token::MapEnd,
    ];

    assert_ser_tokens(&Ser::new(&headers), tokens);
    assert_de_tokens(&De(headers), tokens);
}

#[test]
fn test_method() {
    let method = Method::Put;
    let tokens = &[Token::Str("PUT")];

    assert_ser_tokens(&Ser::new(&method), tokens);
    assert_de_tokens(&De(method), tokens);
}
