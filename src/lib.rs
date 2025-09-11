#![deny(unsafe_code)]
#![doc = include_str!("../README.md")]

use std::str::FromStr;

use serde::Deserialize;

pub mod body;
pub mod error;
pub mod header;
use error::Error;

pub type Result<T> = std::result::Result<T, crate::error::Error>;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Ofx {
    pub header: header::Header,
    #[serde(rename = "OFX")]
    pub body: body::Body,
}

impl FromStr for Ofx {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        // Per 1.6 spec, 1.2.1:
        // A blank line also separates the Open Financial Exchange headers and the request.
        // (See Chapter 2, “Structure” for more information about the Open Financial Exchange
        // headers.) A brief note here that a “blank line” means a carriage return and a
        // linefeed pair – CRLF.
        //
        // So in theory we could `split_once` and be done, but it seems that some banks provide
        // OFX files without this blank line, so we will be more flexible by just starting at the
        // `<OFX>` tag.
        let start = s
            .find("<OFX>")
            .ok_or(Error::ParseError("no `<OFX>` found".into()))?;

        let (raw_header, raw_body) = (&s[..start], &s[start..]);

        let header = raw_header.parse()?;
        let body = raw_body.parse()?;
        Ok(Self { header, body })
    }
}
