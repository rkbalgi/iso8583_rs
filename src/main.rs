#[macro_use]
extern crate lazy_static;
extern crate hex;
extern crate byteorder;

#[macro_use]
extern crate log;
extern crate simplelog;

use std::time::Duration;
use crate::iso8583::server::IsoServer;

pub mod iso8583;


#[derive(Copy, Clone)]
pub struct SampleMsgProcessor {}

unsafe impl Sync for MsgProcessor{}
unsafe impl Send for MsgProcessor{}

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
                        iso_resp_msg.set("message_type", "1110");

                        match amt.parse::<u32>() {
                            Ok(i_amt) => {
                                debug!("amount = {}", i_amt);
                                if i_amt < 100 {
                                    iso_resp_msg.set_on(38, "APPR01");
                                    iso_resp_msg.set_on(39, "000");
                                } else {
                                    iso_resp_msg.set_on(39, "100");
                                }
                            }
                            Err(e) => {
                                iso_resp_msg.set_on(39, "107");
                            }
                        };

                        match iso_resp_msg.echo_from(&iso_msg, &[2, 3, 4, 11, 14, 96]) {
                            Err(e) => {
                                error!("failed to echo fields into response. error = {}", "!");
                            }
                            _ => {}
                        };

                        iso_resp_msg.fd_map.insert("bitmap".to_string(), iso_resp_msg.bmp.as_vec());
                    }
                    Err(e) => {
                        iso_resp_msg.set("message_type", "1110");
                        iso_resp_msg.set_on(39, "115");
                        match iso_resp_msg.echo_from(&iso_msg, &[2, 3, 4, 11, 14, 96]) {
                            Err(e) => {
                                error!("failed to echo fields into response. error = {}", "!");
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

    let iso_spec = crate::iso8583::iso_spec::spec("");

    info!("starting iso server for spec {} at port {}", iso_spec.name(), 6666);
    let server: IsoServer = **match crate::iso8583::server::new("localhost:6666".to_string(), Box::new(SampleMsgProcessor {}), iso_spec) {
        Ok(server) => {
            server
        }
        Err(e) => {
            panic!(e)
        }
    };
    server.start().join();
}


