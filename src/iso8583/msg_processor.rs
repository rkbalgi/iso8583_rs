use crate::iso8583::server::IsoServer;
use std::collections::HashMap;
use crate::iso8583::{bitmap, IsoError};
use crate::iso8583::iso_spec::IsoMsg;

#[derive(Copy, Clone)]

// MsgProcessor is used by the IsoServer to handle incoming requests
pub struct MsgProcessor {}

impl MsgProcessor {
    pub fn process(&self, iso_server: &IsoServer, msg: Vec<u8>) -> Result<(Vec<u8>, IsoMsg), IsoError> {
        match iso_server.spec.parse(msg) {
            Ok(iso_msg) => {
                debug!("parsed incoming request - message = \"{}\" successfully. \n : parsed message: \n --- \n {} \n ----\n",
                       iso_msg.msg.name(), iso_msg);

                let mut iso_resp_msg = IsoMsg { spec: &iso_msg.spec, msg: &iso_msg.spec.get_message_from_header("1110").unwrap(),
                    fd_map: HashMap::new(), bmp: bitmap::new_bmp(0, 0, 0) };

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

                        match iso_resp_msg.echo_from(&iso_msg, &[2, 3, 4, 11, 14]) {
                            Err(e) => {
                                error!("failed to echo fields into response. error = {}", "!");
                            }
                            _ => {}
                        };
                    }
                    Err(e) => {
                        iso_resp_msg.set("message_type", "1110");
                        iso_resp_msg.set_on(39, "115");
                        match iso_resp_msg.echo_from(&iso_msg, &[2, 3, 4, 11, 14]) {
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