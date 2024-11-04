# solana-validator-grpc
Rust implementation of solana transaction processing from a gRPC endpoint to a Validator over RPC

This repository is for validators who want:

* [to sell transaction bandwidth](https://github.com/noncepad/solpipe-market/tree/main/txproc)


## Transaction Processing

To use this library, first:
1. implement the `Processor` trait.
1. then, call the `run` function, which starts a gRPC server.
