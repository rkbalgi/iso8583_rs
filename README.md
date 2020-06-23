# iso8583_rs
ISO8583 library written in Rust 

![Crates.io](https://img.shields.io/crates/d/iso8583_rs?style=flat-square)

* Define and start a ISO8583 server
* Define a message-processor that can "act" on an incoming message and generate a response
* Use a TCP client to invoke the ISO server

(This project is mostly WIP) 


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






```


## Notes
* A sample spec is defined in [sample_spec.yaml](sample_spec/sample_spec.yaml)
* An ENV variable **SPEC_FILE** defines the location of the yaml spec definition file 
* Supports ASCII,EBCDIC, BINARY/BCD encoding 

## Run ISO Server
* Run main.rs to start the ISO server (backed by above spec)

```
C:/Users/rkbal/.cargo/bin/cargo.exe run --color=always --package iso8583_rs --bin iso8583_rs
   Compiling iso8583_rs v0.1.3 (C:\Users\rkbal\IdeaProjects\iso8583_rs)
    Finished dev [unoptimized + debuginfo] target(s) in 4.75s
     Running `target\debug\iso8583_rs.exe`
current-dir: C:\Users\rkbal\IdeaProjects\iso8583_rs
spec-file: sample_spec\sample_spec.yaml
20:10:04 [INFO] starting iso server for spec SampleSpec at port 6666
20:10:14 [DEBUG] (2) iso8583_rs::iso8583::server: Accepted new connection .. Ok(V6([::1]:57018))
20:10:14 [DEBUG] (3) iso8583_rs::iso8583::server: received request: 

|31313030 f0242000 0000100e 80000001| 1100.$ ......... 00000000
|00000000 00000001 00000000 31323132| ............1212 00000010
|33343536 37383931 30313030 34303030| 3456789101004000 00000020
|30303030 30303030 30313939 37373935| 0000000001997795 00000030
|38313232 3034f8f4 f0010203 04050607| 812204.......... 00000040
|08001152 61676861 76656e64 726111d9| ...Raghavendra.. 00000050
|81878881 a5859584 998140c2 81938789| ..........@..... 00000060
|f0f1f138 37383737 36323235 32353132| ...8787762252512 00000070
|33343838 3838|                       348888           00000080
                                                       00000086

 len = 134
20:10:14 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: computed header value for incoming message = 1100
20:10:14 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: parsing field : message_type
20:10:14 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: parsing field : bitmap
20:10:14 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - pan
20:10:14 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - proc_code
20:10:14 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - amount
20:10:14 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - stan
20:10:14 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - expiration_date
20:10:14 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - country_code
20:10:14 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - pin_data
20:10:14 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - private_1
20:10:14 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - private_2
20:10:14 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - private_3
20:10:14 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - key_mgmt_data
20:10:14 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - reserved_data
24
20:10:14 [DEBUG] (3) iso8583_rs: parsed incoming request - message = "1100 - Authorization" successfully. 
 : parsed message: 
 --- 
 
message_type             : 1100 
bitmap                   : f02420000000100e80000001000000000000000100000000 
pan                  [002]: 123456789101 
proc_code            [003]: 004000 
amount               [004]: 000000000199 
stan                 [011]: 779581 
expiration_date      [014]: 2204 
country_code         [019]: 840 
pin_data             [052]: 0102030405060708 
private_1            [061]: Raghavendra 
private_2            [062]: Raghavendra Balgi 
private_3            [063]: 87877622525 
key_mgmt_data        [096]: 1234 
reserved_data        [160]: 8888  
 ----

20:10:14 [DEBUG] (3) iso8583_rs: amount = 199
20:10:14 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: echoing .. 2: 123456789101
20:10:14 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: echoing .. 3: 004000
20:10:14 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: echoing .. 4: 000000000199
20:10:14 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: echoing .. 11: 779581
20:10:14 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: echoing .. 14: 2204
20:10:14 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: echoing .. 19: 840
20:10:14 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: echoing .. 96: 1234
20:10:14 [DEBUG] (3) iso8583_rs::iso8583::server: iso_response : 
|31313130 f0242000 0200000e 00000001| 1110.$ ......... 00000000
|00000000 31323132 33343536 37383931| ....121234567891 00000010
|30313030 34303030 30303030 30303030| 0100400000000000 00000020
|30313939 37373935 38313232 3034f8f4| 01997795812204.. 00000030
|f0313030 00165261 67686176 656e6472| .100..Raghavendr 00000040
|61202d20 4f4b16d9 81878881 a5859584| a - OK.......... 00000050
|998140c2 81938789 406040d6 d2f0f0f3| ..@.....@`@..... 00000060
|30303731 323334|                     0071234          00000070
                                                       00000077
 
 parsed :
--- 
message_type             : 1110 
bitmap                   : f02420000200000e0000000100000000 
pan                  [002]: 123456789101 
proc_code            [003]: 004000 
amount               [004]: 000000000199 
stan                 [011]: 779581 
expiration_date      [014]: 2204 
country_code         [019]: 840 
action_code          [039]: 100 
private_1            [061]: Raghavendra - OK 
private_2            [062]: Raghavendra Balgi - OK 
private_3            [063]: 007 
key_mgmt_data        [096]: 1234  -- 

20:10:14 [INFO] client socket closed : [::1]:57018

``` 

## ISO TCP Client

Now run src/iso8583/test.rs:test_send_recv_iso(..)

```
Testing started at 01:40 ...
raw iso msg = 008631313030f02420000000100e80000001000000000000000100000000313231323334353637383931303130303430303030303030303030303031393937373935383132323034f8f4f001020304050607080011526167686176656e64726111d981878881a5859584998140c281938789f0f1f138373837373632323532353132333438383838
received response: with  119 bytes.
|31313130 f0242000 0200000e 00000001| 1110.$ ......... 00000000
|00000000 31323132 33343536 37383931| ....121234567891 00000010
|30313030 34303030 30303030 30303030| 0100400000000000 00000020
|30313939 37373935 38313232 3034f8f4| 01997795812204.. 00000030
|f0313030 00165261 67686176 656e6472| .100..Raghavendr 00000040
|61202d20 4f4b16d9 81878881 a5859584| a - OK.......... 00000050
|998140c2 81938789 406040d6 d2f0f0f3| ..@.....@`@..... 00000060
|30303731 323334|                     0071234          00000070
                                                       00000077
current-dir: C:\Users\rkbal\IdeaProjects\iso8583_rs
spec-file: sample_spec/sample_spec.yaml
16
parsed iso-response "1100 - Authorization" 
 
message_type             : 1110 
bitmap                   : f02420000200000e0000000100000000 
pan                  [002]: 123456789101 
proc_code            [003]: 004000 
amount               [004]: 000000000199 
stan                 [011]: 779581 
expiration_date      [014]: 2204 
country_code         [019]: 840 
action_code          [039]: 100 
private_1            [061]: Raghavendra - OK 
private_2            [062]: Raghavendra Balgi - OK 
private_3            [063]: 007 
key_mgmt_data        [096]: 1234  


```

