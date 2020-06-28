//! This module contains the implementation of a ISO server (TCP)
use std::io::{BufReader, Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::thread::JoinHandle;

use hexdump::hexdump_iter;

use crate::iso8583::IsoError;
use crate::iso8583::iso_spec::{IsoMsg, Spec};
use crate::iso8583::mli::{MLI, MLI2E, MLI2I, MLI4E, MLI4I, MLIType};

/// This struct represents an error associated with server errors
pub struct IsoServerError {
    pub msg: String
}

/// This struct represents a IsoServer
pub struct ISOServer {
    /// The listen address for this server
    sock_addr: Vec<SocketAddr>,
    pub(crate) mli: Arc<Box<dyn MLI>>,
    /// The specification associated with the server
    pub spec: &'static crate::iso8583::iso_spec::Spec,
    /// The message processor to be used to handle incoming requests
    pub(crate) msg_processor: Arc<Box<dyn MsgProcessor>>,
}

/// This trait whose implementation is used by the IsoServer to handle incoming requests
pub trait MsgProcessor: Send + Sync {
    fn process(&self, iso_server: &ISOServer, msg: &mut Vec<u8>) -> Result<(Vec<u8>, IsoMsg), IsoError>;
}

impl ISOServer {
    /// Returns a new ISO server on success or a IsoServer if the provided addr is incorrect
    pub fn new<'a>(host_port: String, spec: &'static Spec, mli_type: MLIType, msg_processor: Box<dyn MsgProcessor>) -> Result<ISOServer, IsoServerError> {
        let mli: Arc<Box<dyn MLI>>;

        match mli_type {
            MLIType::MLI2E => {
                mli = Arc::new(Box::new(MLI2E {}));
            }
            MLIType::MLI2I => {
                mli = Arc::new(Box::new(MLI2I {}));
            }
            MLIType::MLI4E => {
                mli = Arc::new(Box::new(MLI4E {}));
            }
            MLIType::MLI4I => {
                mli = Arc::new(Box::new(MLI4I {}));
            }
        }

        match host_port.to_socket_addrs() {
            Ok(addrs) => {
                let addrs = addrs.as_slice();
                //use only ipv4 for now
                let addrs = addrs.iter().filter(|s| s.is_ipv4()).map(|s| *s).collect::<Vec<SocketAddr>>();

                if addrs.len() > 0 {
                    Ok(ISOServer { sock_addr: addrs, spec, mli, msg_processor: Arc::new(msg_processor) })
                } else {
                    Err(IsoServerError { msg: format!("invalid host_port: {} : unresolvable?", &host_port) })
                }
            }
            Err(e) => Err(IsoServerError { msg: format!("invalid host_port: {}: cause: {}", &host_port, e.to_string()) })
        }
    }

    /// Starts the server in a separate thread
    pub fn start(&self) -> JoinHandle<()> {
        let server = ISOServer {
            sock_addr: self.sock_addr.clone(),
            spec: self.spec,
            mli: self.mli.clone(),
            msg_processor: self.msg_processor.clone(),
        };

        std::thread::spawn(move || {
            let listener = std::net::TcpListener::bind(server.sock_addr.as_slice()).unwrap();

            for stream in listener.incoming() {
                let client = stream.unwrap();
                debug!("Accepted new connection .. {:?}", &client.peer_addr());
                new_client(&server, client);
            }
        })
    }
}

/// Runs a new thread to handle a new incoming connection
fn new_client(iso_server: &ISOServer, stream_: TcpStream) {
    let server = ISOServer {
        sock_addr: iso_server.sock_addr.clone(),
        spec: iso_server.spec,
        mli: iso_server.mli.clone(),
        msg_processor: iso_server.msg_processor.clone(),
    };

    std::thread::spawn(move || {
        let stream = stream_;
        let mut reading_mli = true;
        let mut mli: u32 = 0;

        let mut reader = BufReader::new(&stream);
        let mut writer: Box<dyn Write> = Box::new(&stream);

        'done:
        loop {
            if reading_mli {
                match server.mli.parse(&mut reader) {
                    Ok(n) => {
                        mli = n;
                        reading_mli = false;
                    }
                    Err(e) => {
                        error!("client socket_err: {} {}", &stream.peer_addr().unwrap().to_string(), e.msg);
                        break 'done;
                    }
                };
            } else {
                if mli > 0 {
                    let mut data = vec![0; mli as usize];
                    match reader.read_exact(&mut data[..]) {
                        Err(e) => {
                            error!("client socket_err: {} {}", stream.peer_addr().unwrap().to_string(), e.to_string());
                            break 'done;
                        }
                        _ => (),
                    };

                    debug!("received request: \n{}\n len = {}", get_hexdump(&data), mli);
                    let t1 = std::time::Instant::now();

                    match server.msg_processor.process(&server, &mut data) {
                        Ok(resp) => {
                            debug!("iso_response : {} \n parsed :\n --- {} \n --- \n", get_hexdump(&resp.0), resp.1);
                            match server.mli.create(&(resp.0).len()) {
                                Ok(mut resp_data) => {
                                    debug!("request processing time = {} millis", std::time::Instant::now().duration_since(t1).as_millis());
                                    (&mut resp_data).write_all(resp.0.as_slice()).unwrap();
                                    writer.write_all(resp_data.as_slice()).unwrap();
                                    writer.flush().unwrap();
                                }
                                Err(e) => {
                                    error!("failed to construct mli {}", e.msg)
                                }
                            }
                        }
                        Err(e) => {
                            error!("failed to handle incoming req - {}", e.msg)
                        }
                    }
                    mli = 0;
                    reading_mli = true;
                }
            }
        }
    });
}


pub(in crate::iso8583) fn get_hexdump(data: &Vec<u8>) -> String {
    let mut hexdmp = String::new();
    hexdmp.push_str("\n");
    hexdump_iter(data).for_each(|f| {
        hexdmp.push_str(f.as_ref());
        hexdmp.push_str("\n");
    });
    hexdmp
}



