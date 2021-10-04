use async_trait::async_trait;

use super::*;

pub mod csv;

#[async_trait]
pub trait ClientWriter {
    /// Append a [`Client`] to whatever output method the implementor uses.
    async fn append_client(&mut self, client: Client) -> Result<(), Error>;

    /// Close the `ClientWriter`, flushing any data
    async fn close(self) -> Result<(), Error>;
}
