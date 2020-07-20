use hex;
use log::{info, debug, error, warn};
use simplelog;
use hex_literal::hex as hex_l;

use iso8583_rs::iso8583::iso_spec::{IsoMsg, new_msg};
use iso8583_rs::iso8583::IsoError;
use iso8583_rs::iso8583::mli::MLIType::MLI2E;
use iso8583_rs::iso8583::server::ISOServer;
use iso8583_rs::iso8583::server::MsgProcessor;
use iso8583_rs::crypto::pin::verify_pin;
use iso8583_rs::crypto::pin::PinFormat::ISO0;
use std::path::Path;
use iso8583_rs::crypto::mac::MacAlgo::RetailMac;
use iso8583_rs::crypto::mac::PaddingType::Type1;
use iso8583_rs::crypto::mac::verify_mac;


// Below is an example implementation of a MsgProcessor i.e the entity responsible for handling incoming messages
// at the server
#[derive(Copy, Clone)]
pub struct SampleMsgProcessor {}


impl MsgProcessor for SampleMsgProcessor {
    fn process(&self, iso_server: &ISOServer, msg: &mut Vec<u8>) -> Result<(Vec<u8>, IsoMsg), IsoError> {
        match iso_server.spec.parse(msg) {
            Ok(iso_msg) => {
                debug!("parsed incoming request - message = \"{}\" successfully. \n : parsed message: \n --- \n {} \n ----\n",
                       iso_msg.msg.name(), iso_msg);

                let req_msg_type = iso_msg.get_field_value(&"message_type".to_string()).unwrap();
                let resp_msg_type = if req_msg_type == "1100" {
                    "1110"
                } else if req_msg_type == "1420" {
                    "1430"
                } else {
                    return Err(IsoError { msg: format!("unsupported msg_type {}", req_msg_type) });
                };


                let mut iso_resp_msg = new_msg(&iso_msg.spec, &iso_msg.spec.get_message_from_header(resp_msg_type).unwrap());

                if req_msg_type == "1420" {
                    iso_resp_msg.set("message_type", resp_msg_type).unwrap_or_default();
                    iso_resp_msg.echo_from(&iso_msg, &[2, 3, 4, 11, 14, 19, 96])?;
                    iso_resp_msg.set_on(39, "400").unwrap_or_default();
                } else if req_msg_type == "1100" {
                    handle_1100(&iso_msg, msg, &mut iso_resp_msg)?
                }


                match iso_resp_msg.assemble() {
                    Ok(resp_data) => Ok((resp_data, iso_resp_msg)),
                    Err(e) => {
                        error!("Failed to assemble response message, dropping message - {}", e.msg);
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


// Handle the incoming 1100 message based on amount
// if amount (F4) <100 then
//   F38 = APPR01;
//   F39 = 000;
// else
//   F39 = 100;
//
//
fn handle_1100(iso_msg: &IsoMsg, raw_msg: &Vec<u8>, iso_resp_msg: &mut IsoMsg) -> Result<(), IsoError> {
    iso_resp_msg.set("message_type", "1110").unwrap_or_default();
    //validate the mac
    if iso_msg.bmp.is_on(64) || iso_msg.bmp.is_on(128) {
        let key = hex_l!("e0f4543f3e2a2c5ffc7e5e5a222e3e4d").to_vec();
        let expected_mac = match iso_msg.bmp.is_on(64) {
            true => {
                iso_msg.bmp_child_value(64)
            }
            false => {
                iso_msg.bmp_child_value(128)
            }
        };
        let mac_data = &raw_msg.as_slice()[0..raw_msg.len() - 8];
        match verify_mac(&RetailMac, &Type1, mac_data, &key, &hex::decode(expected_mac.unwrap()).unwrap()) {
            Ok(_) => {
                debug!("mac verified OK!");
            }
            Err(e) => {
                error!("failed to verify mac. Reason: {}", e.msg);
                iso_resp_msg.set("message_type", "1110").unwrap_or_default();
                iso_resp_msg.set_on(39, "916").unwrap_or_default();
                iso_resp_msg.echo_from(&iso_msg, &[2, 3, 4, 11, 14, 19, 96]);
                return Ok(());
            }
        }
    }


    if !iso_msg.bmp.is_on(4) {
        error!("No amount in request, responding with F39 = 115 ");
        iso_resp_msg.set("message_type", "1110").unwrap_or_default();
        iso_resp_msg.set_on(39, "115").unwrap_or_default();
        iso_resp_msg.echo_from(&iso_msg, &[2, 3, 4, 11, 14, 19, 96])
    } else {
        // process the incoming request based on amount
        let amt = iso_msg.bmp_child_value(4).unwrap();
        match amt.parse::<u32>() {
            Ok(i_amt) => {
                debug!("amount = {}", i_amt);
                if i_amt < 100 {
                    iso_resp_msg.set_on(39, "000").unwrap_or_default();
                } else {
                    iso_resp_msg.set_on(39, "100").unwrap_or_default();
                }


                if iso_msg.bmp.is_on(52) {
                    //validate the pin
                    let f52 = iso_msg.bmp_child_value(52).unwrap();
                    debug!("{}", "verifying pin ... ");
                    match verify_pin(&ISO0, "1234", &hex::decode(f52).unwrap(),
                                     iso_msg.bmp_child_value(2).unwrap().as_str(), "e0f4543f3e2a2c5ffc7e5e5a222e3e4d") {
                        Ok(res) => {
                            if res {
                                debug!("{}", "PIN verified OK.");
                            } else {
                                warn!("{}", "PIN verified Failed!!");
                                iso_resp_msg.set_on(39, "117").unwrap_or_default();
                            }
                        }
                        Err(e) => {
                            error!("failed to verify PIN, {}", e.msg);
                            iso_resp_msg.set_on(39, "126").unwrap_or_default();
                        }
                    };
                }

                if iso_msg.bmp.is_on(61) {
                    let mut val = iso_msg.bmp_child_value(61).unwrap();
                    val += "-OK";
                    iso_resp_msg.set_on(61, val.as_str()).unwrap();
                }

                if iso_msg.bmp.is_on(62) {
                    let mut val = iso_msg.bmp_child_value(62).unwrap();
                    val += "-OK";
                    iso_resp_msg.set_on(62, val.as_str()).unwrap();
                }

                iso_resp_msg.set_on(63, "007").unwrap_or_default();
                iso_resp_msg.set_on(160, "F160").unwrap_or_default();


                if iso_resp_msg.bmp_child_value(39).unwrap() == "000" {
                    // generate a approval code
                    iso_resp_msg.set_on(38, "APPR01").unwrap_or_default();
                }
            }
            Err(_e) => {
                iso_resp_msg.set_on(39, "107").unwrap_or_default();
            }
        };

        iso_resp_msg.echo_from(&iso_msg, &[2, 3, 4, 11, 14, 19, 96])?;
        iso_resp_msg.fd_map.insert("bitmap".to_string(), iso_resp_msg.bmp.as_vec());

        Ok(())
    }
}


fn main() {
    let path = Path::new(".").join("sample_spec").join("sample_spec.yaml");
    let spec_file = path.to_str().unwrap();
    std::env::set_var("SPEC_FILE", spec_file);

    let _ = simplelog::SimpleLogger::init(simplelog::LevelFilter::Debug, simplelog::Config::default());

    let iso_spec = iso8583_rs::iso8583::iso_spec::spec("");

    info!("starting iso server for spec {} at port {}", iso_spec.name(), 6666);
    let server = match ISOServer::new("127.0.0.1:6666".to_string(),
                                      iso_spec,
                                      MLI2E,
                                      Box::new(SampleMsgProcessor {})) {
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


