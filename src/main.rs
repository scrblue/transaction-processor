use serde::{Deserialize, Serialize};

mod db_layer;
mod reader;
mod transaction_processing;

/// The number of [`Transaction`]s to allow in the [`tokio::sync::mpsc::Receiver`]'s queue. Each
/// [`Transaction`] will be roughly 120 bytes (plus padding) and the overhead of the mpsc channel.
// TODO: Make this number configurable
const READER_BUFFER: usize = 1024;

/// The number of [`Client`]s to allow in the [`tokio::sync::mpsc::Receiver`]'s queue. Each
/// [`Client`] will be roughly 200 bytes (plus padding) and the overhead of the mpsc channel.
// TODO: Make this number configurable
const DB_BUFFER: usize = 1024;

/// The path of the RocksDB key value store
const ROCKS_DB_PATH: &str = "./database";

/// A global error type
#[derive(Debug)]
pub enum Error {
    /// If a Deposit or Withdrawal transaction has no amount
    NoAmount,
    /// If the Withdrawal can not process because of insufficient available funds
    InsufficientFunds,
    /// If the Dispute, Resolve, or Chargeback Transaction can not process because the referenced
    /// Transaction does not exist
    ReferenceDoesNotExist,
    /// If a Dispute, Resolve, or Chargeback transaction references a transaction with a differnt
    /// client than expected
    ReferencesWrongClient,

    /// An error in the DbLayer
    DbError(String),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

/// A single transaction to be processed by the application
#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone)]
pub struct Transaction {
    /// Types including deposits, withdrawals, disputes, resolutions of disputes, and chargebacks
    #[serde(rename = "type")]
    ty: TransactionType,

    /// A unique client ID for which all transactions are tied to
    client: u16,

    /// A unique transaction ID given to deposits or withdrawals. Disputes, resolutions, and
    /// chargebacks reference transaction IDs of deposits
    tx: u32,

    /// The amount of the deposit or withdrawal. This field will be None for any other TransactionType
    #[serde(default)]
    amount: Option<i64>,
}

/// A single client's data to be output by the application
#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Debug)]
pub struct Client {
    /// The client ID
    client: u16,

    /// The total funds available for withdrawal or other use.
    available: i64,

    /// The total funds held for dispute
    held: i64,

    /// The total funds of the account disputed or not. Equal to available + held.
    total: i64,

    /// Whether the account has been locked after a chargeback
    locked: bool,
}

fn main() {}
