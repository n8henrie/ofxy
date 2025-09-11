use std::{collections::HashMap, str::FromStr};

use serde::Deserialize;

use crate::{Result, error::Error};

// Per the 1.6 spec, 2.2:
// All OFX headers are required. NONE should be returned if client or server does not make use of
// an individual element, e.g., COMPRESSION:NONE, OLDFILEUID:NONE
#[derive(Debug, Deserialize, PartialEq)]
pub struct Header {
    pub ofxheader: u32,
    pub data: Data,
    pub version: Version,
    pub security: Security,
    pub encoding: Encoding,
    pub charset: String,
    pub compression: String,
    pub oldfileuid: String,
    pub newfileuid: String,
}

impl FromStr for Header {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        // Prolog is only valid for XML-based OFX files (2.0 and later), but leaving this in case
        // we can support it in the future.
        let prolog_flag = "<?OFX ";
        let headers_map: HashMap<_, _> = if let Some(start) = s.find(prolog_flag) {
            let Some(end_delta) = s[start..].find("?>") else {
                return Err(Error::ParseError("invalid OFX header".into()));
            };
            let end = start + end_delta;
            let prolog_contents = &s[start + prolog_flag.len()..end];
            prolog_contents
                .split(r#"" "#)
                .map(|s| {
                    let s = s.trim();
                    let Some((key, value)) = s.split_once('=') else {
                        return Err(Error::ParseError(format!("invalid OFX header at {s}")));
                    };
                    Ok((key.to_string(), value.trim_matches('"').to_string()))
                })
                .collect::<Result<HashMap<_, _>>>()?
        } else {
            s.lines()
                .filter(|line| !line.trim().is_empty())
                .map(|line| {
                    let line = line.trim();
                    let Some((key, value)) = line.split_once(':') else {
                        return Err(Error::ParseError(format!("invalid OFX header at {line}")));
                    };
                    Ok((key.to_string(), value.to_string()))
                })
                .collect::<Result<HashMap<_, _>>>()?
        };

        Ok(Self {
            ofxheader: headers_map
                .get("OFXHEADER")
                .ok_or_else(|| Error::ParseError("headers missing 'OFXHEADER'".into()))?
                .parse()?,
            data: headers_map
                .get("DATA")
                .map(|s| s.parse())
                .transpose()?
                .unwrap_or_default(),
            version: headers_map
                .get("VERSION")
                .ok_or_else(|| Error::ParseError("headers missing 'VERSION'".into()))?
                .parse()?,
            security: headers_map
                .get("SECURITY")
                .map(|s| s.parse())
                .transpose()?
                .unwrap_or_default(),
            encoding: headers_map
                .get("ENCODING")
                .map(|s| s.parse())
                .transpose()?
                .unwrap_or_default(),
            charset: headers_map
                .get("CHARSET")
                .cloned()
                .ok_or_else(|| Error::ParseError("headers missing 'CHARSET'".into()))?,
            compression: headers_map.get("COMPRESSION").cloned().unwrap_or_default(),
            oldfileuid: headers_map
                .get("OLDFILEUID")
                .cloned()
                .ok_or_else(|| Error::ParseError("headers missing 'OLDFILEUID'".into()))?,
            newfileuid: headers_map
                .get("NEWFILEUID")
                .cloned()
                .ok_or_else(|| Error::ParseError("headers missing 'NEWFILEUID'".into()))?,
        })
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[non_exhaustive]
pub enum Version {
    V102,
    V103,
    V151,
    V160,
}

impl FromStr for Version {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "102" => Ok(Version::V102),
            "103" => Ok(Version::V103),
            "151" => Ok(Version::V151),
            "160" => Ok(Version::V160),
            // Will need to fix this if OFX eventually gets to V10
            _ if s.starts_with('1') => Err(Error::ParseError(format!(
                "Ofxy was not aware of OFX version {s}. \
                 Please submit an issue so we can take a closer look!"
            ))),
            _ => Err(Error::ParseError(format!(
                "Ofxy does not yet support OFX version: {s}"
            ))),
        }
    }
}

#[derive(Debug, Default, Deserialize, PartialEq)]
#[non_exhaustive]
pub enum Encoding {
    Unicode,
    #[default]
    UsAscii,
}

impl FromStr for Encoding {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "UNICODE" => Ok(Encoding::Unicode),
            "USASCII" => Ok(Encoding::UsAscii),
            _ => Err(Error::ParseError("invalid encoding type".into())),
        }
    }
}

#[derive(Debug, Default, Deserialize, PartialEq)]
#[non_exhaustive]
pub enum Data {
    #[default]
    Ofxsgml,
}

impl FromStr for Data {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "OFXSGML" => Ok(Data::Ofxsgml),
            _ => Err(Error::ParseError("invalid data type".into())),
        }
    }
}

#[derive(Debug, Default, Deserialize, PartialEq)]
pub enum Security {
    #[default]
    None,
    Type1,
}

impl FromStr for Security {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "NONE" => Ok(Security::None),
            "TYPE1" => Ok(Security::Type1),
            _ => Err(Error::ParseError("invalid security type".into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_header() {
        let input = "OFXHEADER:100
DATA:OFXSGML
VERSION:102
SECURITY:NONE
ENCODING:USASCII
CHARSET:1252
COMPRESSION:NONE
OLDFILEUID:NONE
NEWFILEUID:NONE

";
        let header: Header = input.parse().unwrap();
        assert_eq!(header.ofxheader, 100);
        assert_eq!(header.version, Version::V102);
        assert_eq!(header.security, Security::None);
        assert_eq!(header.oldfileuid, "NONE");
        assert_eq!(header.newfileuid, "NONE");
        assert_eq!(header.data, Data::Ofxsgml);
        assert_eq!(header.encoding, Encoding::UsAscii);
        assert_eq!(header.charset, "1252");
        assert_eq!(header.compression, "NONE");
    }
}
