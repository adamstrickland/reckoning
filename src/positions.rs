use crate::inout::Record;
use anyhow::Result;
use itertools::Itertools;
use std::convert::From;
use std::collections::HashSet;

const DEPOSIT: &str = "deposit";
const WITHDRAWAL: &str = "withdrawal";
const DISPUTE: &str = "dispute";
const RESOLVE: &str = "resolve";
const CHARGEBACK: &str = "chargeback";

#[derive(Debug, Clone)]
pub struct Transaction {
    pub tx_type: String,
    pub client_id: u16,
    pub tx_id: u32,
    pub raw_amount: f64,
    pub directed_amount: f64,
}

impl Transaction {
    fn deposit(client_id: u16, tx_id: u32, amount: f64) -> Transaction {
        Transaction {
            client_id,
            tx_type: String::from(DEPOSIT),
            tx_id,
            raw_amount: amount,
            directed_amount: amount,
        }
    }

    fn withdrawal(client_id: u16, tx_id: u32, amount: f64) -> Transaction {
        Transaction {
            client_id,
            tx_type: String::from(WITHDRAWAL),
            tx_id,
            raw_amount: amount,
            directed_amount: -amount,
        }
    }

    fn dispute(client_id: u16, tx_id: u32) -> Transaction {
        Transaction {
            client_id,
            tx_type: String::from(DISPUTE),
            tx_id,
            raw_amount: 0.0,
            directed_amount: 0.0,
        }
    }

    fn resolve(client_id: u16, tx_id: u32) -> Transaction {
        Transaction {
            client_id,
            tx_type: String::from(RESOLVE),
            tx_id,
            raw_amount: 0.0,
            directed_amount: 0.0,
        }
    }

    fn chargeback(client_id: u16, tx_id: u32) -> Transaction {
        Transaction {
            client_id,
            tx_type: String::from(CHARGEBACK),
            tx_id,
            raw_amount: 0.0,
            directed_amount: 0.0,
        }
    }
}

impl From<Record> for Transaction {
    fn from(r: Record) -> Self {
        return match r.tx_type.as_str() {
            DEPOSIT => Transaction::deposit(r.client_id, r.tx_id, r.amount.unwrap()),
            WITHDRAWAL => Transaction::withdrawal(r.client_id, r.tx_id, r.amount.unwrap()),
            DISPUTE => Transaction::dispute(r.client_id, r.tx_id),
            RESOLVE => Transaction::resolve(r.client_id, r.tx_id),
            CHARGEBACK => Transaction::chargeback(r.client_id, r.tx_id),
            _ => Transaction::deposit(r.client_id, r.tx_id, 0.0),
        };
    }
}

#[derive(Debug)]
pub struct Position {
    pub client_id: u16,
    pub available: f64,
    pub held: f64,
    pub total: f64,
    pub locked: bool,
}

pub type PositionVector = Vec<Position>;

