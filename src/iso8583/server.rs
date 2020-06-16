use std::net::{ToSocketAddrs, SocketAddr, TcpStream};
use std::io::{Read, Write};
use byteorder::{ByteOrder, WriteBytesExt};

use crate::iso8583::iso_spec::{IsoMsg, Spec};
use log;
use std::collections::HashMap;
use crate::iso8583::{bitmap, IsoError};
use std::thread::JoinHandle;
use crate::iso8583::msg_processor::MsgProcessor;

pub struct IsoServerError {
    msg: String
}



#[derive(Copy, Clone)]
pub struct IsoServer {
    sock_addr: SocketAddr,
    pub (crate) spec: &'static crate::iso8583::iso_spec::Spec,
    msg_processor: crate::iso8583::msg_processor::MsgProcessor,
}


impl IsoServer {
    pub fn start(&self) -> JoinHandle<()> {
        //let iso_serv=*self;
        let cp = *self;

        std::thread::spawn(move || {
            let listener = std::net::TcpListener::bind(cp.sock_addr).unwrap();

            for stream in listener.incoming() {
                let mut client = stream.unwrap();
                debug!("Accepted new connection .. {:?}", &client.peer_addr());
                new_client(cp, client);
            }
        })
    }
}


fn new_client(iso_server: IsoServer, stream_: TcpStream) {
    std::thread::spawn(move || {
        let mut buf: [u8; 512] = [0; 512];

        let mut stream = stream_;

        let mut reading_mli = true;
        let mut in_buf: Vec<u8> = Vec::with_capacity(512);
        let mut mli: u16 = 0;

        loop {
            //TODO:: MLI is assumed to be 2E for now

            match (&stream).read(&mut buf[..]) {
                Ok(n) => {
                    if n > 0 {
                        trace!("read {} from {}", hex::encode(&buf[0..n]), stream.peer_addr().unwrap().to_string());
                        in_buf.append(&mut buf[0..n].to_vec());


                        while in_buf.len() > 0 {
                            if reading_mli {
                                if in_buf.len() >= 2 {
                                    trace!("while reading mli .. {}", hex::encode(&in_buf.as_slice()));
                                    mli = byteorder::BigEndian::read_u16(&in_buf[0..2]);
                                    in_buf.drain(0..2 as usize).for_each(drop);
                                    reading_mli = false;
                                }
                            } else {
                                //reading data
                                if mli > 0 && in_buf.len() >= mli as usize {
                                    let data = &in_buf[0..mli as usize];
                                    debug!("received request len = {}  : data = {}", mli, hex::encode(data));

                                    match iso_server.msg_processor.process(&iso_server, data.to_vec()) {
                                        Ok(resp) => {
                                            debug!("iso_response \n raw:: {}, \n parsed:: \n {} \n ", hex::encode(&resp.0), resp.1);

                                            let mut resp_data = Vec::new();
                                            resp_data.write_u16::<byteorder::BigEndian>((&resp.0).len() as u16);
                                            resp_data.write_all(resp.0.as_slice());
                                            stream.write_all(resp_data.as_slice());
                                        }
                                        Err(e) => {
                                            error!("failed to handle incoming req - {}", e.msg)
                                        }
                                    }

                                    in_buf.drain(0..mli as usize).for_each(drop);
                                    mli = 0;
                                    reading_mli = true;
                                }
                            }
                        }
                    } else {
                        //socket may have been closed??
                        info!("client socket closed : {}", stream.peer_addr().unwrap().to_string());
                        return;
                    }
                }
                Err(e) => {
                    error!("client socket_err: {} {}", stream.peer_addr().unwrap().to_string(), e.to_string());
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



