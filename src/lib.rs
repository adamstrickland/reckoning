use anyhow::Result;
use std::io;

mod inout;
mod positions;

fn balances(records: Vec<inout::Record>) -> Result<Vec<inout::Balance>> {
    let transactions = records
        .into_iter()
        .map(|r| positions::Transaction::from(r))
        .collect();

    let positions = positions::to_positions(&transactions)?;

    let mut balances: Vec<inout::Balance> = positions
        .into_iter()
        .map(|p| inout::Balance::from(p))
        .collect();

    balances.sort_by(|l, r| l.client.cmp(&r.client));

    return Ok(balances);
}

pub fn run(path: String) -> Result<()> {
    let records = inout::records_from_file(path)?;
    let outrecs = balances(records)?;
    let mut wtr = csv::Writer::from_writer(io::stdout());

    for or in outrecs.iter() {
        wtr.serialize(or)?;
    }

    wtr.flush()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use crate::run;

    // fn fixture(file_name: &str) -> String {
    //     return fixture_path(format!("../tests/support/{}", file_name).as_str());
    // }

    // fn fixture_path(rel_path: &str) -> String {
    //     return Path::new(file!())
    //         .parent()
    //         .unwrap()
    //         .join(rel_path)
    //         .into_os_string()
    //         .into_string()
    //         .unwrap();
    // }

    // NOTE: since the instructions make clear that no portion of the instructions should be
    // committed, I don't feel comfortable committing the fixutre files, so I'm removing these
    // tests.
    //
    // #[test]
    // fn run_with_good_path_is_ok() {
    //     let subj = run(fixture("good_transactions.csv"));
    //     assert!(!subj.is_err());
    // }

    // #[test]
    // fn run_with_bad_path_is_not_ok() {
    //     let subj = run(fixture("file_that_doesnt_exist.csv"));
    //     assert!(subj.is_err());
    // }

    // #[test]
    // fn run_with_good_path_bad_data_is_not_ok() {
    //     let subj = run(fixture("bad_transactions.csv"));
    //     assert!(subj.is_err());
    // }

    // #[test]
    // fn run_with_good_path_with_dispute_is_ok() {
    //     let subj = run(fixture("good_transactions_with_dispute.csv"));
    //     assert!(!subj.is_err());
    // }

    // #[test]
    // fn run_with_good_path_with_dispute_and_resolution_is_ok() {
    //     let subj = run(fixture("good_transactions_with_dispute_and_resolution.csv"));
    //     assert!(!subj.is_err());
    // }
}
