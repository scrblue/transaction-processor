use super::*;

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
