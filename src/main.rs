use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt;

mod db_layer;
mod fixed_point_util;
mod model;
mod reader;
mod transaction_processing;
mod writer;

use db_layer::DbLayer;
use model::*;
use reader::TransactionReader;
use writer::ClientWriter;

/// The number of [`Transaction`]s to allow in the [`tokio::sync::mpsc::Receiver`]'s queue. Each
/// [`Transaction`] will be roughly 120 bytes (plus padding) and the overhead of the mpsc channel.
// TODO: Make this number configurable
const READER_BUFFER: usize = 1024;

/// The number of [`Client`]s to allow in the [`tokio::sync::mpsc::Receiver`]'s queue. Each
/// [`Client`] will be roughly 200 bytes (plus padding) and the overhead of the mpsc channel.
// TODO: Make this number configurable
const DB_BUFFER: usize = 1024;

/// The path of the RocksDB key value store
// TODO: Make this path configurable
const DB_PATH: &str = "./database";

// FIXME: Eliminate unwraps
#[tokio::main]
async fn main() {
    let mut args = std::env::args();
    let _ = args.next();

    // Read from a CSV file with the path given in the first argument
    let reader = reader::csv::CsvReader::new(
        args.next()
            .expect("Must have one argument with the path of a CSV file"),
        READER_BUFFER,
    )
    .await
    .unwrap();
    let mut receiver = reader.start();

    // Using a couple of `HashMaps` or a `sled::Db`, hold transaction and client information
    #[cfg(feature = "no_persist")]
    let mut db_layer = db_layer::hashmap::HashMapDb::new(DB_BUFFER);
    #[cfg(not(feature = "no_persist"))]
    let mut db_layer = db_layer::sled_db::SledDb::new(DB_PATH, DB_BUFFER).unwrap();

    // Process each transaction from the reader's receiver
    while let Some(input) = receiver.recv().await {
        // Fail silently here
        let _ = transaction_processing::process_transaction(&mut db_layer, input).await;
    }

    // When all transactions in the batch have been processed, write the final state of each Client
    // to stdout
    let mut receiver = db_layer.stream_clients().await;
    let mut writer = writer::csv::CsvWriter::new();
    while let Some(output) = receiver.recv().await {
        writer.append_client(output.unwrap()).await.unwrap();
    }

    // And you'd close the ClientWriter here were it any implementation where thismethod does
    // something
    // writer.close().await.unwrap();
}
