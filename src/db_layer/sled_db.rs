use async_trait::async_trait;
use sled::Db;
use std::path::Path;

use super::*;

pub struct SledDb {
    db: Db,
    clients_sender: mpsc::Sender<Result<Client, Error>>,
    clients_receiver: Option<mpsc::Receiver<Result<Client, Error>>>,
}

impl SledDb {
    pub fn new(path: impl AsRef<Path>, buffer_size: usize) -> Result<SledDb, sled::Error> {
        let db = sled::open(&path)?;

        // Create the keyspaces
        let _ = db.open_tree(b"transactions")?;
        let _ = db.open_tree(b"clients")?;

        let (clients_sender, clients_receiver) = mpsc::channel(buffer_size);
        let clients_receiver = Some(clients_receiver);

        db.flush()?;

        Ok(SledDb {
            db,
            clients_sender,
            clients_receiver,
        })
    }

    // This code is complicated because the `ColumnFamily`s
    async fn stream(self) {
        // FIXME: Eliminate unwraps
        let tree = self.db.open_tree(b"clients").unwrap();
        for result in tree.iter() {
            match result {
                Ok(client) => match bincode::deserialize::<Client>(&client.1) {
                    Ok(client) => {
                        if self.clients_sender.send(Ok(client)).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        if self
                            .clients_sender
                            .send(Err(Error::DbLayer(format!("Error deserializing: {}", e))))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                },

                Err(e) => {
                    if self
                        .clients_sender
                        .send(Err(Error::DbLayer(format!("{}", e))))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
            }
        }
    }
}

impl From<sled::Error> for Error {
    fn from(e: sled::Error) -> Error {
        Error::DbLayer(format!("{}", e))
    }
}

#[async_trait]
impl DbLayer for SledDb {
    async fn write_transaction(&mut self, transaction: Transaction) -> Result<(), Error> {
        let tree = self.db.open_tree("transactions")?;
        tree.insert(
            transaction.tx.to_le_bytes(),
            bincode::serialize(&transaction).unwrap(),
        )?;
        self.db.flush()?;
        Ok(())
    }
    async fn get_transaction(&mut self, transaction_id: u32) -> Result<Option<Transaction>, Error> {
        let tree = self.db.open_tree("transactions")?;
        if let Some(Ok(transaction)) = tree
            .get(transaction_id.to_le_bytes())?
            .map(|bytes| bincode::deserialize(&bytes))
        {
            Ok(Some(transaction))
        } else {
            Ok(None)
        }
    }

    async fn write_client(&mut self, client: Client) -> Result<(), Error> {
        let tree = self.db.open_tree("clients")?;
        tree.insert(
            client.client.to_le_bytes(),
            bincode::serialize(&client).unwrap(),
        )?;
        self.db.flush()?;
        Ok(())
    }
    async fn get_client(&mut self, client_id: u16) -> Result<Option<Client>, Error> {
        let tree = self.db.open_tree("clients")?;
        if let Some(Ok(client)) = tree
            .get(client_id.to_le_bytes())?
            .map(|bytes| bincode::deserialize(&bytes))
        {
            Ok(Some(client))
        } else {
            Ok(None)
        }
    }

    async fn stream_clients(mut self) -> mpsc::Receiver<Result<Client, Error>> {
        let receiver = self.clients_receiver.take().unwrap();

        tokio::spawn(self.stream());

        receiver
    }
}
