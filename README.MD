### English in [here](https://github.com/Anivie/tcp-test/blob/master/README_EN.MD)  

## 一次关于计算机网络的实验

使用raw socket，我使用(unsafe) rust ffi来完成这一功能，过程非常曲折(
目前已经完成，可以正常握手并接收信息，连接的目标是tokio的echo demo

本项目参考于[这个实验](https://github.com/MaxXor/raw-sockets-example/blob/master/rawsockets.c)，我使用rust手动实现并进行了简单的封装
你可以在Linux中直接使用`cargo run`来运行这段程序  
测试的连接目标为[TcpListener](https://docs.rs/tokio/1.35.1/tokio/net/struct.TcpListener.html)，我将其封装为独立的[服务器](https://github.com/Anivie/tcp-test-server)

测试环境:
 - Ubuntu(WSL)
 - Rust Nightly