use serde::{Serialize, Deserialize};

mod db_layer;
mod reader;

/// The number of [`Transaction`]s to allow in the [`tokio::sync::mpsc::Receiver`]'s queue. Each
/// [`Transaction`] will be roughly 120 bytes (plus padding) and the overhead of the mpsc channel.

// TODO: Make this number configurable
const READER_BUFFER: usize = 1024;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
	Deposit,
	Withdrawal,
	Dispute,
	Resolve,
	Chargeback,
}

/// A single transaction to be processed by the application
#[derive(Serialize, Deserialize, Debug, PartialEq)]
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
#[derive(Serialize, Deserialize)]
pub struct Client {
    /// The total funds available for withdrawal or other use.
	available: i64,

	/// The total funds held for dispute
	held: i64,

	/// The total funds of the account disputed or not. Equal to available + held.
	total: i64,

	/// Whether the account has been locked after a chargeback
	locked: bool,
}

fn main() {
    
}
