//! This crate provides wrappers and convenience functions to make Hyper and
//! Serde work hand in hand.
//!
//! The supported types are:
//!
//! `cookie::Cookie`
//! `hyper::header::ContentType`
//! `hyper::header::Headers`
//! `hyper::http::RawStatus`
//! `hyper::method::Method`
//!
//! # How do I use a data type with a `Headers` member with Serde?
//!
//! Use the serde attributes `deserialize_with` and `serialize_with`.
//!
//! ```
//! struct MyStruct {
//! #[serde(deserialize_with = "hyper_serde::deserialize",
//! serialize_with = "hyper_serde::serialize")]
//! headers: Headers,
//! }
//! ```
//!
//! # How do I encode a `Headers` value with `serde_json::to_string`?
//!
//! Use the `Ser` wrapper.
//!
//! ```
//! serde_json::to_string(&Ser::new(&headers))
//! ```
//!
//! # How do I decode a `Method` value with `serde_json::parse`?
//!
//! Use the `De` wrapper.
//!
//! ```
//! serde_json::parse::<De<Method>>("\"PUT\"").map(De::into_inner)
//! ```
//!
//! # How do I send `Cookie` values as part of an IPC channel?
//!
//! Use the `Serde` wrapper. It implements `Deref` and `DerefMut` for
//! convenience.
//!
//! ```
//! ipc::channel::<Serde<Cookie>>()
//! ```
//!
//!

#![deny(missing_docs)]
#![deny(unsafe_code)]

extern crate cookie;
extern crate hyper;
extern crate mime;
extern crate serde;

use cookie::Cookie;
use hyper::header::{ContentType, Headers};
use hyper::http::RawStatus;
use hyper::method::Method;
use mime::Mime;
use serde::{Deserialize, Deserializer, Error, Serialize, Serializer};
use serde::de::{MapVisitor, Visitor};
use std::cmp::PartialEq;
use std::fmt;
use std::ops::{Deref, DerefMut};

/// Deserialises a `T` value with a given deserializer.
///
/// This is useful to deserialize Hyper types used in structure fields or
/// tuple members with `#[serde(deserialize_with = "hyper_serde::deserialize")]`.
pub fn deserialize<T, D>(deserializer: &mut D) -> Result<T, D::Error>
    where D: Deserializer,
          De<T>: Deserialize,
{
    De::deserialize(deserializer).map(De::into_inner)
}

/// A wrapper to deserialize Hyper types.
///
/// This is useful with functions such as `serde_json::from_str`.
///
/// Values of this type can only be obtained through
/// the `serde::Deserialize` trait.
#[derive(Debug)]
pub struct De<T>(T);

