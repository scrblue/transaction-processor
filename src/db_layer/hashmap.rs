use async_trait::async_trait;
use std::collections::HashMap;

use super::*;

pub struct HashMapDb {
    transactions_map: HashMap<u32, Transaction>,
    clients_map: HashMap<u16, Client>,

    clients_sender: mpsc::Sender<Result<Client, Error>>,
    clients_receiver: Option<mpsc::Receiver<Result<Client, Error>>>,
}

impl HashMapDb {
    pub fn new(buffer_size: usize) -> HashMapDb {
        let transactions_map = HashMap::new();
        let clients_map = HashMap::new();
        let (clients_sender, clients_receiver) = mpsc::channel(buffer_size);
        let clients_receiver = Some(clients_receiver);

        HashMapDb {
            transactions_map,
            clients_map,
            clients_sender,
            clients_receiver,
        }
    }

    async fn stream(mut self) {
        for (_id, client) in self.clients_map.drain() {
            self.clients_sender.send(Ok(client)).await.unwrap();
        }
    }
}

#[async_trait]
impl DbLayer for HashMapDb {
    async fn write_transaction(&mut self, transaction: Transaction) -> Result<(), Error> {
        self.transactions_map.insert(transaction.tx, transaction);
        Ok(())
    }
    async fn get_transaction(&mut self, transaction_id: u32) -> Result<Option<Transaction>, Error> {
        Ok(self
            .transactions_map
            .get(&transaction_id)
            .map(|ref_val| *ref_val))
    }

    async fn write_client(&mut self, client: Client) -> Result<(), Error> {
        self.clients_map.insert(client.client, client);
        Ok(())
    }
    async fn get_client(&mut self, client_id: u16) -> Result<Option<Client>, Error> {
        Ok(self.clients_map.get(&client_id).map(|ref_val| *ref_val))
    }

    async fn stream_clients(mut self) -> mpsc::Receiver<Result<Client, Error>> {
        let receiver = self.clients_receiver.take().unwrap();

        tokio::spawn(self.stream());

        receiver
    }
}
