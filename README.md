# Transaction Processing Tool
A tool that reads transactions from a CSV file and outputs account states as CSV text written to
stdout. A `default.nix` file is included for the convenience of NixOS users like myself -- if you
do not know what that is, it is safe to ignore the file.

## A very important note
By default this crate has the `no_persist` feature enabled such as to make tests run more easily.
If you expect user and transaction data to persist between runs of the application, as would be
expected from a banking application, please disable this feature.

## Dependencies
* async-traits -- For ease of maintenance the reading, processing, and writing of data has been
  abstracted with traits
* serde -- because who in their right mind does serialization and deserialization in Rust without
  Serde
* csv-async -- for ease of reading and writing CSV files
* rocksdb -- for persistence of user and transaction data both during the interpretation of a single
  file and between interpretations of multiple files
* tokio and tokio-stream -- for streaming of CSV data instead of loading the entire file at once

## Dev dependencies
* tempfile -- for creating directories and files for testing

## Design decisions

### On RocksDB
RocksDB is a well-known and relatively efficient key-value store database. The motive behind using
it is two-fold. To work effectively at transaction processing, user and transaction data needs to be
persistent. If a user's balance resets after every batch of transactions is processed, you aren't
being a very good bank. It also serves to minimize the space complexity of the solution in RAM. As
user IDs are `u16` values, a simple `HashMap` could take well over a gibibyte of RAM. While this
wouldn't be a problem on a most servers, a simple cloud VPS could easily be overloaded with so much
memory use.

Ideally the exact nature of the algorithm behind this decision would be configurable, but as the
command line arguments are limited to the path of a single CSV file and introducing a configuration
file seems overkill, I am assuming sane defaults for a low-end server with the assumption that the
`const` variables defined could easily be made configurable in a later iteration of the tool.

NOTE: If `no_persist` is enabled, a `HashMap` will be used instead.

### On Tokio and Tokio-Util
These crates offer asynchronous reading and writing to files as well as allowing very easy streaming
of data. While there are likely better solutions to this problem, Tokio and its related crates are
well-known both to myself and to the Rust community as a whole ensuring that the solution is
well-maintainable.

### On a `TransactionReader` and `DbLayer` trait
I decided to implement transaction processing for any valid `TransactionReader` such as to allow
the eventual expansion to more complex functionality such as processing requests from several
network streams. The transaction and client persistence is done through an implementor of the
`DbLayer` trait. The `DbLayer` trait is implemented for a RocksDB instance and for a struct
containing two `HashMap`s by default. The one used when running the project depends on the
`no_persist` feature.

### On fixed point numbers
Fixed point numbers are used over floating point numbers such as to prevent rounding errors. `i64`s
are used for monetary amounts which provide what I believe to be a sufficient range of values even
even given the four decimal places.


## On TODOs in the code
Upon upload to GitHub, this tool will be complete as per the specification given. Anything marked
as a TODO in code will be an area that could be expanded upon in future iterations.
