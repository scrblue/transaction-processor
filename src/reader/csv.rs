use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::{fs::File, sync::mpsc};
use tokio_stream::StreamExt;

use crate::{Transaction, TransactionType};
use super::*;

/// A Transaction to using a floating point monetary value
#[derive(Serialize, Deserialize)]
struct CsvTransaction {
    /// Types including deposits, withdrawals, disputes, resolutions of disputes, and chargebacks
    ty: TransactionType,

    /// A unique client ID for which all transactions are tied to
    client: u16,

    /// A unique transaction ID given to deposits or withdrawals. Disputes, resolutions, and
    /// chargebacks reference transaction IDs of deposits
    tx: u32,

    /// The amount of the deposit or withdrawal. This field will be None for any other TransactionType
    amount: Option<f64>,
}

impl Into<Transaction> for CsvTransaction {
    fn into(self) -> Transaction {
        Transaction {
            ty: self.ty,
            client: self.client,
            tx: self.tx,
            amount: self.amount.map(|fp_num| {
                fp_num * 1000 as f64;
                fp_num.round();

                // FIXME: Casting like this is unsafe without more checks
                fp_num as i64
            }),
        }
    }
}

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
    pub async fn new(file: &PathBuf, buffer_size: usize) -> std::io::Result<Self> {
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
        let mut reader = csv_async::AsyncDeserializer::from_reader(self.file);
        let reader = reader.deserialize();

        let mut reader = reader.filter_map(|result: Result<CsvTransaction, _>| -> Option<Transaction> {
            // TODO: Return an error instaed of skipping over lines that don't deserialize properly
            if let Ok(result) = result {
                let result= result.into();
                Some(result)
            } else {
                None
            }
        });

        // The method then populates the buffer of the channel until it is full, waiting for a spot
        // to become available before continuing ensuring that there are never more than the
        // configured amount of transactions in the queue
        while let Some(transaction) = reader.next().await {
            // Send the transaction and break the loop if the send is an Err as that means the
            // receiver has been closed
            if let Err(_) = self.sender.send(transaction).await {
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
mod tests {}
