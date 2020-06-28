# iso8583_rs
ISO8583 library written in Rust 

![Crates.io](https://img.shields.io/crates/d/iso8583_rs?style=flat-square)

__Early days., No promise of backward compatibility for v0.1.* :)__

* Define a ISO specification in a YAML file
* Define a message-processor that can "act" on an incoming message and generate a response
* Start a ISO8583 server based on the spec and the message-processor (see example below)
* Use a TCP client to invoke the ISO server
* A sample spec is defined in [sample_spec.yaml](sample_spec/sample_spec.yaml)
* An ENV variable **SPEC_FILE** defines the location of the YAML spec definition file 
* Supports ASCII, EBCDIC, BINARY/BCD encoding

## Notes

Each spec defines a set of header fields (typically the MTI or Message Type), followed by any number
of messages (auth/reversal etc). 

For each incoming request (buffer), the header fields are parsed. The value of the parsed header field is matched against the selector
defined on the message. 

On successful match, the incoming data is parsed against the message. Once parsed, the message is fed into the MsgProcessor
defined on the server. The MsgProcessor applies its logic and generates a response which is sent back to the client.   


## Usage: 
``` rust

extern crate byteorder;
extern crate hex;
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate simplelog;

use iso8583_rs::iso8583::IsoError;
use iso8583_rs::iso8583::iso_spec::{IsoMsg, new_msg};
use iso8583_rs::iso8583::server::ISOServer;
use iso8583_rs::iso8583::server::MsgProcessor;
use iso8583_rs::iso8583::mli::MLIType::MLI2E;


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
                    handle_1100(&iso_msg, &mut iso_resp_msg)?
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


// Handle the incoming 1100 message based on amount
fn handle_1100(iso_msg: &IsoMsg, iso_resp_msg: &mut IsoMsg) -> Result<(), IsoError> {

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
                        val += "-OK";
                        iso_resp_msg.set_on(61, val.as_str()).unwrap();
                    }

                    if iso_msg.bmp.is_on(62) {
                        let mut val = iso_msg.bmp_child_value(62).unwrap();
                        val += "-OK";
                        iso_resp_msg.set_on(62, val.as_str()).unwrap();
                    }

                    iso_resp_msg.set_on(63, "007").unwrap_or_default();
                }
                Err(_e) => {
                    iso_resp_msg.set_on(39, "107").unwrap_or_default();
                }
            };

            iso_resp_msg.echo_from(&iso_msg, &[2, 3, 4, 11, 14, 19, 96])?;
            iso_resp_msg.fd_map.insert("bitmap".to_string(), iso_resp_msg.bmp.as_vec());

            Ok(())
        }
        Err(e) => {
            error!("No amount in request, responding with 115. error = {}", e.msg);
            iso_resp_msg.set("message_type", "1110").unwrap_or_default();
            iso_resp_msg.set_on(39, "115").unwrap_or_default();
            iso_resp_msg.echo_from(&iso_msg, &[2, 3, 4, 11, 14, 19, 96])
        }
    }
}


fn main() {
    std::env::set_var("SPEC_FILE", "sample_spec\\sample_spec.yaml");

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
        std::env::set_var("SPEC_FILE", "sample_spec/sample_spec.yaml");

        let spec = crate::iso8583::iso_spec::spec("");
        let msg_seg = spec.get_message_from_header("1100").unwrap();

        let mut iso_msg = iso_spec::new_msg(spec, msg_seg);

        iso_msg.set("message_type", "1100").unwrap();
        iso_msg.set_on(2, "123456789101").unwrap();
        iso_msg.set_on(3, "004000").unwrap();
        iso_msg.set_on(4, "000000000199").unwrap();
        iso_msg.set_on(11, "779581").unwrap();
        iso_msg.set_on(14, "2204").unwrap();
        iso_msg.set_on(19, "840").unwrap();
        iso_msg.set_on(52, "0102030405060708").unwrap();
        iso_msg.set_on(61, "reserved_1").unwrap();
        iso_msg.set_on(62, "reserved-2").unwrap();
        iso_msg.set_on(63, "87877622525").unwrap();
        iso_msg.set_on(96, "1234").unwrap();
        iso_msg.set_on(160, "5678").unwrap();

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
   Compiling iso8583_rs v0.1.5 (C:\Users\rkbal\IdeaProjects\iso8583_rs)
    Finished dev [unoptimized + debuginfo] target(s) in 2.95s
     Running `target\debug\iso8583_rs.exe`
current-dir: C:\Users\rkbal\IdeaProjects\iso8583_rs
spec-file: sample_spec\sample_spec.yaml
15:14:36 [INFO] starting iso server for spec SampleSpec at port 6666
15:14:48 [DEBUG] (2) iso8583_rs::iso8583::server: Accepted new connection .. Ok(V4(127.0.0.1:62986))
15:14:48 [DEBUG] (3) iso8583_rs::iso8583::server: received request: 

|31313030 f0242000 0000100e 80000001| 1100.$ ......... 00000000
|00000000 00000001 00000000 31323132| ............1212 00000010
|33343536 37383931 30313030 34303030| 3456789101004000 00000020
|30303030 30303030 30313939 37373935| 0000000001997795 00000030
|38313232 3034f8f4 f0010203 04050607| 812204.......... 00000040
|08001072 65736572 7665645f 310a9985| ...reserved_1... 00000050
|a28599a5 858460f2 f0f1f138 37383737| ......`....87877 00000060
|36323235 32353132 33343536 3738|     62252512345678   00000070
                                                       0000007e

 len = 126
