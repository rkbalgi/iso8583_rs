use std::net::{Ipv4Addr, ToSocketAddrs, SocketAddr, TcpStream};
use std::error::Error;
use std::io::Read;


pub struct IsoServerError {
    msg: String
}


pub struct IsoServer {
    sock_addr: SocketAddr
}

impl IsoServer {
    pub fn start(&self) {
        let addr = self.sock_addr.clone();

        std::thread::spawn(move || {
            let listener = std::net::TcpListener::bind(addr).unwrap();

            for stream in listener.incoming() {
                let client = stream.unwrap();
                println!("Accepted new connection .. {:?}", &client.local_addr());
                new_client(client);
            }
        });
    }
}

fn new_client(stream: TcpStream) {
    std::thread::spawn(move || {
        let mut buf: [u8; 1024] = [0; 1024];
        loop {
            match (&stream).read(&mut buf[..]) {
                Ok(n) => {
                    if n > 0 {
                        println!("read {} from {}", hex::encode(&buf[0..n]), stream.local_addr().unwrap().to_string());
                    } else {
                        //socket may have been closed??
                        println!("client socket closed : {}", stream.peer_addr().unwrap().to_string());
                        return;
                    }
                }
                Err(e) => {
                    println!("client socket_err: {}", stream.peer_addr().unwrap().to_string());
                    return;
                }
            }
        }
    });
}

pub fn new(host_port: String) -> Result<IsoServer, IsoServerError> {
    match host_port.to_socket_addrs() {
        Ok(mut i) => {
            match i.next() {
                Some(ip_addr) => {
                    Ok(IsoServer { sock_addr: ip_addr })
                }
                None => {
                    Err(IsoServerError { msg: format!("invalid host_port: {} : unresolvable?", &host_port) })
                }
            }
        }
        Err(e) => Err(IsoServerError { msg: format!("invalid host_port: {}: cause: {}", &host_port, e.to_string()) })
    }
}



