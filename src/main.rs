use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt;

mod db_layer;
mod fixed_point_util;
mod reader;
mod transaction_processing;
mod writer;

use db_layer::DbLayer;
use reader::TransactionReader;
use writer::ClientWriter;

/// The number of [`Transaction`]s to allow in the [`tokio::sync::mpsc::Receiver`]'s queue. Each
/// [`Transaction`] will be roughly 120 bytes (plus padding) and the overhead of the mpsc channel.
// TODO: Make this number configurable
const READER_BUFFER: usize = 1024;

/// The number of [`Client`]s to allow in the [`tokio::sync::mpsc::Receiver`]'s queue. Each
/// [`Client`] will be roughly 200 bytes (plus padding) and the overhead of the mpsc channel.
// TODO: Make this number configurable
const DB_BUFFER: usize = 1024;

/// The path of the RocksDB key value store
// TODO: Make this path configurable
const DB_PATH: &str = "./database";

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

/// A single transaction meant to be readable by a human
#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone)]
pub struct HumanReadableTransaction {
    /// Types including deposits, withdrawals, disputes, resolutions of disputes, and chargebacks
    #[serde(rename = "type")]
    ty: TransactionType,

    /// A unique client ID for which all transactions are tied to
    client: u16,

    /// A unique transaction ID given to deposits or withdrawals. Disputes, resolutions, and
    /// chargebacks reference transaction IDs of deposits
    tx: u32,

    /// The amount of the deposit or withdrawal. This field will be None for any other TransactionType
    #[serde(default, deserialize_with="fixed_point_util::deserialize")]
    amount: Option<i64>,
}

impl From<HumanReadableTransaction> for Transaction {
	fn from(transaction: HumanReadableTransaction) -> Transaction {
		Transaction {
			ty: transaction.ty,
			client: transaction.client,
			tx: transaction.tx,
			amount: transaction.amount,
		}
	}
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

/// A single client's data to be output by the application in a human readable format
#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Debug)]
pub struct HumanReadableClient {
    /// The client ID
    client: u16,

    /// The total funds available for withdrawal or other use.
    #[serde(serialize_with = "fixed_point_util::serialize")]
    available: i64,

    /// The total funds held for dispute
    #[serde(serialize_with = "fixed_point_util::serialize")]
    held: i64,

    /// The total funds of the account disputed or not. Equal to available + held.
    #[serde(serialize_with = "fixed_point_util::serialize")]
    total: i64,

    /// Whether the account has been locked after a chargeback
    locked: bool,
}

impl From<Client> for HumanReadableClient {
	fn from(client: Client) -> HumanReadableClient {
    	HumanReadableClient {
			client: client.client,
			available: client.available,
			held: client.held,
			total: client.total,
			locked: client.locked,
    	}
	}
}

// FIXME: Eliminate unwraps
#[tokio::main]
async fn main() {
    let mut args = std::env::args();
    let _ = args.next();

    // Read from a CSV file with the path given in the first argument
    let reader = reader::csv::CsvReader::new(
        args.next()
            .expect("Must have one argument with the path of a CSV file"),
        READER_BUFFER,
    )
    .await
    .unwrap();
    let mut receiver = reader.start();

    // Using a couple of `HashMaps` or a `sled::Db`, hold transaction and client information
    #[cfg(feature = "no_persist")]
    let mut db_layer = db_layer::hashmap::HashMapDb::new(DB_BUFFER);
    #[cfg(not(feature = "no_persist"))]
    let mut db_layer = db_layer::sled_db::SledDb::new(DB_PATH, DB_BUFFER).unwrap();

    // Process each transaction from the reader's receiver
    while let Some(input) = receiver.recv().await {
        // Fail silently here
        let _ = transaction_processing::process_transaction(&mut db_layer, input).await;
    }

    // When all transactions in the batch have been processed, write the final state of each Client
    // to stdout
    let mut receiver = db_layer.stream_clients().await;
    let mut writer = writer::csv::CsvWriter::new();
    while let Some(output) = receiver.recv().await {
        writer.append_client(output.unwrap()).await.unwrap();
    }

    // And you'd close the ClientWriter here were it any implementation where thismethod does
    // something
    // writer.close().await.unwrap();
}
