# ZKP RDF Query PoC

This is a proof of concept that ZKP can be done over arbitrary SPARQL Queries

## What does this package do?

The prover can take a SPARQL Query and dataset as input. It outputs:
 - A hash of the query
 - A hash of the results set
 - A hash of the input data

## To run this execute the command

Get risczero set up on your machine using [this documentation](https://dev.risczero.com/api/getting-started) and then run the following command in the root directory.

```bash
cargo run --release
```