15:14:48 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: computed header value for incoming message = 1100
15:14:48 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: parsing field : message_type
15:14:48 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: parsing field : bitmap
15:14:48 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - pan
15:14:48 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - proc_code
15:14:48 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - amount
15:14:48 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - stan
15:14:48 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - expiration_date
15:14:48 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - country_code
15:14:48 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - pin_data
15:14:48 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - private_1
15:14:48 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - private_2
15:14:48 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - private_3
15:14:48 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - key_mgmt_data
15:14:48 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - reserved_data
24
15:14:48 [DEBUG] (3) iso8583_rs: parsed incoming request - message = "1100 - Authorization" successfully. 
 : parsed message: 
 --- 
 
-Field-              : -Position-  : -Field Value- 
message_type         :             : 1100 
bitmap               :             : f02420000000100e80000001000000000000000100000000 
pan                  : 002         : 123456789101 
proc_code            : 003         : 004000 
amount               : 004         : 000000000199 
stan                 : 011         : 779581 
expiration_date      : 014         : 2204 
country_code         : 019         : 840 
pin_data             : 052         : 0102030405060708 
private_1            : 061         : reserved_1 
private_2            : 062         : reserved-2 
private_3            : 063         : 87877622525 
key_mgmt_data        : 096         : 1234 
reserved_data        : 160         : 5678  
 ----

15:14:48 [DEBUG] (3) iso8583_rs: amount = 199
15:14:48 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: echoing .. 2: 123456789101
15:14:48 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: echoing .. 3: 004000
15:14:48 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: echoing .. 4: 000000000199
15:14:48 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: echoing .. 11: 779581
15:14:48 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: echoing .. 14: 2204
15:14:48 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: echoing .. 19: 840
15:14:48 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: echoing .. 96: 1234
15:14:48 [DEBUG] (3) iso8583_rs::iso8583::server: iso_response : 
|31313130 f0242000 0200000e 00000001| 1110.$ ......... 00000000
|00000000 31323132 33343536 37383931| ....121234567891 00000010
|30313030 34303030 30303030 30303030| 0100400000000000 00000020
|30313939 37373935 38313232 3034f8f4| 01997795812204.. 00000030
|f0313030 00137265 73657276 65645f31| .100..reserved_1 00000040
|2d4f4b0d 9985a285 99a58584 60f260d6| -OK.........`.`. 00000050
|d2f0f0f3 30303731 323334|            ....0071234      00000060
                                                       0000006b
 
 parsed :
 --- 
-Field-              : -Position-  : -Field Value- 
message_type         :             : 1110 
bitmap               :             : f02420000200000e0000000100000000 
pan                  : 002         : 123456789101 
proc_code            : 003         : 004000 
amount               : 004         : 000000000199 
stan                 : 011         : 779581 
expiration_date      : 014         : 2204 
country_code         : 019         : 840 
action_code          : 039         : 100 
private_1            : 061         : reserved_1-OK 
private_2            : 062         : reserved-2-OK 
private_3            : 063         : 007 
key_mgmt_data        : 096         : 1234  
 --- 

15:14:48 [DEBUG] (3) iso8583_rs::iso8583::server: request processing time = 5 millis
15:14:48 [INFO] client socket closed : 127.0.0.1:62986

``` 

## ISO TCP Client

Now run src/iso8583/test.rs:test_send_recv_iso(..)

```
Testing started at 20:44 ...
current-dir: C:\Users\rkbal\IdeaProjects\iso8583_rs
spec-file: sample_spec/sample_spec.yaml
raw iso msg = 007e31313030f02420000000100e80000001000000000000000100000000313231323334353637383931303130303430303030303030303030303031393937373935383132323034f8f4f00102030405060708001072657365727665645f310a9985a28599a5858460f2f0f1f138373837373632323532353132333435363738
connected to server @ Ok(V4(127.0.0.1:62986))
received response: with  107 bytes. 
 
|31313130 f0242000 0200000e 00000001| 1110.$ ......... 00000000
|00000000 31323132 33343536 37383931| ....121234567891 00000010
|30313030 34303030 30303030 30303030| 0100400000000000 00000020
|30313939 37373935 38313232 3034f8f4| 01997795812204.. 00000030
|f0313030 00137265 73657276 65645f31| .100..reserved_1 00000040
|2d4f4b0d 9985a285 99a58584 60f260d6| -OK.........`.`. 00000050
|d2f0f0f3 30303731 323334|            ....0071234      00000060
                                                       0000006b


16
Received 1100 - Authorization 
 
-Field-              : -Position-  : -Field Value- 
message_type         :             : 1110 
bitmap               :             : f02420000200000e0000000100000000 
pan                  : 002         : 123456789101 
proc_code            : 003         : 004000 
amount               : 004         : 000000000199 
stan                 : 011         : 779581 
expiration_date      : 014         : 2204 
country_code         : 019         : 840 
action_code          : 039         : 100 
private_1            : 061         : reserved_1-OK 
private_2            : 062         : reserved-2-OK 
private_3            : 063         : 007 
key_mgmt_data        : 096         : 1234 

```

