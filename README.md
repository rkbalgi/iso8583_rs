# iso8583_rs
ISO8583 library written in Rust 

* Define and start a ISO8583 server
* Define a message-processor that can "act" on an incoming message and generate a response
* Use a TCP client to invoke the ISO server

(This project is mostly WIP) 


## Usage: 
``` rust

extern crate byteorder;
extern crate hex;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate simplelog;

use std::collections::HashMap;
use std::time::Duration;

use iso8583_rs::iso8583::{bitmap, IsoError};
use iso8583_rs::iso8583::iso_spec::IsoMsg;
use iso8583_rs::iso8583::msg_processor::MsgProcessor;
use iso8583_rs::iso8583::server::IsoServer;


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

    let iso_spec = iso8583_rs::iso8583::iso_spec::spec("");

    info!("starting iso server for spec {} at port {}", iso_spec.name(), 6666);
    let server: IsoServer = match iso8583_rs::iso8583::server::new("localhost:6666".to_string(), Box::new(SampleMsgProcessor {}), iso_spec) {
        Ok(server) => {
            server
        }
        Err(e) => {
            panic!(e)
        }
    };
    server.start().join();
}




```

###  Latest
* Read spec definition from a YAML file

## Notes
* A sample spec is defined in [sample_spec.yaml](sample_spec/sample_spec.yaml)
* An ENV variable **SPEC_FILE** defines the location of the yaml spec definition file 
* Only ASCII encoding is supported at this time

## Run ISO Server
* Run main.rs to start the ISO server (backed by above spec)

```
    Finished dev [unoptimized + debuginfo] target(s) in 2.22s
     Running `target\debug\iso8583_rs.exe`
current-dir: C:\Users\rkbal\IdeaProjects\iso8583_rs
spec-file: sample_spec\sample_spec.yaml
05:26:10 [INFO] starting iso server for spec SampleSpec at port 6666
05:28:19 [DEBUG] (2) iso8583_rs::iso8583::server: Accepted new connection .. Ok(V6([::1]:56859))
05:28:19 [DEBUG] (3) iso8583_rs::iso8583::server: received request len = 94  : data = 31313030f02400000000100080000001000000000000000100000000313231323334353637383931303130303430303030303030303030303031393937373935383132323034303130323033303430353036303730383132333438383838
05:28:19 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: computed header value for incoming message = 1100
05:28:19 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: parsing field : message_type
05:28:19 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: parsing field : bitmap
05:28:19 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - pan
05:28:19 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - proc_code
05:28:19 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - amount
05:28:19 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - stan
05:28:19 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - expiration_date
05:28:19 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - pin_data
05:28:19 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - key_mgmt_data
05:28:19 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - reserved_data
05:28:19 [DEBUG] (3) iso8583_rs: parsed incoming request - message = "1100 - Authorization" successfully. 
 : parsed message: 
 --- 
 
reserved_data       : 0708 
amount              : 000000000199 
bitmap              : f02400000000100080000001000000000000000100000000 
expiration_date     : 2204 
proc_code           : 004000 
stan                : 779581 
pin_data            : 3031303230333034 
message_type        : 1100 
pan                 : 123456789101 
key_mgmt_data       : 0506  
 ----

05:28:19 [DEBUG] (3) iso8583_rs: amount = 199
05:28:19 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: echoing .. 2: 123456789101
05:28:19 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: echoing .. 3: 004000
05:28:19 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: echoing .. 4: 000000000199
05:28:19 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: echoing .. 11: 779581
05:28:19 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: echoing .. 14: 2204
05:28:19 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: echoing .. 96: 0506
05:28:19 [DEBUG] (3) iso8583_rs::iso8583::server: iso_response 
 raw:: 31313130f024000002000000000000010000000031323132333435363738393130313030343030303030303030303030303139393737393538313232303431303030353036, 
 parsed:: 
 
pan                 : 123456789101 
proc_code           : 004000 
stan                : 779581 
expiration_date     : 2204 
amount              : 000000000199 
key_mgmt_data       : 0506 
message_type        : 1110 
bitmap              : f0240000020000000000000100000000 
action_code         : 100  
 
05:28:19 [INFO] client socket closed : [::1]:56859


``` 

## ISO TCP Client

Now run src/iso8583/test.rs:test_send_recv_iso(..)

```

Testing started at 10:58 ...
raw iso msg = 005e31313030f02400000000100080000001000000000000000100000000313231323334353637383931303130303430303030303030303030303031393937373935383132323034303130323033303430353036303730383132333438383838
received response:  "31313130f024000002000000000000010000000031323132333435363738393130313030343030303030303030303030303139393737393538313232303431303030353036" with  69 bytes.
current-dir: C:\Users\rkbal\IdeaProjects\iso8583_rs
spec-file: sample_spec/sample_spec.yaml
parsed iso-response "1100 - Authorization" 
 
proc_code           : 004000 
pan                 : 123456789101 
bitmap              : f0240000020000000000000100000000 
action_code         : 100 
key_mgmt_data       : 0506 
amount              : 000000000199 
stan                : 779581 
message_type        : 1110 
expiration_date     : 2204  



```

