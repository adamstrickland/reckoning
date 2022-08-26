use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use serde::Serializer;

use crate::positions;

fn _ser_f64<S>(f: &f64, s: S) -> Result<S::Ok, S::Error>
    where S: Serializer
{
    s.serialize_str(&format!("{:.4}", f))
}


#[derive(Debug, Serialize)]
#[serde(rename_all="snake_case")]
pub struct Balance {
    pub client: u16,
    #[serde(serialize_with = "_ser_f64")]
    pub available: f64,
    #[serde(serialize_with = "_ser_f64")]
    pub held: f64,
    #[serde(serialize_with = "_ser_f64")]
    pub total: f64,
    pub locked: bool,
}

impl From<positions::Position> for Balance {
    fn from(p: positions::Position) -> Self {
        return Balance {
           client: p.client_id,
           available: p.available,
           held: p.held,
           total: p.total,
           locked: p.locked,
        };
    }
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub(crate) struct Record {
    #[serde(rename = "type")]
    pub tx_type: String,

    #[serde(rename = "client")]
    pub client_id: u16,

    #[serde(rename = "tx")]
    pub tx_id: u32,

    #[serde(rename = "amount")]
    pub amount: Option<f64>,
}


pub(crate) fn records_from_file(path: String) -> Result<Vec<Record>> {
    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .from_path(path)?;

    let mut records: Vec<Record> = Vec::new();
    for result in rdr.deserialize() {
        let record: Record = result?;
        records.push(record);
    }

    Ok(records)
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use crate::inout::records_from_file;

    fn fixture_path(rel_path: &str) -> String {
        return Path::new(file!())
            .parent()
            .unwrap()
            .join(rel_path)
            .into_os_string()
            .into_string()
            .unwrap();
    }

    fn good_path() -> String {
        return fixture_path("../tests/support/good_transactions.csv");
    }

    fn bad_path() -> String {
        return fixture_path("../tests/supporx/bad_transactions.csv");
    }

    fn good_path_bad_data() -> String {
        return fixture_path("../tests/support/bad_transactions.csv");
    }

    #[test]
    fn records_from_file_with_good_path_is_ok() {
        let subj = records_from_file(good_path());
        assert!(!subj.is_err());
    }

    #[test]
    fn records_from_file_with_good_path_parses_file() {
        let subj = &records_from_file(good_path())
            .unwrap()
            [0];        
        assert_eq!(subj.tx_id, 1);
    }

    #[test]
    fn records_from_file_with_bad_path_is_not_ok() {
        let subj = records_from_file(bad_path());
        assert!(subj.is_err());
    }

    #[test]
    fn records_from_file_with_good_path_bad_data_is_not_ok() {
        let subj = records_from_file(good_path_bad_data());
        assert!(subj.is_err());
    }
}
