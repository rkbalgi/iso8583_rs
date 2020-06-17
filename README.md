# iso8583_rs
ISO8583 library written in Rust 

* Define and start a ISO8583 server
* Define a message-processor that can "act" on an incoming message and generate a response
* Use a TCP client to invoke the ISO server

(This project is mostly WIP) 



###  Latest
* Included header fields support - messages are selected (using selector) based on header_fields defined on the Spec

## Notes
* A sample spec is defined in [iso_spec.rs](src/iso8583/iso_spec.rs)
* Only ASCII encoding is supported at this time

## Run ISO Server
* Run test_server in test.rs to start the ISO server (backed by above spec)

```
    Finished dev [unoptimized + debuginfo] target(s) in 1.77s
     Running `target\debug\iso8583_rs.exe`
13:37:09 [INFO] starting iso server for spec SampleSpec at port 6666
16:58:07 [DEBUG] (2) iso8583_rs::iso8583::server: Accepted new connection .. Ok(V6([::1]:59818))
16:58:07 [DEBUG] (4) iso8583_rs::iso8583::server: received request len = 54  : data = 313130307024000000000000313231323334353637383931303130303430303030303030303030303031393937373935383132323034
16:58:07 [DEBUG] (4) iso8583_rs::iso8583::iso_spec: computed header value for incoming message = 1100
16:58:07 [DEBUG] (4) iso8583_rs::iso8583::iso_spec: parsing field : message_type
16:58:07 [DEBUG] (4) iso8583_rs::iso8583::iso_spec: parsing field : bitmap
16:58:07 [DEBUG] (4) iso8583_rs::iso8583::bitmap: parsing field - pan
16:58:07 [DEBUG] (4) iso8583_rs::iso8583::bitmap: parsing field - proc_code
16:58:07 [DEBUG] (4) iso8583_rs::iso8583::bitmap: parsing field - amount
16:58:07 [DEBUG] (4) iso8583_rs::iso8583::bitmap: parsing field - stan
16:58:07 [DEBUG] (4) iso8583_rs::iso8583::bitmap: parsing field - expiration_date
16:58:07 [DEBUG] (4) iso8583_rs::iso8583::msg_processor: parsed incoming request - message = "Authorization Request - 1100" successfully. 
 : parsed message: 
 --- 
 
amount              : 000000000199 
stan                : 779581 
message_type        : 1100 
expiration_date     : 2204 
bitmap              : 7024000000000000 
pan                 : 123456789101 
proc_code           : 004000  
 ----

16:58:07 [DEBUG] (4) iso8583_rs::iso8583::msg_processor: amount = 199
16:58:07 [DEBUG] (4) iso8583_rs::iso8583::iso_spec: echoing .. 2: 123456789101
16:58:07 [DEBUG] (4) iso8583_rs::iso8583::iso_spec: echoing .. 3: 004000
16:58:07 [DEBUG] (4) iso8583_rs::iso8583::iso_spec: echoing .. 4: 000000000199
16:58:07 [DEBUG] (4) iso8583_rs::iso8583::iso_spec: echoing .. 11: 779581
16:58:07 [DEBUG] (4) iso8583_rs::iso8583::iso_spec: echoing .. 14: 2204
16:58:07 [DEBUG] (4) iso8583_rs::iso8583::server: iso_response 
 raw:: 313131307024000002000000313231323334353637383931303130303430303030303030303030303031393937373935383132323034313030, 
 parsed:: 
 
amount              : 000000000199 
proc_code           : 004000 
expiration_date     : 2204 
action_code         : 100 
message_type        : 1110 
stan                : 779581 
pan                 : 123456789101  
 
16:58:07 [INFO] client socket closed : [::1]:59818

``` 

## ISO TCP Client

Now run src/iso8583/test.rs:32

```

Testing started at 22:27 ...
raw iso msg = 0036313130307024000000000000313231323334353637383931303130303430303030303030303030303031393937373935383132323034
received response:  "313131307024000002000000313231323334353637383931303130303430303030303030303030303031393937373935383132323034313030" with  57 bytes.
parsed iso-response "Authorization Response - 1110" 
 
expiration_date     : 2204 
amount              : 000000000199 
proc_code           : 004000 
bitmap              : 7024000002000000 
stan                : 779581 
message_type        : 1110 
pan                 : 123456789101 
action_code         : 100  

```

