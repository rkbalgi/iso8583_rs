extern crate byteorder;
extern crate hex;
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate simplelog;

use std::collections::HashMap;

use iso8583_rs::iso8583::{bitmap, IsoError};
use iso8583_rs::iso8583::iso_spec::IsoMsg;
use iso8583_rs::iso8583::server::IsoServer;
use iso8583_rs::iso8583::server::MsgProcessor;


// Below is an example implementation of a MsgProcessor i.e the entity responsible for handling incoming messages
// at the server
#[derive(Copy, Clone)]
pub struct SampleMsgProcessor {}


impl MsgProcessor for SampleMsgProcessor {
    fn process(&self, iso_server: &IsoServer, msg: &mut Vec<u8>) -> Result<(Vec<u8>, IsoMsg), IsoError> {
        match iso_server.spec.parse(msg) {
            Ok(iso_msg) => {
                debug!("parsed incoming request - message = \"{}\" successfully. \n : parsed message: \n --- \n {} \n ----\n",
                       iso_msg.msg.name(), iso_msg);

                let mut iso_resp_msg = IsoMsg {
                    spec: &iso_msg.spec,
                    msg: &iso_msg.spec.get_message_from_header("1110").unwrap(),
                    fd_map: HashMap::new(),
                    bmp: bitmap::new_bmp(0, 0, 0),
                };

                // process the incoming request based on amount
                match iso_msg.bmp_child_value(4) {
                    Ok(amt) => {
                        iso_resp_msg.set("message_type", "1110").unwrap_or_default();

                        match amt.parse::<u32>() {
                            Ok(i_amt) => {
                                debug!("amount = {}", i_amt);
                                if i_amt < 100 {
                                    iso_resp_msg.set_on(38, "APPR01").unwrap_or_default();
                                    iso_resp_msg.set_on(39, "000").unwrap_or_default();
                                } else {
                                    iso_resp_msg.set_on(39, "100").unwrap_or_default();
                                }

                                if iso_msg.bmp.is_on(61) {
                                    let mut val = iso_msg.bmp_child_value(61).unwrap();
                                    val.push_str(" - OK");
                                    iso_resp_msg.set_on(61, val.as_str()).unwrap_or_default();
                                }
                                
                                if iso_msg.bmp.is_on(62) {
                                    let mut val = iso_msg.bmp_child_value(62).unwrap();
                                    val.push_str(" - OK");
                                    iso_resp_msg.set_on(62, val.as_str()).unwrap_or_default();
                                }

                                iso_resp_msg.set_on(63, "007").unwrap_or_default();
                            }
                            Err(_e) => {
                                iso_resp_msg.set_on(39, "107").unwrap_or_default();
                            }
                        };

                        match iso_resp_msg.echo_from(&iso_msg, &[2, 3, 4, 11, 14, 19, 96]) {
                            Err(e) => {
                                error!("failed to echo fields into response. error = {:?}", e);
                            }
                            _ => {}
                        };

                        iso_resp_msg.fd_map.insert("bitmap".to_string(), iso_resp_msg.bmp.as_vec());
                    }
                    Err(e) => {
                        error!("No amount in request, responding with 115. error = {}", e.msg);
                        iso_resp_msg.set("message_type", "1110").unwrap_or_default();
                        iso_resp_msg.set_on(39, "115").unwrap_or_default();
                        match iso_resp_msg.echo_from(&iso_msg, &[2, 3, 4, 11, 14, 19, 96]) {
                            Err(e) => {
                                error!("failed to echo fields into response. error = {}", e.msg);
                            }
                            _ => {}
                        };
                    }
                }

                match iso_resp_msg.assemble() {
                    Ok(resp_data) => Ok((resp_data, iso_resp_msg)),
                    Err(e) => {
                        error!("Failed to assemble response message - {}", e.msg);
                        Err(IsoError { msg: format!("error: msg assembly failed..{} ", e.msg) })
                    }
                }
            }
            Err(e) => {
                Err(IsoError { msg: e.msg })
            }
        }
    }
}

fn main() {
    std::env::set_var("SPEC_FILE", "sample_spec\\sample_spec.yaml");

    let _ = simplelog::SimpleLogger::init(simplelog::LevelFilter::Debug, simplelog::Config::default());

    let iso_spec = iso8583_rs::iso8583::iso_spec::spec("");

    info!("starting iso server for spec {} at port {}", iso_spec.name(), 6666);
    let server: IsoServer = match iso8583_rs::iso8583::server::new("localhost:6666".to_string(),
                                                                   Box::new(iso8583_rs::iso8583::mli::MLI2E {}),
                                                                   Box::new(SampleMsgProcessor {}), iso_spec) {
        Ok(server) => {
            server
        }
        Err(e) => {
            error!("failed to start ISO server - {}", e.msg);
            panic!(e)
        }
    };
    server.start().join().unwrap()
}


