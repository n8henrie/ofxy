# ofxy

master: [![master branch build status](https://github.com/n8henrie/ofxy/actions/workflows/build.yml/badge.svg?branch=master)](https://github.com/n8henrie/ofxy/actions/workflows/build.yml)

[![Crates.io](https://img.shields.io/crates/v/ofxy.svg)](https://crates.io/crates/ofxy)
[![Docs.rs](https://docs.rs/ofxy/badge.svg)](https://docs.rs/ofxy)

## Introduction

Ofxy is a Rust library for reading Open Financial Exchange (OFX) data.
It is still in early stages of development, and I am not a financial professional or a developer by training; temper your expectations accordingly.
Contributions and feedback / constructive criticism are welcome and appreciated.

My primary motivation is for reading the OFX files provided by financial institutions that I use.
Because Ofxy is now able to successfully read these files from all 3 different institutions that use, I thought it seemed reasonable to open source.
There is a **lot** more to the OFX spec than what I've provided here; contributions to make Ofxy more correct or broadly useful are appreciated.

## Features

- Support for parsing OFX 1.6 files

## Quickstart

```rust
use ofxy::{Ofx, body::TransactionType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string("tests/files/simple.ofx")?;
    let doc: ofxy::Ofx = content.parse()?;
    for tx in doc
        .body
        .credit_card
        .expect("credit card section not found")
        .transaction_response
        .statement
        .bank_transactions
        .expect("bank transactions not found")
        .transactions
    {
        let ofxy::body::Transaction {
            transaction_type,
            date_posted,
            amount,
            name,
            ..
        } = tx;
        match transaction_type {
            TransactionType::Debit => {
                println!("{date_posted}: spent {amount} at {}", name.unwrap());
            }
            TransactionType::Payment => {
                println!("{date_posted}: paid {amount} to {}", name.unwrap());
            }
            _ => (),
        }
    }
    Ok(())
}
```

### Development Setup

1. Clone the repo: `git clone https://github.com/n8henrie/ofxy && cd ofxy`
2. Install dependencies: `cargo build`

## Acknowledgements

- sgmlish (MIT) - <https://github.com/mernen/sgmlish>
  - does the heavy lifting of the SGML parsing for the 1.6 OFX spec

## Goals

- No unsafe code
- Idiomatic error handling
- Well tested code
- Readable code

## Wishlist / Maybe / Would Be Nice

- OFX 2+ support
- Decent performance
- Serialization

## OFX 1.6 Spec

- <https://financialdataexchange.org/common/Uploaded%20files/OFX%20files/OFX1.6.zip> ([archive.org](https://web.archive.org/web/20221210013837/https://financialdataexchange.org/common/Uploaded%20files/OFX%20files/OFX1.6.zip))
- md5sum: b19e9ab7cbbd83797d51781b32f04386 OFX1.6.zip

## Troubleshooting / FAQ

- Ofxy fails to parse my OFX file!
  - If you're comfortable redacting the private information and comfortable with GPL3 licensing, please submit the file in a GitHub Issue; I'll gladly try to sort out the failure and add it to the existing test suite
  - Note that _many_ financial institutions seem to provide OFX files that don't adhere to the spec, some of which currently fail. I'm not yet fully decided whether Ofxy should try to accomodate these by default, provide some kind of `Strict` vs `Relaxed` parsing, or reject the files as invalid. Feel free to submit opinions and justification.

## Attribution

Test files were copied (with gratitude!) from:

- <https://github.com/jseutter/ofxparse>
  - license: MIT
  - commit: 25aa80072be71da9f4ea0924ae1d1aaea2d5820c
- <https://github.com/mernen/sgmlish/blob/main/tests/deserialize.rs>
  - license: MIT
  - commit: 2f50fbf82340d8a82e582114f1bd259695ae30de
- <https://github.com/asgrim/ofxparser>
  - license: MIT
  - commit: ef7ca52b5c951a71c37c466925e526950a52b292
- <https://github.com/ofx-reader/ofx-reader>
  - license: MIT
  - commit: a9f1cba32baa9f1cb7a45faa14e3503e5cdf0c8f
- <https://github.com/csingley/ofxtools>
  - license: GPL3
  - commit: 6ad692e386e0cb82d5903682d89c18be00742c29
