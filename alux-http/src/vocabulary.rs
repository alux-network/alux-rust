//! Names every request method and output kind the specification states, once.
//!
//! Each list is read by whoever states something of it: the method or kind itself, and the
//! declaration an author writes it with. Keeping one list is what stops the two from drifting.

/// States every request method, as `declaration => Marker, "TOKEN"`.
macro_rules! with_http_methods {
    ($states:ident) => {
        $states! {
            get     => Get, "GET",
            post    => Post, "POST",
            put     => Put, "PUT",
            patch   => Patch, "PATCH",
            delete  => Delete, "DELETE",
            head    => Head, "HEAD",
            options => Options, "OPTIONS",
            trace   => Trace, "TRACE",
            connect => Connect, "CONNECT",
        }
    };
}

/// States every output kind, as `declaration => Kind, KindAlg, Selected, "meaning"`.
macro_rules! with_output_kinds {
    ($states:ident) => {
        $states! {
            json     => JsonOut, JsonOutAlg, Json, "JSON",
            file     => FileOut, FileOutAlg, File, "streamed-file",
            text     => TextOut, TextOutAlg, Text, "plain-text",
            html     => HtmlOut, HtmlOutAlg, Html, "HTML",
            bytes    => BytesOut, BytesOutAlg, Bytes, "raw-byte",
            empty    => EmptyOut, EmptyOutAlg, Empty, "empty",
            redirect => RedirectOut, RedirectOutAlg, Redirect, "redirect",
            stream   => StreamOut, StreamOutAlg, Stream, "streamed",
        }
    };
}