fn to_position(client_id: u16, txns: &Vec<&Transaction>) -> Position {
    fn apply_amount(bal: f64, amt: f64) -> f64 {
        let tot = bal + amt;
        return if tot < 0.0 {
            bal
        } else {
            tot
        }
    }

    fn diag<I, T>(_s: &str, _v: I)
    where T: std::fmt::Debug,
          I: IntoIterator<Item = T> {
        // println!("{}:", _s);
        // for t in _v { println!(" => {:?}", t) }
    }

    diag("txns", txns);

    let chargebacks: HashSet<u32> = txns
        .iter()
        .filter(|t| t.tx_type == CHARGEBACK)
        .map(|t| t.tx_id)
        .collect();
    diag("chargebacks", &chargebacks);
    let resolves: HashSet<u32> = txns
        .iter()
        .filter(|t| t.tx_type == RESOLVE)
        .map(|t| t.tx_id)
        .collect();
    diag("resolves", &resolves);
    let disputes: HashSet<u32> = txns
        .iter()
        .filter(|t| t.tx_type == DISPUTE)
        .map(|t| t.tx_id)
        .collect();
    diag("disputes", &disputes);
    let dedisputed: HashSet<u32> = chargebacks.union(&resolves)
        .cloned()
        .collect();
    diag("dedisputed", &dedisputed);
    let disputed: Vec<u32> = (&disputes - &dedisputed)
        .iter()
        .cloned()
        .collect();
    diag("disputed", &disputed);
    let disputeds: Vec<&Transaction> = disputed
        .into_iter()
        .map(|txid| *txns.iter().find(|t| t.tx_id == txid).unwrap())
        .collect();
    diag("disputeds", &disputeds);
    let resolveds: Vec<&Transaction> = resolves
        .into_iter()
        .map(|txid| *txns.iter().find(|t| t.tx_id == txid).unwrap())
        .collect();
    diag("resolveds", &resolveds);

    let undisputed = txns
        .iter()
        .map(|t| t.directed_amount)
        .fold(0.0, apply_amount);
    let resolved = resolveds
        .iter()
        .map(|t| t.raw_amount)
        .fold(0.0, apply_amount);
    let held = disputeds
        .iter()
        .map(|t| t.raw_amount)
        .fold(0.0, apply_amount);

    let available = undisputed + resolved;
    let total = available + held;

    return Position{
        client_id,
        available,
        held,
        total,
        locked: chargebacks.len() > 0,
    }
}

pub(crate) fn to_positions(transactions: &Vec<Transaction>) -> Result<PositionVector> {
    let positions: PositionVector = transactions
        .into_iter()
        .into_group_map_by(|t| t.client_id)
        .iter()
        .map(|(cid, txns)| to_position(*cid, txns))
        .collect();
    return Ok(positions);
}

#[cfg(test)]
mod tests {
    use crate::positions::{to_positions,Transaction,WITHDRAWAL,DEPOSIT};
    use crate::inout::Record;

    #[test]
    fn from_record_to_transaction_deposit() {
        let r = Record{
            tx_type: String::from(DEPOSIT),
            client_id: 1,
            tx_id: 1,
            amount: Some(1.0),
        };
        let t = Transaction::from(r.clone());
        assert_eq!(t.client_id, r.client_id);
        assert_eq!(t.raw_amount, r.amount.unwrap());
        assert_eq!(t.directed_amount, r.amount.unwrap());
    }

    #[test]
    fn from_record_to_transaction_withdrawal() {
        let r = Record{
            tx_type: String::from(WITHDRAWAL),
            client_id: 1,
            tx_id: 1,
            amount: Some(1.0),
        };
        let t = Transaction::from(r.clone());
        assert_eq!(t.client_id, r.client_id);
        assert_eq!(t.raw_amount, r.amount.unwrap());
        assert_eq!(t.directed_amount, -r.amount.unwrap());
    }

    #[test]
    fn new_deposit() {
        let t = Transaction::deposit(1, 1, 1.0);
        assert_eq!(t.client_id, 1);
        assert_eq!(t.raw_amount, 1.0);
        assert_eq!(t.directed_amount, 1.0);
    }

    #[test]
    fn new_withdrawal() {
        let t = Transaction::withdrawal(1, 1, 1.0);
        assert_eq!(t.client_id, 1);
        assert_eq!(t.raw_amount, 1.0);
        assert_eq!(t.directed_amount, -1.0);
    }

    fn one_good_transaction() -> Transaction {
        return Transaction::deposit(1, 1, 1.0);
    }

    fn one_clients_transactions() -> Vec<Transaction> {
        return [
            one_good_transaction(),
            Transaction::deposit(1, 2, 2.0),
            Transaction::withdrawal(1, 4, 1.5),
        ].to_vec();
    }

    fn multiple_clients_transactions() -> Vec<Transaction> {
        let mut txns = one_clients_transactions();
        txns.extend([
            Transaction::deposit(2, 3, 2.0),
            Transaction::withdrawal(2, 5, 3.0),
        ]);
        return txns;
    }

