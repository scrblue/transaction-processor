use serde::{Deserialize, Serialize};

use crate::fixed_point_util;

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
    /// If a Resolve or a Chargeback references a transaction that isn't disputed
    NotDisputed,

    /// An error in the DbLayer
    DbLayer(String),
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
    pub ty: TransactionType,

    /// A unique client ID for which all transactions are tied to
    pub client: u16,

    /// A unique transaction ID given to deposits or withdrawals. Disputes, resolutions, and
    /// chargebacks reference transaction IDs of deposits
    pub tx: u32,

    /// The amount of the deposit or withdrawal. This field will be None for any other TransactionType
    #[serde(default)]
    pub amount: Option<i64>,

    pub disputed: bool,
}

/// A single transaction meant to be readable by a human
#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone)]
pub struct HumanReadableTransaction {
    /// Types including deposits, withdrawals, disputes, resolutions of disputes, and chargebacks
    #[serde(rename = "type")]
    pub ty: TransactionType,

    /// A unique client ID for which all transactions are tied to
    pub client: u16,

    /// A unique transaction ID given to deposits or withdrawals. Disputes, resolutions, and
    /// chargebacks reference transaction IDs of deposits
    pub tx: u32,

    /// The amount of the deposit or withdrawal. This field will be None for any other TransactionType
    #[serde(default, deserialize_with = "fixed_point_util::deserialize")]
    pub amount: Option<i64>,
}

impl From<HumanReadableTransaction> for Transaction {
    fn from(transaction: HumanReadableTransaction) -> Transaction {
        Transaction {
            ty: transaction.ty,
            client: transaction.client,
            tx: transaction.tx,
            amount: transaction.amount,
            disputed: false,
        }
    }
}

/// A single client's data to be output by the application
#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Debug)]
pub struct Client {
    /// The client ID
    pub client: u16,

    /// The total funds available for withdrawal or other use.
    pub available: i64,

    /// The total funds held for dispute
    pub held: i64,

    /// The total funds of the account disputed or not. Equal to available + held.
    pub total: i64,

    /// Whether the account has been locked after a chargeback
    pub locked: bool,
}

/// A single client's data to be output by the application in a human readable format
#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Debug)]
pub struct HumanReadableClient {
    /// The client ID
    pub client: u16,

    /// The total funds available for withdrawal or other use.
    #[serde(serialize_with = "fixed_point_util::serialize")]
    pub available: i64,

    /// The total funds held for dispute
    #[serde(serialize_with = "fixed_point_util::serialize")]
    pub held: i64,

    /// The total funds of the account disputed or not. Equal to available + held.
    #[serde(serialize_with = "fixed_point_util::serialize")]
    pub total: i64,

    /// Whether the account has been locked after a chargeback
    pub locked: bool,
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
