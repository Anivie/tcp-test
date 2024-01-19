## An Experiment about Computer Networks

## A computer network experiment

Using raw socket, I used (unsafe) rust ffi to complete this function, the process was very tortuous (  
Currently considered completed, but now it can handshake normally and receive data from server  
The target connection is tokio's echo demo.

This project refers to [this experiment](https://github.com/MaxXor/raw-sockets-example/blob/master/rawsockets.c), and I manually implemented and simplified it using Rust.
You can run this program directly in Linux using `cargo run`.

Test environment:
- Ubuntu (WSL)
- Rust Nightly