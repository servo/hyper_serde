Serde support for Hyper types
=============================

This crate provides wrappers and convenience functions to support [Serde] for
the types defined in [cookie], [Hyper] and [mime].

[cookie]: https://github.com/alexcrichton/cookie-rs
[Hyper]: https://github.com/hyperium/hyper
[mime]: https://github.com/hyperium/mime.rs
[Serde]: https://github.com/serde-rs/serde

The supported types are:

* `cookie::Cookie`
* `hyper::header::ContentType`
* `hyper::header::Headers`
* `hyper::http::RawStatus`
* `hyper::method::Method`
* `mime::Mime`

For more details, see the crate documentation.
