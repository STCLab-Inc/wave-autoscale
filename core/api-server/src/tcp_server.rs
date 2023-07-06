use log::debug;
use tokio::{io::AsyncBufReadExt, net::TcpListener, spawn};

pub async fn run_tcp_server(address: &str, port: u16) {
    let address = format!("{}:{}", address, port);
    let listener = TcpListener::bind(address.clone()).await;

    if listener.is_err() {
        panic!("Failed to bind TCP Server to address: {}", address);
    }
    let listener = listener.unwrap();
    debug!("Listening on: {}", address);

    spawn(async move {
        loop {
            let result = listener.accept().await;
            if result.is_err() {
                panic!("Failed to accept TCP connection: {:?}", result);
            }
            let (mut socket, _) = result.unwrap();

            spawn(async move {
                let (reader, _) = socket.split();
                let mut reader = tokio::io::BufReader::new(reader);
                let mut msg: String = String::new();
                loop {
                    match reader.read_line(&mut msg).await {
                        Ok(_) => {
                            if msg.is_empty() {
                                break;
                            }
                            debug!("Received message: {}", msg);
                            msg.clear();
                        }
                        Err(e) => {
                            println!("Failed to read from socket; err = {:?}", e);
                            return;
                        }
                    }
                }
            });
        }
    });
}
