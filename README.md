# iso8583_rs
ISO8583 library written in Rust 

![Crates.io](https://img.shields.io/crates/v/iso8583_rs?style=flat-square)
![Crates.io](https://img.shields.io/crates/d/iso8583_rs?style=flat-square)
[![Build Status](https://travis-ci.org/rkbalgi/iso8583_rs.svg?branch=master)](https://travis-ci.org/rkbalgi/iso8583_rs)

__Early days., No promise of backward compatibility for v0.1.* :)__

### New in 0.1.7
* Support for building PIN blocks (F52) in ISO0,ISO1,ISO2,ISO3 formats
### New in 0.1.8
* Support for Retail (X9.19 or ISO9797 Algorithm-3) and CBC MAC (ISO9797 Algorithm-1)
## Features

* Define a ISO specification in a YAML file
* Define a message-processor that can "act" on an incoming message and generate a response
* Start a ISO8583 server based on the spec and the message-processor (see example below)
* Use a TCP client to invoke the ISO server
* A sample spec is defined in [sample_spec.yaml](sample_spec/sample_spec.yaml)
* An ENV variable **SPEC_FILE** defines the location of the YAML spec definition file 
* Supports ASCII, EBCDIC, BINARY/BCD encoding
* Support for building PIN blocks (F52) in ISO0,ISO1,ISO2,ISO3 formats
* Support for Retail (X9.19 or ISO9797 Algorithm-3) and CBC MAC (ISO9797 Algorithm-1)

## Notes

Each spec defines a set of header fields (typically the MTI or Message Type), followed by any number
of messages (auth/reversal etc). 

For each incoming request (buffer), the header fields are parsed. The value of the parsed header field is matched against the selector
defined on the message. 

On successful match, the incoming data is parsed against the message. Once parsed, the message is fed into the MsgProcessor
defined on the server. The MsgProcessor applies its logic and generates a response which is sent back to the client.   


## Example Server Application: 
(Adapted from [main.rs](https://github.com/rkbalgi/iso8583-server/blob/master/src/main.rs) )

```rust
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

    let key = hex_l!("e0f4543f3e2a2c5ffc7e5e5a222e3e4d").to_vec();

    iso_resp_msg.set("message_type", "1110").unwrap_or_default();
    //validate the mac
    if iso_msg.bmp.is_on(64) || iso_msg.bmp.is_on(128) {
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
                iso_resp_msg.echo_from(&iso_msg, &[2, 3, 4, 11, 14, 19, 96]).unwrap_or_default();
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
                                     iso_msg.bmp_child_value(2).unwrap().as_str(), &key) {
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





```

## Sample TCP client

```rust
    
fn test_send_recv_iso_1100() -> Result<(), IsoError> {
        let path = Path::new(".").join("sample_spec").join("sample_spec.yaml");
        std::env::set_var("SPEC_FILE", path.to_str().unwrap());

        let spec = crate::iso8583::iso_spec::spec("");
        let msg_seg = spec.get_message_from_header("1100").unwrap();


        let mut iso_msg = iso_spec::new_msg(spec, msg_seg);

        iso_msg.set("message_type", "1100").unwrap();
        iso_msg.set_on(2, "4567909845671235").unwrap();
        iso_msg.set_on(3, "004000").unwrap();
        iso_msg.set_on(4, "000000000029").unwrap();
        iso_msg.set_on(11, "779581").unwrap();
        iso_msg.set_on(14, "2204").unwrap();
        iso_msg.set_on(19, "840").unwrap();


        let mut cfg = Config::new();
        cfg.with_pin(ISO0, String::from("e0f4543f3e2a2c5ffc7e5e5a222e3e4d"))
            .with_mac(RetailMac, Type1, String::from("e0f4543f3e2a2c5ffc7e5e5a222e3e4d"));


        //--------- set pin - F52

        //this will compute a pin based on cfg and the supplied pan and set bit position 52
        iso_msg.set_pin("1234", iso_msg.bmp_child_value(2).unwrap().as_str(), &cfg).unwrap();

        // You can also directly set this if there are other means of computing the pin block
        // iso_msg.set_on(52, "0102030405060708").unwrap(); //binary field are represented in their hex encoded format

        //--------- set pin - F52

        iso_msg.set_on(61, "reserved_1").unwrap();
        iso_msg.set_on(62, "reserved-2").unwrap();
        iso_msg.set_on(63, "87877622525").unwrap();
        iso_msg.set_on(96, "1234").unwrap();


        //--------- set mac  - either F64 or F128
        iso_msg.set_mac(&cfg);
        //--------- set mac


        let mut client = ISOTcpClient::new("localhost:6666", &spec, MLI2E);

        match client.send(&iso_msg) {
            Ok(resp_iso_msg) => {
                println!("Received {} \n {}", resp_iso_msg.msg.name(), resp_iso_msg);
            }
            Err(e) => {
                eprintln!("{:?}", e)
            }
        }
        Ok(())
    }

```

## Run ISO Server
* Run main.rs to start the ISO server (backed by above spec)

```
C:/Users/rkbal/.cargo/bin/cargo.exe run --color=always --package iso8583_rs --bin iso8583_rs
   Compiling iso8583_rs v0.1.6 (C:\Users\rkbal\IdeaProjects\iso8583_rs)
    Finished dev [unoptimized + debuginfo] target(s) in 2.97s
     Running `target\debug\iso8583_rs.exe`

current-dir: C:\Users\rkbal\IdeaProjects\iso8583_rs
spec-file: .\sample_spec\sample_spec.yaml
15:54:37 [INFO] starting iso server for spec SampleSpec at port 6666
15:54:47 [DEBUG] (2) iso8583_rs::iso8583::server: Accepted new connection .. Ok(V4(127.0.0.1:56307))
15:54:47 [DEBUG] (3) iso8583_rs::iso8583::server: received request: 

|31313030 f0242000 0000100e 00000001| 1100.$ ......... 00000000
|00000001 31363435 36373930 39383435| ....164567909845 00000010
|36373132 33353030 34303030 30303030| 6712350040000000 00000020
|30303030 30303239 37373935 38313232| 0000002977958122 00000030
|3034f8f4 f077fcbd 9ffc0dfa 6f001072| 04...w......o..r 00000040
|65736572 7665645f 310a9985 a28599a5| eserved_1....... 00000050
|858460f2 f0f1f138 37383737 36323235| ..`....878776225 00000060
|32353132 3334e470 06f5de8c 70b9|     251234.p....p.   00000070
                                                       0000007e

 len = 126
15:54:47 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: computed header value for incoming message = 1100
15:54:47 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: parsing field : message_type
15:54:47 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: parsing field : bitmap
15:54:47 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - pan
15:54:47 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - proc_code
15:54:47 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - amount
15:54:47 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - stan
15:54:47 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - expiration_date
15:54:47 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - country_code
15:54:47 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - pin_data
15:54:47 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - private_1
15:54:47 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - private_2
15:54:47 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - private_3
15:54:47 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - key_mgmt_data
15:54:47 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - mac_2
15:54:47 [DEBUG] (3) iso8583_rs: parsed incoming request - message = "1100 - Authorization" successfully. 
 : parsed message: 
 --- 
 
-Field-              : -Position-  : -Field Value- 
message_type         :             : 1100 
bitmap               :             : f02420000000100e0000000100000001 
pan                  :    002      : 4567909845671235 
proc_code            :    003      : 004000 
amount               :    004      : 000000000029 
stan                 :    011      : 779581 
expiration_date      :    014      : 2204 
country_code         :    019      : 840 
pin_data             :    052      : 77fcbd9ffc0dfa6f 
private_1            :    061      : reserved_1 
private_2            :    062      : reserved-2 
private_3            :    063      : 87877622525 
key_mgmt_data        :    096      : 1234 
mac_2                :    128      : e47006f5de8c70b9  
 ----

generating mac on 31313030f02420000000100e000000010000000131363435363739303938343536373132333530303430303030303030303030303030323937373935383132323034f8f4f077fcbd9ffc0dfa6f001072657365727665645f310a9985a28599a5858460f2f0f1f1383738373736323235323531323334
15:54:47 [DEBUG] (3) iso8583_rs: mac verified OK!
15:54:47 [DEBUG] (3) iso8583_rs: amount = 29
15:54:47 [DEBUG] (3) iso8583_rs: verifying pin ... 
15:54:47 [DEBUG] (3) iso8583_rs::crypto::pin: verifying pin - expected_pin: 1234,  block: 77fcbd9ffc0dfa6f, pan:4567909845671235, key:e0f4543f3e2a2c5ffc7e5e5a222e3e4d
15:54:47 [DEBUG] (3) iso8583_rs: PIN verified OK.
15:54:47 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: echoing .. 2: 4567909845671235
15:54:47 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: echoing .. 3: 004000
15:54:47 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: echoing .. 4: 000000000029
15:54:47 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: echoing .. 11: 779581
15:54:47 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: echoing .. 14: 2204
15:54:47 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: echoing .. 19: 840
15:54:47 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: echoing .. 96: 1234
15:54:47 [DEBUG] (3) iso8583_rs::iso8583::server: iso_response : 
|31313130 f0242000 0600000e 80000001| 1110.$ ......... 00000000
|00000000 00000001 00000000 31363435| ............1645 00000010
|36373930 39383435 36373132 33353030| 6790984567123500 00000020
|34303030 30303030 30303030 30303239| 4000000000000029 00000030
|37373935 38313232 3034f8f4 f0415050| 7795812204...APP 00000040
|52303130 30300013 72657365 72766564| R01000..reserved 00000050
|5f312d4f 4b0d9985 a28599a5 858460f2| _1-OK.........`. 00000060
|60d6d2f0 f0f33030 37313233 34463136| `.....0071234F16 00000070
|30|                                  0                00000080
                                                       00000081
 
 parsed :
 --- 
-Field-              : -Position-  : -Field Value- 
message_type         :             : 1110 
bitmap               :             : f02420000600000e80000001000000000000000100000000 
pan                  :    002      : 4567909845671235 
proc_code            :    003      : 004000 
amount               :    004      : 000000000029 
stan                 :    011      : 779581 
expiration_date      :    014      : 2204 
country_code         :    019      : 840 
approval_code        :    038      : APPR01 
action_code          :    039      : 000 
private_1            :    061      : reserved_1-OK 
private_2            :    062      : reserved-2-OK 
private_3            :    063      : 007 
key_mgmt_data        :    096      : 1234 
reserved_data        :    160      : F160  
 --- 

15:54:47 [DEBUG] (3) iso8583_rs::iso8583::server: request processing time = 9 millis
15:54:47 [ERROR] client socket_err: 127.0.0.1:56307 failed to fill whole buffer


``` 

## ISO TCP Client

Now run src/iso8583/test.rs:test_send_recv_iso_1100(..)

```
Testing started at 21:24 ...
kbalIdeaProjectsiso8583_rs
spec-file: .sample_specsample_spec.yaml
= 04123423142edc39
generating mac on 31313030f02420000000100e000000010000000131363435363739303938343536373132333530303430303030303030303030303030323937373935383132323034f8f4f077fcbd9ffc0dfa6f001072657365727665645f310a9985a28599a5858460f2f0f1f1383738373736323235323531323334
raw iso msg = 007e31313030f02420000000100e000000010000000131363435363739303938343536373132333530303430303030303030303030303030323937373935383132323034f8f4f077fcbd9ffc0dfa6f001072657365727665645f310a9985a28599a5858460f2f0f1f1383738373736323235323531323334e47006f5de8c70b9
connected to server @ Ok(V4(127.0.0.1:56307))
received response: with  129 bytes. 
 
|31313130 f0242000 0600000e 80000001| 1110.$ ......... 00000000
|00000000 00000001 00000000 31363435| ............1645 00000010
|36373930 39383435 36373132 33353030| 6790984567123500 00000020
|34303030 30303030 30303030 30303239| 4000000000000029 00000030
|37373935 38313232 3034f8f4 f0415050| 7795812204...APP 00000040
|52303130 30300013 72657365 72766564| R01000..reserved 00000050
|5f312d4f 4b0d9985 a28599a5 858460f2| _1-OK.........`. 00000060
|60d6d2f0 f0f33030 37313233 34463136| `.....0071234F16 00000070
|30|                                  0                00000080
                                                       00000081


Received 1100 - Authorization 
 
-Field-              : -Position-  : -Field Value- 
message_type         :             : 1110 
bitmap               :             : f02420000600000e80000001000000000000000100000000 
pan                  :    002      : 4567909845671235 
proc_code            :    003      : 004000 
amount               :    004      : 000000000029 
stan                 :    011      : 779581 
expiration_date      :    014      : 2204 
country_code         :    019      : 840 
approval_code        :    038      : APPR01 
action_code          :    039      : 000 
private_1            :    061      : reserved_1-OK 
private_2            :    062      : reserved-2-OK 
private_3            :    063      : 007 
key_mgmt_data        :    096      : 1234 
reserved_data        :    160      : F160 


```

