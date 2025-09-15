use std::str::FromStr;

use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, Utc};
use rust_decimal::Decimal;
use serde::{self, Deserialize, Deserializer};
use sgmlish::Parser;

use crate::{Result, error::Error};

fn deserialize_datetime<'de, D>(deserializer: D) -> std::result::Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error as SerdeErr;
    let s = String::deserialize(deserializer)?;

    // Per 1.6 spec, 3.2.8.2:
    // Note that times zones are specified by an offset and optionally, a time zone name. The offset
    // defines the time zone. Valid offset values are in the range from –12 to +12 for whole number
    // offsets. Formatting is +12.00 to -12.00 for fractional offsets, plus sign may be omitted.
    let (dt_str, offset_str) = s.find('[').map_or_else(
        || (&s[..], "0"),
        |idx| (&s[..idx], &s[idx + 1..s.len() - 1]),
    );
    let offset_num = offset_str
        .split(':')
        .next()
        .ok_or_else(|| SerdeErr::custom(format!("invalid timezone offset format: {offset_str}")))?
        .parse::<f32>()
        .map_err(serde::de::Error::custom)?;

    if offset_num.abs() > 12.0 {
        return Err(SerdeErr::custom(format!(
            "timezone offset too large or small: {offset_num}"
        )));
    }
    #[allow(clippy::cast_possible_truncation)]
    let offset_secs = (offset_num * 3600.0).round() as i32;

    let offset = FixedOffset::east_opt(offset_secs)
        .ok_or_else(|| SerdeErr::custom(format!("invalid timezone offset: {offset_num}")))?;

    let naive = match dt_str.chars().count() {
        18 => NaiveDateTime::parse_from_str(dt_str, "%Y%m%d%H%M%S%.3f"),
        14 => NaiveDateTime::parse_from_str(dt_str, "%Y%m%d%H%M%S"),
        8 => Ok(NaiveDate::parse_from_str(dt_str, "%Y%m%d")
            .map_err(|err| SerdeErr::custom(format!("unable to parse as date: {dt_str}: {err}")))?
            .and_hms_opt(0, 0, 0)
            .expect("couldn't set time to 00:00:00")),
        _ => {
            return Err(SerdeErr::custom(format!("invalid datetime: {dt_str}")));
        }
    };

    naive
        .map_err(|err| SerdeErr::custom(format!("unable to parse '{s}' as datetime: {err}")))?
        .and_local_timezone(offset)
        .single()
        .map(|dt| dt.with_timezone(&Utc))
        .ok_or_else(|| SerdeErr::custom(format!("ambiguous or invalid local datetime: {s}")))
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub struct BankTransactionList {
    pub dtstart: Option<String>,
    pub dtend: Option<String>,
    #[serde(rename = "STMTTRN")]
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub struct Transaction {
    #[serde(rename = "TRNTYPE")]
    pub transaction_type: TransactionType,
    #[serde(rename = "DTPOSTED")]
    #[serde(deserialize_with = "deserialize_datetime")]
    pub date_posted: DateTime<Utc>,
    #[serde(rename = "TRNAMT")]
    pub amount: Decimal,
    #[serde(rename = "FITID")]
    pub id: String,
    pub name: Option<String>,
    pub memo: Option<String>,
    pub currency: Option<Currency>,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum TransactionType {
    Credit,
    Debit,
    #[serde(rename = "INT")]
    Interest,
    #[serde(rename = "DIV")]
    Dividend,
    Fee,
    #[serde(rename = "SRVCHG")]
    ServiceCharge,
    #[serde(rename = "DEP")]
    Deposit,
    Atm,
    #[serde(rename = "POS")]
    PointOfSale,
    #[serde(rename = "XFER")]
    Transfer,
    Check,
    Payment,
    Cash,
    #[serde(rename = "DIRECTDEP")]
    DirectDeposit,
    DirectDebit,
    #[serde(rename = "REPEATPMT")]
    RepeatPayment,
    Other,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct Currency {
    #[serde(rename = "CURRATE")]
    pub rate: Decimal,
    #[serde(rename = "CURSYM")]
    pub symbol: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct SignOnMessageResponse {
    #[serde(rename = "SONRS")]
    pub response: SignOnResponse,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct FinancialInstitution {
    #[serde(rename = "ORG")]
    pub organization: String,
    #[serde(rename = "FID")]
    pub id: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct SignOnResponse {
    #[serde(rename = "STATUS")]
    pub status: Status,
    #[serde(rename = "DTSERVER")]
    #[serde(deserialize_with = "deserialize_datetime")]
    pub server_date: DateTime<Utc>,
    #[serde(rename = "LANGUAGE")]
    pub language: Language,
    #[serde(rename = "FI")]
    pub financial_institution: Option<FinancialInstitution>,
}

#[derive(Debug, PartialEq)]
pub struct Language(isolang::Language);

impl<'de> serde::Deserialize<'de> for Language {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        let lang = String::deserialize(d)?.trim().to_lowercase();
        let parsed = lang.parse().map_err(serde::de::Error::custom)?;
        Ok(Language(parsed))
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct CreditCardMessageResponse {
    #[serde(rename = "CCSTMTTRNRS")]
    pub transaction_response: CreditCardStatementTransactionResponse,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct CreditCardStatementTransactionResponse {
    #[serde(rename = "TRNUID")]
    pub transaction_id: String,
    #[serde(rename = "STATUS")]
    pub status: Status,
    #[serde(rename = "CCSTMTRS")]
    pub statement: CreditCardStatementResponse,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct CreditCardStatementResponse {
    #[serde(rename = "CURDEF")]
    pub currency: String,
    #[serde(rename = "CCACCTFROM")]
    pub account: Account,
    #[serde(rename = "BANKTRANLIST")]
    pub bank_transactions: Option<BankTransactionList>,
    #[serde(rename = "LEDGERBAL")]
    pub ledger_balance: Balance,
    #[serde(rename = "AVAILBAL")]
    pub available_balance: Option<Balance>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Account {
    #[serde(rename = "ACCTID")]
    pub id: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Status {
    #[serde(rename = "CODE")]
    pub code: u32,
    #[serde(rename = "SEVERITY")]
    pub severity: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Balance {
    #[serde(rename = "BALAMT")]
    pub amount: String,
    #[serde(rename = "DTASOF")]
    #[serde(deserialize_with = "deserialize_datetime")]
    pub date: DateTime<Utc>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct BankMessageResponse {
    #[serde(rename = "STMTTRNRS")]
    pub transaction_response: StatementTransactionResponse,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct StatementTransactionResponse {
    #[serde(rename = "TRNUID")]
    pub transaction_id: String,
    #[serde(rename = "STATUS")]
    pub status: Status,
    #[serde(rename = "STMTRS")]
    pub statement: StatementResponse,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct StatementResponse {
    #[serde(rename = "CURDEF")]
    pub currency: String,
    #[serde(rename = "BANKACCTFROM")]
    pub account: Option<BankAccount>,
    #[serde(rename = "BANKTRANLIST")]
    pub bank_transactions: Option<BankTransactionList>,
    #[serde(rename = "LEDGERBAL")]
    pub ledger_balance: Option<Balance>,
    #[serde(rename = "AVAILBAL")]
    pub available_balance: Option<Balance>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct BankAccount {
    #[serde(rename = "BANKID")]
    pub bank_id: String,
    #[serde(rename = "ACCTID")]
    pub id: String,
    #[serde(rename = "ACCTTYPE")]
    pub account_type: AccountType,
}

// 11.3.1.2 Account Types for <ACCTTYPE> and <ACCTTYPE2> Elements
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum AccountType {
    Checking,
    Savings,
    Moneymrkt,
    Creditline,
    Cma,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Body {
    #[serde(rename = "SIGNONMSGSRSV1")]
    pub sign_on: Option<SignOnMessageResponse>,
    #[serde(rename = "CREDITCARDMSGSRSV1")]
    pub credit_card: Option<CreditCardMessageResponse>,
    #[serde(rename = "BANKMSGSRSV1")]
    pub bank: Option<BankMessageResponse>,
}

impl FromStr for Body {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let sgml = Parser::builder()
            .expand_entities(|entity| match entity {
                "lt" => Some("<"),
                "gt" => Some(">"),
                "amp" => Some("&"),
                "nbsp" => Some(" "),
                _ => None,
            })
            .parse(s)?;
        let sgml = sgmlish::transforms::normalize_end_tags(sgml)?;
        Ok(sgmlish::from_fragment::<Body>(sgml)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::{TimeZone, Timelike, Utc};

    use serde::de::IntoDeserializer;
    use serde::de::value::{Error as SerdeError, StrDeserializer};

    #[test]
    fn test_parse_date_spec_example() {
        // Per 1.6 spec, 3.2.8.1
        // There is one format for representing dates, times, and time zones. The complete form is:
        // YYYYMMDDHHMMSS.XXX [gmt offset[:tz name]]
        //
        // 3.2.8.2
        // For example, “19961005132200.124[-5:EST]”
        // This is the same as 6:22 p.m. Greenwich Mean Time (GMT).

        let examples = [
            (
                "19961005132200.124[-5:EST]",
                Utc.with_ymd_and_hms(1996, 10, 5, 18, 22, 0)
                    .unwrap()
                    .with_nanosecond(124_000_000)
                    .unwrap(),
            ),
            (
                "19961005132200.124[-5]",
                Utc.with_ymd_and_hms(1996, 10, 5, 18, 22, 0)
                    .unwrap()
                    .with_nanosecond(124_000_000)
                    .unwrap(),
            ),
            (
                "19961005132200.124",
                Utc.with_ymd_and_hms(1996, 10, 5, 13, 22, 0)
                    .unwrap()
                    .with_nanosecond(124_000_000)
                    .unwrap(),
            ),
            (
                "19961005132200",
                Utc.with_ymd_and_hms(1996, 10, 5, 13, 22, 0).unwrap(),
            ),
            (
                "19961005",
                Utc.with_ymd_and_hms(1996, 10, 5, 0, 0, 0).unwrap(),
            ),
        ];

        for (input, expected) in examples {
            let deserializer: StrDeserializer<SerdeError> = input.into_deserializer();
            assert_eq!(deserialize_datetime(deserializer), Ok(expected));
        }
    }

    #[test]
    fn test_alt_timezone_formats() {
        // Per 1.6 spec, 3.2.8.2:
        // Note that times zones are specified by an offset and optionally, a time zone name. The offset
        // defines the time zone. Valid offset values are in the range from –12 to +12 for whole number
        // offsets. Formatting is +12.00 to -12.00 for fractional offsets, plus sign may be omitted.
        let examples = [
            "19961005132200.124[-5:EST]",
            "19961005132200.124[-5]",
            "19961005132200.124[-5:]",
            "19961005132200.124[-5.0:EST]",
            "19961005212200.124[+3:EST]",
            "19961005212200.124[+3]",
            "19961005212200.124[+3:]",
            "19961005212200.124[+3.0:EST]",
        ];

        let expected = Utc
            .with_ymd_and_hms(1996, 10, 5, 18, 22, 0)
            .unwrap()
            .with_nanosecond(124_000_000)
            .unwrap();

        for input in examples {
            let deserializer: StrDeserializer<SerdeError> = input.into_deserializer();
            assert_eq!(deserialize_datetime(deserializer), Ok(expected));
        }
    }
}