impl<T> De<T>
    where De<T>: Deserialize,
{
    /// Consumes this wrapper, returning the deserialized value.
    #[inline(always)]
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl Deserialize for De<ContentType> {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: Deserializer,
    {
        Mime::deserialize(deserializer).map(ContentType).map(De)
    }
}

impl Deserialize for De<Cookie> {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: Deserializer,
    {
        struct CookieVisitor;

        impl Visitor for CookieVisitor {
            type Value = De<Cookie>;

            fn visit_str<E>(&mut self, v: &str) -> Result<Self::Value, E>
                where E: Error,
            {
                Cookie::parse(v)
                    .map(De)
                    .map_err(|e| E::custom(format!("{:?}", e)))
            }
        }

        deserializer.deserialize_string(CookieVisitor)
    }
}

impl Deserialize for De<Headers> {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: Deserializer,
    {
        struct HeadersVisitor;

        impl Visitor for HeadersVisitor {
            type Value = De<Headers>;

            fn visit_unit<E>(&mut self) -> Result<Self::Value, E>
                where E: Error,
            {
                Ok(De(Headers::new()))
            }

            fn visit_map<V>(&mut self,
                            mut visitor: V)
                            -> Result<Self::Value, V::Error>
                where V: MapVisitor,
            {
                let mut headers = Headers::new();
                while let Some((key, value)) = visitor.visit::<String, _>()? {
                    headers.set_raw(key, value);
                }
                visitor.end()?;
                Ok(De(headers))
            }
        }

        deserializer.deserialize_map(HeadersVisitor)
    }
}

impl Deserialize for De<Method> {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: Deserializer,
    {
        struct MethodVisitor;

        impl Visitor for MethodVisitor {
            type Value = De<Method>;

            fn visit_str<E>(&mut self, v: &str) -> Result<Self::Value, E>
                where E: Error,
            {
                v.parse::<Method>()
                    .map(De)
                    .map_err(|err| E::invalid_value(&err.to_string()))
            }
        }

        deserializer.deserialize_string(MethodVisitor)
    }
}

impl Deserialize for De<RawStatus> {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: Deserializer,
    {
        let (code, reason) = Deserialize::deserialize(deserializer)?;
        Ok(De(RawStatus(code, reason)))
    }
}

/// Serialises `value` with a given serializer.
///
/// This is useful to serialize Hyper types used in structure fields or
/// tuple members with `#[serde(serialize_with = "hyper_serde::serialize")]`.
pub fn serialize<T, S>(value: &T, serializer: &mut S) -> Result<(), S::Error>
    where S: Serializer,
          for<'a> Ser<'a, T>: Serialize,
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

impl<'a, T> Ser<'a, T>
    where Ser<'a, T>: serde::Serialize,
{
    /// Returns a new `Ser` wrapper.
    #[inline(always)]
    pub fn new(value: &'a T) -> Self {
        Ser(value)
    }
}

impl<'a> Serialize for Ser<'a, ContentType> {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer,
    {
        (self.0).0.serialize(serializer)
    }
}

impl<'a> Serialize for Ser<'a, Cookie> {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'a> Serialize for Ser<'a, Headers> {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer,
    {
        let mut map_state = serializer.serialize_map(Some(self.0.len()))?;
        for header in self.0.iter() {
            let name = header.name();
            let value = self.0.get_raw(name).unwrap();
            serializer.serialize_map_key(&mut map_state, name)?;
            serializer.serialize_map_value(&mut map_state, value)?;
        }
        serializer.serialize_map_end(map_state)
    }
}

impl<'a> Serialize for Ser<'a, Method> {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer,
    {
        Serialize::serialize(self.0.as_ref(), serializer)
    }
}

impl<'a> Serialize for Ser<'a, RawStatus> {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer,
    {
        ((self.0).0, &(self.0).1).serialize(serializer)
    }
}

/// A convenience wrapper to be used as a type parameter, for example when
/// a `Vec<T>` need to be passed to serde.
#[derive(Clone, PartialEq)]
pub struct Serde<T>(pub T)
    where De<T>: Deserialize,
          for<'a> Ser<'a, T>: Serialize;

impl<T> Serde<T>
    where De<T>: Deserialize,
          for<'a> Ser<'a, T>: Serialize,
{
    /// Consumes this wrapper, returning the inner value.
    #[inline(always)]
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> fmt::Debug for Serde<T>
    where T: fmt::Debug,
          De<T>: Deserialize,
          for<'a> Ser<'a, T>: Serialize,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.0.fmt(formatter)
    }
}

impl<T> Deref for Serde<T>
    where De<T>: Deserialize,
          for<'a> Ser<'a, T>: Serialize,
{
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for Serde<T>
    where De<T>: Deserialize,
          for<'a> Ser<'a, T>: Serialize,
{
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: PartialEq> PartialEq<T> for Serde<T>
    where De<T>: Deserialize,
          for<'a> Ser<'a, T>: Serialize,
{
    fn eq(&self, other: &T) -> bool {
        self.0 == *other
    }
}

impl<T> Deserialize for Serde<T>
    where De<T>: Deserialize,
          for<'a> Ser<'a, T>: Serialize,
{
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: Deserializer,
    {
        De::deserialize(deserializer).map(De::into_inner).map(Serde)
    }
}

impl<T> Serialize for Serde<T>
    where De<T>: Deserialize,
          for<'a> Ser<'a, T>: Serialize,
{
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer,
    {
        Ser(&self.0).serialize(serializer)
    }
}
