use async_trait::async_trait;
use tokio::sync::mpsc;

use super::*;

pub mod hashmap;
pub mod sled_db;

/// The layer which stores `Client`s, processes `Transaction`s, and streams the stored `Client`s
/// after all `Transaction`s  have been processed
#[async_trait]
pub trait DbLayer {
    /// Write a single transaction to the DbLayer implementor
    async fn write_transaction(&mut self, transaction: Transaction) -> Result<(), Error>;

    /// Get a single transaction from the DBLayer implementor
    async fn get_transaction(&mut self, transaction_id: u32) -> Result<Option<Transaction>, Error>;

    /// Write a single client to the DbLayer implementor
    async fn write_client(&mut self, client: Client) -> Result<(), Error>;

    /// Get a single client from the DbLayer implementor
    async fn get_client(&mut self, client_id: u16) -> Result<Option<Client>, Error>;

    /// Return a [`mpsc::Receiver`] which streams all of the stored `Client`s for outputting data
    async fn stream_clients(self) -> mpsc::Receiver<Result<Client, Error>>;
}