    #[test]
    fn to_positions_happy_path_is_ok() {
        let subj = to_positions(&multiple_clients_transactions());
        assert!(!subj.is_err());
    }

    #[test]
    fn to_positions_given_one_good_txn_it_aggregates_to_one_position() {
        let rec = one_good_transaction();
        let subj = to_positions(&[ rec.clone() ].to_vec())
            .unwrap();
        assert_eq!(subj.len(), 1);
        assert_eq!(subj[0].client_id, rec.client_id);
        assert_eq!(subj[0].total,     rec.raw_amount);
        assert_eq!(subj[0].available, rec.raw_amount);
    }

    #[test]
    fn to_positions_given_multiple_good_txns_it_aggregates_to_one_position() {
        let txns = one_clients_transactions();
        let subj = to_positions(&txns).unwrap();
        assert_eq!(subj.len(), 1);
        assert_eq!(subj[0].client_id, one_good_transaction().client_id);
        assert_eq!(subj[0].total,     1.5);
        assert_eq!(subj[0].available, 1.5);
    }

    #[test]
    fn to_positions_given_multiple_clients_txns_it_aggregates_to_multiple_positions() {
        let txns = multiple_clients_transactions();
        let mut subj = to_positions(&txns).unwrap();
        let first_clients_id = one_good_transaction().client_id;

        assert_eq!(subj.len(), 2);

        subj.sort_by(|l, r| l.client_id.cmp(&r.client_id));

        assert_eq!(subj[0].client_id, first_clients_id);
        assert_eq!(subj[0].total,     1.5);
        assert_eq!(subj[0].available, 1.5);
        assert_ne!(subj[1].client_id, first_clients_id);
        assert_eq!(subj[1].total,     2.0);
        assert_eq!(subj[1].available, 2.0);
    }

    fn one_clients_transactions_with_dispute() -> Vec<Transaction> {
        let mut txns = one_clients_transactions();
        txns.extend([
            Transaction::dispute(1, 4),
        ]);
        return txns;
    }

    fn one_clients_transactions_with_dispute_and_resolution() -> Vec<Transaction> {
        let mut txns = one_clients_transactions();
        txns.extend([
            Transaction::dispute(1, 4),
            Transaction::resolve(1, 4),
        ]);
        return txns;
    }

    fn one_clients_transactions_with_dispute_and_chargeback() -> Vec<Transaction> {
        let mut txns = one_clients_transactions();
        txns.extend([
            Transaction::dispute(1, 4),
            Transaction::chargeback(1, 4),
        ]);
        return txns;
    }

    #[test]
    fn to_positions_given_dispute() {
        let txns = one_clients_transactions_with_dispute();
        let subj = to_positions(&txns).unwrap();
        assert_eq!(subj.len(), 1);
        assert_eq!(subj[0].client_id, one_good_transaction().client_id);
        assert_eq!(subj[0].total,     3.0);
        assert_eq!(subj[0].available, 1.5);
        assert_eq!(subj[0].held,      1.5);
    }

    #[test]
    fn to_positions_given_dispute_and_resolution() {
        let txns = one_clients_transactions_with_dispute_and_resolution();
        let subj = to_positions(&txns).unwrap();
        assert_eq!(subj.len(), 1);
        assert_eq!(subj[0].client_id, one_good_transaction().client_id);
        assert_eq!(subj[0].total,     3.0);
        assert_eq!(subj[0].available, 3.0);
        assert_eq!(subj[0].held,      0.0);
    }

    #[test]
    fn to_positions_given_dispute_and_chargeback() {
        let txns = one_clients_transactions_with_dispute_and_chargeback();
        let subj = to_positions(&txns).unwrap();
        assert_eq!(subj.len(), 1);
        assert_eq!(subj[0].client_id, one_good_transaction().client_id);
        assert_eq!(subj[0].total,     1.5);
        assert_eq!(subj[0].available, 1.5);
        assert_eq!(subj[0].held,      0.0);
        assert_eq!(subj[0].locked,    true);
    }

}
