//! This module contains implementation of a ISO TCP client

use crate::iso8583::iso_spec::{Spec, IsoMsg};
use crate::iso8583::IsoError;
use std::net::TcpStream;
use crate::iso8583::mli::{MLI, MLIType, MLI2E, MLI2I, MLI4E, MLI4I};
use std::io::{Write, Read};
use crate::iso8583::server::get_hexdump;
use std::borrow::BorrowMut;


/// This struct represents a ISO8583 TCP client
pub struct ISOTcpClient {
    server_addr: String,
    mli: Box<dyn MLI>,
    spec: &'static Spec,
    _tcp_stream: Option<TcpStream>,
}


impl ISOTcpClient {
    /// Creates a new ISOTcpClient
    pub fn new(server_addr: &str, spec: &'static Spec, mli_type: MLIType) -> ISOTcpClient {
        let mli: Box<dyn MLI>;

        match mli_type {
            MLIType::MLI2E => mli = Box::new(MLI2E {}),
            MLIType::MLI2I => mli = Box::new(MLI2I {}),
            MLIType::MLI4E => mli = Box::new(MLI4E {}),
            MLIType::MLI4I => mli = Box::new(MLI4I {})
        }

        ISOTcpClient {
            server_addr: server_addr.to_string(),
            spec,
            mli,
            _tcp_stream: None,
        }
    }

    /// Sends a ISO message to the server and returns the response from server on success
    /// or a IsoError on failure
    pub fn send(&mut self, iso_msg: &IsoMsg) -> Result<IsoMsg, IsoError> {
        match iso_msg.assemble() {
            Ok(data) => {
                let mut buf = self.mli.create(&data.len()).unwrap();
                buf.extend(data);
                self.send_recv(&buf)
            }
            Err(e) => {
                Err(IsoError { msg: format!("Failed to assemble request message: {}", e.msg) })
            }
        }
    }

    fn send_recv(&mut self, raw_msg: &Vec<u8>) -> Result<IsoMsg, IsoError> {
        println!("raw iso msg = {}", hex::encode(raw_msg.as_slice()));

        if self._tcp_stream.is_none() {
            self._tcp_stream = match TcpStream::connect(&self.server_addr) {
                Err(e) => return Err(IsoError { msg: e.to_string() }),
                Ok(c) => {
                    println!("connected to server @ {:?}", c.local_addr());
                    Option::Some(c)
                }
            }
        }

        let mut client = self._tcp_stream.as_mut().unwrap();

        client.write_all(raw_msg.as_slice()).unwrap();
        client.flush();

        // read the response
        let len: u32;
        match self.mli.parse(client) {
            Ok(n) => len = n,
            Err(e) => return Err(e)
        };

        let mut out_buf = vec![0; len as usize];

        match client.read_exact(&mut out_buf[..]) {
            Ok(()) => {
                println!("received response: with  {} bytes. \n {}\n", len, get_hexdump(&out_buf));
                match self.spec.parse(&mut out_buf) {
                    Ok(resp_iso_msg) => {
                        Ok(resp_iso_msg)
                    }
                    Err(e) => {
                        Err(IsoError { msg: e.msg })
                    }
                }
            }
            Err(e) => {
                Err(IsoError { msg: e.to_string() })
            }
        }
    }
}