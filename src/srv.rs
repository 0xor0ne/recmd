use std::io::Read;
use std::net::{IpAddr, Ipv4Addr, Shutdown, SocketAddr, TcpListener, TcpStream};
use std::thread;

#[derive(Debug)]
pub struct Srv {
    port: u16,
}

impl Srv {
    pub fn new(port: u16) -> Self {
        Srv { port }
    }

    pub fn run(&mut self) {
        self.recv_simple();
    }

    fn handle_connection(mut stream: TcpStream) {
        let mut data = [0 as u8; 50]; // using 50 byte buffer
        match stream.read(&mut data) {
            Ok(_size) => {
                // echo everything!
                println!("Received: {}", std::str::from_utf8(&data).unwrap());
            }
            Err(_) => {
                println!(
                    "An error occurred, terminating connection with {}",
                    stream.peer_addr().unwrap()
                );
                stream.shutdown(Shutdown::Both).unwrap();
            }
        } {}
    }

    fn recv_simple(&self) {
        let listener = TcpListener::bind(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            self.port,
        ))
        .unwrap();

        println!("Server listening on port {}", self.port);
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("New connection from {}", stream.peer_addr().unwrap());
                    thread::spawn(move || {
                        Srv::handle_connection(stream);
                    });
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            };
        }

        drop(listener);
    }
}
