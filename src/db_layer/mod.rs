use tokio::sync::mpsc;

use super::*;

/// The layer which stores `Client`s, processes `Transaction`s, and streams the stored `Client`s
/// after all `Transaction`s  have been processed
pub trait DbLayer {
    type Error;

	/// Apply a single transaction to the DbLayer implementor
	fn write_transaction(&mut self, transaction: Transaction) -> Result<(), Self::Error>;

	/// Return a [`mpsc::Receiver`] which streams all of the stored `Client`s for outputting data
	fn stream_clients(&mut self) -> Result<mpsc::Receiver<Client>, Self::Error>;
}
