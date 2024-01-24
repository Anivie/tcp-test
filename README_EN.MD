## An Experiment about Computer Networks

## A computer network experiment

Using raw sockets, I used (unsafe) Rust FFI to accomplish this functionality. The process was quite convoluted(  
It can now perform a handshake and receive information. The target connection is the Tokio echo demo.

This project was inspired by [this experiment](https://github.com/MaxXor/raw-sockets-example/blob/master/rawsockets.c). I manually implemented it in Rust and added some simple wrappers. You can run this program directly using `cargo run` on Linux.

The test connection target is [TcpListener](https://docs.rs/tokio/1.35.1/tokio/net/struct.TcpListener.html), which I encapsulated into a standalone [server](https://github.com/Anivie/tcp-test-server).

Test environment:
- Ubuntu (WSL)
- Rust Nightly