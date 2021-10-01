use tokio::sync::mpsc;

/// Implementors of this trait provide a method which begin the reading of transactions from an
/// arbitrary source and send it to a returned [`mpsc::Receiver`] which may be read from to begin
/// transaction processing
pub trait TransactionReader {
	fn start(self) -> mpsc::Receiver<super::Transaction>;

	// TODO: A cancellation method for something like a TCP stream
}
