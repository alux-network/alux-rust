# alux-http-parts

`alux-http-parts` reads a body arriving as parts, for [`alux-http`](https://docs.rs/alux-http)
interpretations whose framework does not read one itself.

Poem, axum, and Salvo read multipart bodies natively. The other interpreters receive only the bytes
and media type, so this crate provides the shared multipart reader instead of duplicating it in each
interpreter.

This crate names no framework. It turns the incoming bytes into `ChunksAlg` of `PartAlg`, the same
portable representation used by every interpretation that reads multipart bodies.
