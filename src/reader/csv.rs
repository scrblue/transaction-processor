use std::path::Path;
use tokio::{fs::File, sync::mpsc};
use tokio_stream::StreamExt;

use super::*;
use crate::{HumanReadableTransaction, Transaction};

/// An Implementor of the TransactionReader trait which reads CSV values from a given file
pub struct CsvReader {
    /// The file to read from
    file: File,

    /// The [`mpsc::Sender`] through which read transactions will be sent
    sender: mpsc::Sender<Transaction>,

    /// The [`mpsc::Receiver`] from which [`Transaction`]s will be received. Will be None after the
    /// [`TransactionReader::start`] method is called
    receiver: Option<mpsc::Receiver<Transaction>>,
}

impl CsvReader {
    pub async fn new(file: impl AsRef<Path>, buffer_size: usize) -> std::io::Result<Self> {
        let file = File::open(file).await?;
        let (sender, receiver) = mpsc::channel(buffer_size);
        let receiver = Some(receiver);

        Ok(Self {
            file,
            sender,
            receiver,
        })
    }

    async fn read(self) {
        let mut reader = csv_async::AsyncReaderBuilder::new()
            .trim(csv_async::Trim::All)
            .flexible(true)
            .create_deserializer(self.file);
        let reader = reader.deserialize();

        let mut reader = reader.filter_map(
            |result: Result<HumanReadableTransaction, _>| -> Option<Transaction> {
                // TODO: Return an error instaed of skipping over lines that don't deserialize properly
                if let Ok(result) = result {
                    let result = result.into();
                    Some(result)
                } else {
                    None
                }
            },
        );

        // The method then populates the buffer of the channel until it is full, waiting for a spot
        // to become available before continuing ensuring that there are never more than the
        // configured amount of transactions in the queue
        while let Some(transaction) = reader.next().await {
            // Send the transaction and break the loop if the send is an Err as that means the
            // receiver has been closed
            if self.sender.send(transaction).await.is_err() {
                break;
            }
        }
    }
}

impl TransactionReader for CsvReader {
    fn start(mut self) -> mpsc::Receiver<Transaction> {
        let receiver = self.receiver.take();

        tokio::spawn(self.read());

        // The `unwrap` is definitely code smell, but the receiver will always be Some upon creation
        // and the `new` and `start` methods are the only public methods, so if receiver is None,
        // something has gone seriously wrong
        receiver.unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{convert::TryInto, path::PathBuf};
    use tempfile::TempDir;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn basic() {
        let dir = TempDir::new_in("./").unwrap();
        let mut path: PathBuf = dir.path().into();
        path.push("test.csv");

        let file_contents = r#"type, client, tx, amount
		deposit, 1, 1, 1.0
		deposit, 2, 2, 2.0
		deposit, 1, 3, 2.0
		withdrawal, 1, 4, 1.5
		withdrawal, 2, 5, 3.0
		"#;

        let expected = vec![
            Transaction {
                ty: TransactionType::Deposit,
                client: 1,
                tx: 1,
                amount: Some(10000),
                disputed: false,
            },
            Transaction {
                ty: TransactionType::Deposit,
                client: 2,
                tx: 2,
                amount: Some(20000),
                disputed: false,
            },
            Transaction {
                ty: TransactionType::Deposit,
                client: 1,
                tx: 3,
                amount: Some(20000),
                disputed: false,
            },
            Transaction {
                ty: TransactionType::Withdrawal,
                client: 1,
                tx: 4,
                amount: Some(15000),
                disputed: false,
            },
            Transaction {
                ty: TransactionType::Withdrawal,
                client: 2,
                tx: 5,
                amount: Some(30000),
                disputed: false,
            },
        ];

        // Ensure the File object has been dropped before attempting to read from it
        {
            let mut file = File::create(&path).await.unwrap();
            file.write_all(file_contents.as_bytes()).await.unwrap();
        }

        let reader = CsvReader::new(&path, 2).await.unwrap();
        let mut receiver = reader.start();

        let mut actual = Vec::new();
        while let Some(transaction) = receiver.recv().await {
            actual.push(transaction);
        }

        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn skip_malformed_data() {
        let dir = TempDir::new_in("./").unwrap();
        let mut path: PathBuf = dir.path().into();
        path.push("test.csv");

        let file_contents = r#"type, client, tx, amount
		deposit, 1, 1, 1.0
		deposit, 2, 2, 2.0
		deposit, 1, 3, 2.0
		deposit, invalid, 2, 2.0
		withdrawal, 1, 4, 1.5
		withdrawal, 2, 5, 3.0
		"#;

        let expected = vec![
            Transaction {
                ty: TransactionType::Deposit,
                client: 1,
                tx: 1,
                amount: Some(10000),
                disputed: false,
            },
            Transaction {
                ty: TransactionType::Deposit,
                client: 2,
                tx: 2,
                amount: Some(20000),
                disputed: false,
            },
            Transaction {
                ty: TransactionType::Deposit,
                client: 1,
                tx: 3,
                amount: Some(20000),
                disputed: false,
            },
            Transaction {
                ty: TransactionType::Withdrawal,
                client: 1,
                tx: 4,
                amount: Some(15000),
                disputed: false,
            },
            Transaction {
                ty: TransactionType::Withdrawal,
                client: 2,
                tx: 5,
                amount: Some(30000),
                disputed: false,
            },
        ];

        {
            let mut file = File::create(&path).await.unwrap();
            file.write_all(file_contents.as_bytes()).await.unwrap();
        }

        let reader = CsvReader::new(&path, 2).await.unwrap();
        let mut receiver = reader.start();

        let mut actual = Vec::new();
        while let Some(transaction) = receiver.recv().await {
            actual.push(transaction);
        }

        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn dispute_resolve_and_chargeback() {
        let dir = TempDir::new_in("./").unwrap();
        let mut path: PathBuf = dir.path().into();
        path.push("test.csv");

        let file_contents = r#"type, client, tx, amount
        deposit, 1, 1, 1.0
		dispute, 1, 1
		resolve, 1, 1
		chargeback, 1, 1
		"#;

        let expected = vec![
            Transaction {
                ty: TransactionType::Deposit,
                client: 1,
                tx: 1,
                amount: Some(10000),
                disputed: false,
            },
            Transaction {
                ty: TransactionType::Dispute,
                client: 1,
                tx: 1,
                amount: None,
                disputed: false,
            },
            Transaction {
                ty: TransactionType::Resolve,
                client: 1,
                tx: 1,
                amount: None,
                disputed: false,
            },
            Transaction {
                ty: TransactionType::Chargeback,
                client: 1,
                tx: 1,
                amount: None,
                disputed: false,
            },
        ];

        {
            let mut file = File::create(&path).await.unwrap();
            file.write_all(file_contents.as_bytes()).await.unwrap();
        }

        let reader = CsvReader::new(&path, 2).await.unwrap();
        let mut receiver = reader.start();

        let mut actual = Vec::new();
        while let Some(transaction) = receiver.recv().await {
            actual.push(transaction);
        }

        assert_eq!(expected, actual);
    }
}
