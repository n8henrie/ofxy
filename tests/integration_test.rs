use std::fs::read_dir;
use std::{ffi::OsStr, path::Path, result};
use std::{path::PathBuf, str::FromStr};

use chrono::{TimeZone, Utc};
use rust_decimal::Decimal;

use ofxy::{
    Ofx,
    body::{Currency, TransactionType},
    header::{Encoding, Version},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn test_ofx_sgmlish() -> result::Result<(), Box<dyn std::error::Error>> {
    let input = std::fs::read_to_string("tests/files/sgmlish/example.ofx")
        .expect("unable to read test file");
    let ofx = Ofx::from_str(&input)?;

    let transaction_list = ofx
        .body
        .credit_card
        .expect("missing credit card statement")
        .transaction_response
        .statement
        .bank_transactions
        .expect("missing transaction list");

    assert_eq!(transaction_list.dtstart.unwrap(), "20190501");
    assert_eq!(transaction_list.dtend.unwrap(), "20190531");
    assert_eq!(transaction_list.transactions.len(), 3);

    let trn = &transaction_list.transactions[0];
    assert_eq!(trn.transaction_type, TransactionType::Payment);
    assert_eq!(
        trn.date_posted,
        Utc.with_ymd_and_hms(2019, 5, 10, 0, 0, 0).unwrap()
    );

    assert_eq!(trn.amount, Decimal::from_str("-16.4").unwrap());
    assert_eq!(trn.id, "1DFE2867-0DE3-4357-B865-16986CBFC10A");
    assert_eq!(trn.memo, None);
    assert_eq!(trn.currency, None);

    let trn = &transaction_list.transactions[1];
    assert_eq!(trn.transaction_type, TransactionType::Payment);
    assert_eq!(
        trn.date_posted,
        Utc.with_ymd_and_hms(2019, 5, 12, 0, 0, 0).unwrap()
    );

    assert_eq!(trn.amount, Decimal::from(-10));
    assert_eq!(trn.id, "25518539-814F-4C2C-97E8-3B49A6D48A45");
    assert_eq!(trn.memo.as_deref(), Some("Example International Payment "));
    assert_eq!(
        trn.currency,
        Some(Currency {
            rate: Decimal::from_str("1.1153").unwrap(),
            symbol: "EUR".to_owned(),
        }),
    );

    let trn = &transaction_list.transactions[2];
    assert_eq!(trn.transaction_type, TransactionType::Credit);
    assert_eq!(
        trn.date_posted,
        Utc.with_ymd_and_hms(2019, 5, 20, 0, 0, 0).unwrap()
    );
    assert_eq!(trn.amount, Decimal::from(1_000));
    assert_eq!(trn.id, "3088354E-018B-41D1-ABA7-DB8F66F7B2DB");
    assert_eq!(trn.memo.as_deref(), Some("PAYMENT RECEIVED"));
    assert_eq!(trn.currency, None);

    Ok(())
}

#[test]
fn test_ofx_simple() -> Result<()> {
    let input = std::fs::read_to_string("tests/files/simple.ofx")?;

    let ofx: Ofx = input.parse()?;

    assert_eq!(ofx.header.ofxheader, 100);
    assert_eq!(ofx.header.version, Version::V102);
    assert_eq!(ofx.header.encoding, Encoding::UsAscii);
    assert_eq!(ofx.header.compression, "NONE");

    assert_eq!(
        ofx.body.sign_on.expect("no sign on").response.status.code,
        0
    );

    let transaction_response = ofx
        .body
        .credit_card
        .expect("missing credit card statement")
        .transaction_response;
    assert_eq!(&transaction_response.statement.account.id, "abc123");

    let transactions = transaction_response
        .statement
        .bank_transactions
        .expect("missing bank transactions")
        .transactions;
    assert_eq!(transactions.len(), 4);
    assert_eq!(transactions[0].name, Some("SOME HOSPITAL SOMEWHERE".into()));
    Ok(())
}

fn ofx_files_from_dir(path: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    Ok(read_dir(path)?
        .filter_map(result::Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            matches!(
                path.extension()
                    .and_then(OsStr::to_str)
                    .map(str::to_lowercase)
                    .as_deref(),
                Some("ofx" | "qfx")
            )
        })
        .collect())
}

#[test]
fn parses_files() -> result::Result<(), Box<dyn std::error::Error>> {
    let source_dirs = ["sgmlish", "ofx-reader", "ofxparse", "ofxparser"]
        .into_iter()
        .map(|dir| "tests/files/".to_owned() + dir);

    for dir in source_dirs {
        for entry in ofx_files_from_dir(dir)? {
            let skip_tests = [
                // 'fre' is not a valid ISO 639-1 or 639-3 code.
                "tests/files/ofxparser/ofxdata-oneline.ofx",
                // Language is a required tag per spec; in this test file the
                // tag is present but value is empty. Should this be permitted?
                "tests/files/ofxparse/ofx-v102-empty-tags.ofx",
                // OFX version 2
                "tests/files/ofx-reader/suncorp.ofx",
                "tests/files/ofxparse/anzcc.ofx",
                "tests/files/ofxparse/error_message.ofx",
                "tests/files/ofxparse/multiple_accounts.ofx",
                "tests/files/ofxparse/multiple_accounts2.ofx",
                "tests/files/ofxparse/suncorp.ofx",
                "tests/files/ofxparser/ofx-multiple-accounts-xml.ofx",
                "tests/files/ofxparser/ofxdata-banking-xml200.ofx",
                "tests/files/ofxparser/ofxdata-investments-multiple-accounts-xml.ofx",
                "tests/files/ofxparser/ofxdata-investments-oneline-xml.ofx",
                "tests/files/ofxparser/ofxdata-investments-xml.ofx",
                "tests/files/ofxparser/ofxdata-xml.ofx",
                // Bad datetimes
                //   has `00000000000000` as a date
                "tests/files/ofx-reader/Bradesco.ofx",
                //   bad timezone: `[-:EST]`
                "tests/files/ofxparse/investment_medium.ofx",
            ];

            if skip_tests.contains(&entry.display().to_string().as_str()) {
                continue;
            }

            let bytes = std::fs::read(&entry)?;
            let mut detector = chardetng::EncodingDetector::new();
            detector.feed(&bytes, true);
            let encoding = detector.guess(None, true);
            let (text, _, had_errors) = encoding.decode(&bytes);
            assert!(!had_errors);

            if let Err(e) = text.parse::<Ofx>() {
                println!("Failed to parse OFX file: {:?}", entry.display());
                panic!("Error: {e:?}");
            }
        }
    }
    Ok(())
}
