use async_trait::async_trait;

use super::*;

/// Writes CSV values to stdout
pub struct CsvWriter {
    writer: csv_async::AsyncSerializer<tokio::io::Stdout>,
}

impl CsvWriter {
    pub fn new() -> CsvWriter {
        let writer = csv_async::AsyncSerializer::from_writer(tokio::io::stdout());
        CsvWriter { writer }
    }
}

#[async_trait]
impl ClientWriter for CsvWriter {
    // FIXME: Eliminate unwrap
    async fn append_client(&mut self, client: Client) -> Result<(), Error> {
        let client: HumanReadableClient = client.into();
        self.writer.serialize(client).await.unwrap();
        Ok(())
    }

    async fn close(self) -> Result<(), Error> {
        Ok(())
    }
}
