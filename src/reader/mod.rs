use tokio::sync::mpsc;

use super::Transaction;

pub mod csv;

/// Implementors of this trait provide a method which begin the reading of transactions from an
/// arbitrary source and send it to a returned [`mpsc::Receiver`] which may be read from to begin
/// transaction processing

// TODO: A way of cancelling a Reader for something like a TCP stream
// TODO: A way of sending back errors to something like a TCP stream eg. if there is an attemped
// withdrawal above the available funds
pub trait TransactionReader {
    fn start(self) -> mpsc::Receiver<Transaction>;
}
