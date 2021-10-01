use super::*;
use db_layer::DbLayer;

/// Process a single transaction
pub async fn process_transaction(
    db: &mut impl db_layer::DbLayer,
    transaction: Transaction,
) -> Result<(), Error> {
    // If there is already a client with that ID, modify it
    let mut client = if let Some(client) = db.get_client(transaction.client).await? {
        client
    } else {
        Client {
            client: transaction.client,
            available: 0,
            held: 0,
            total: 0,
            locked: false,
        }
    };

    match transaction.ty {
        TransactionType::Deposit => {
            process_deposit(&mut client, transaction)?;
            db.write_transaction(transaction).await?;
        }

        TransactionType::Withdrawal => {
            process_withdrawal(&mut client, transaction)?;
            db.write_transaction(transaction).await?;
        }

        TransactionType::Dispute => {
            let referenced_transaction = db.get_transaction(transaction.tx).await?;
            process_dispute(&mut client, referenced_transaction)?;
        }

        TransactionType::Resolve => {
            let referenced_transaction = db.get_transaction(transaction.tx).await?;
            process_resolve(&mut client, referenced_transaction)?;
        }

        TransactionType::Chargeback => {
            let referenced_transaction = db.get_transaction(transaction.tx).await?;
            process_chargeback(&mut client, referenced_transaction)?;
        }
    }

    db.write_client(client).await?;
    Ok(())
}

fn process_deposit(client: &mut Client, transaction: Transaction) -> Result<(), Error> {
    if let Some(amount) = transaction.amount {
        client.available += amount;
        client.total += amount;
        Ok(())
    } else {
        return Err(Error::NoAmount);
    }
}

fn process_withdrawal(client: &mut Client, transaction: Transaction) -> Result<(), Error> {
    if let Some(amount) = transaction.amount {
        if client.available - amount >= 0 {
            client.available -= amount;
            client.total -= amount;
            Ok(())
        } else {
            Err(Error::InsufficientFunds)
        }
    } else {
        Err(Error::NoAmount)
    }
}

fn process_dispute(
    client: &mut Client,
    referenced_transaction: Option<Transaction>,
) -> Result<(), Error> {
    if let Some(referenced_transaction) = referenced_transaction {
        if referenced_transaction.client == client.client {
            if let Some(amount) = referenced_transaction.amount {
                client.available -= amount;
                client.held += amount;
                Ok(())
            } else {
                Err(Error::NoAmount)
            }
        } else {
            Err(Error::ReferencesWrongClient)
        }
    } else {
        Err(Error::ReferenceDoesNotExist)
    }
}

fn process_resolve(
    client: &mut Client,
    referenced_transaction: Option<Transaction>,
) -> Result<(), Error> {
    if let Some(referenced_transaction) = referenced_transaction {
        if referenced_transaction.client == client.client {
            if let Some(amount) = referenced_transaction.amount {
                client.available += amount;
                client.held -= amount;
                Ok(())
            } else {
                Err(Error::NoAmount)
            }
        } else {
            Err(Error::ReferencesWrongClient)
        }
    } else {
        Err(Error::ReferenceDoesNotExist)
    }
}

fn process_chargeback(
    client: &mut Client,
    referenced_transaction: Option<Transaction>,
) -> Result<(), Error> {
    if let Some(referenced_transaction) = referenced_transaction {
        if referenced_transaction.client == client.client {
            if let Some(amount) = referenced_transaction.amount {
                client.held -= amount;
                client.total -= amount;
                client.locked = true;
                Ok(())
            } else {
                Err(Error::NoAmount)
            }
        } else {
            Err(Error::ReferencesWrongClient)
        }
    } else {
        Err(Error::ReferenceDoesNotExist)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn basic() {
        let inputs = vec![
            Transaction {
                ty: TransactionType::Deposit,
                client: 1,
                tx: 1,
                amount: Some(10000),
            },
            Transaction {
                ty: TransactionType::Deposit,
                client: 2,
                tx: 2,
                amount: Some(20000),
            },
            Transaction {
                ty: TransactionType::Deposit,
                client: 1,
                tx: 3,
                amount: Some(20000),
            },
            Transaction {
                ty: TransactionType::Withdrawal,
                client: 1,
                tx: 4,
                amount: Some(15000),
            },
            Transaction {
                ty: TransactionType::Withdrawal,
                client: 2,
                tx: 5,
                amount: Some(30000),
            },
        ];

        let mut db_layer = db_layer::hashmap::HashMapDb::new(2);

        for input in inputs {
            let _ = process_transaction(&mut db_layer, input).await;
        }

        let client_1 = Client {
            client: 1,
            available: 15000,
            held: 0,
            total: 15000,
            locked: false,
        };

        let client_2 = Client {
            client: 2,
            available: 20000,
            held: 0,
            total: 20000,
            locked: false,
        };

        let actual_out = db_layer
            .stream_clients()
            .await
            .recv()
            .await
            .unwrap()
            .unwrap();
        println!("{:?}", actual_out);

        assert!(actual_out == client_1 || actual_out == client_2);
    }
}