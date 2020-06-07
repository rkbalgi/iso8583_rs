use std::net::{ToSocketAddrs, SocketAddr, TcpStream};
use std::io::Read;
use byteorder::ByteOrder;
use std::sync::Arc;
use crate::iso8583::iso_spec::{IsoMsg, Spec};

pub struct IsoServerError {
    msg: String
}

#[derive(Copy, Clone)]
pub struct MsgProcessor {}

impl MsgProcessor {
    pub fn process(&self, iso_server: &IsoServer, msg: Vec<u8>) -> Result<[u8; 8], IsoServerError> {
        match iso_server.spec.parse(msg) {
            Ok(iso_msg) => {
                println!("parsed incoming request - message type = {} successfully, \n {} \n ", "", iso_msg);

                Ok([0; 8])
            }
            Err(e) => {
                Err(IsoServerError { msg: e.msg })
            }
        }
    }
}

#[derive(Copy, Clone)]
pub struct IsoServer {
    sock_addr: SocketAddr,
    spec: &'static crate::iso8583::iso_spec::Spec,
    msg_processor: MsgProcessor,
}


impl IsoServer {
    pub fn start(&self) {
        //let iso_serv=*self;
        let cp=*self;

        std::thread::spawn(move || {
            let listener = std::net::TcpListener::bind(cp.sock_addr).unwrap();

            for stream in listener.incoming() {
                let client = stream.unwrap();
                println!("Accepted new connection .. {:?}", &client.peer_addr());
                new_client(cp, client);
            }
        });
    }
}


fn new_client(iso_server: IsoServer, stream: TcpStream) {
    std::thread::spawn(move || {
        let mut buf: [u8; 512] = [0; 512];

        let mut reading_mli = true;
        let mut in_buf: Vec<u8> = Vec::with_capacity(512);
        let mut mli: u16 = 0;

        loop {
            //TODO:: MLI is assumed to be 2E for now

            match (&stream).read(&mut buf[..]) {
                Ok(n) => {
                    if n > 0 {
                        println!("read {} from {}", hex::encode(&buf[0..n]), stream.peer_addr().unwrap().to_string());
                        in_buf.append(&mut buf[0..n].to_vec());


                        while in_buf.len() > 0 {
                            if reading_mli {
                                if in_buf.len() >= 2 {
                                    println!("while reading mli .. {}", hex::encode(&in_buf.as_slice()));
                                    mli = byteorder::BigEndian::read_u16(&in_buf[0..2]);
                                    in_buf.drain(0..2 as usize).collect::<Vec<u8>>();
                                    reading_mli = false;
                                }
                            } else {
                                //reading data
                                if mli > 0 && in_buf.len() >= mli as usize {
                                    let data = &in_buf[0..mli as usize];
                                    println!("received request len = {}  : data = {}", mli, hex::encode(data));
                                    iso_server.msg_processor.process(&iso_server, data.to_vec());
                                    //TODO:: get the response and respond
                                    in_buf.drain(0..mli as usize).collect::<Vec<u8>>();
                                    mli = 0;
                                    reading_mli = true;
                                }
                            }
                        }
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

pub fn new(host_port: String, spec: &'static Spec) -> Result<IsoServer, IsoServerError> {
    match host_port.to_socket_addrs() {
        Ok(mut i) => {
            match i.next() {
                Some(ip_addr) => {
                    Ok(IsoServer { sock_addr: ip_addr, spec, msg_processor: MsgProcessor {} })
                }
                None => {
                    Err(IsoServerError { msg: format!("invalid host_port: {} : unresolvable?", &host_port) })
                }
            }
        }
        Err(e) => Err(IsoServerError { msg: format!("invalid host_port: {}: cause: {}", &host_port, e.to_string()) })
    }
}



