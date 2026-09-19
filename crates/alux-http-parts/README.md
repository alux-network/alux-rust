# alux-http-parts

`alux-http-parts` states the parts a request and an answer are made of, for
[`alux-http`](https://docs.rs/alux-http) interpretations that need them.

A request and an answer are stated here as values: `DirectRequest` is what a caller sent, and
`DirectResponse` is what answering it produced, neither naming a transport nor a framework. A crate
stating requests, such as `alux-http-conformance`, needs no interpreter to state them with, and an
interpretation answering them re-exports these rather than restating them.

It also reads a body arriving as parts, for interpretations whose framework does not read one
itself.

Poem, axum, and Salvo read multipart bodies natively. The other interpreters receive only the bytes
and media type, so this crate provides the shared multipart reader instead of duplicating it in each
interpreter.

This crate names no framework. It turns the incoming bytes into `ChunksAlg` of `PartAlg`, the same
portable representation used by every interpretation that reads multipart bodies.
