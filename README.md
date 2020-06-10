# iso8583_rs
ISO8583 library written in Rust

# This project is WIP

* A sample spec is defined in iso_spec.rs
* Only ASCII encoding is supported at this time
* Run main.rs..

```
    Finished dev [unoptimized + debuginfo] target(s) in 1.77s
     Running `target\debug\iso8583_rs.exe`
13:37:09 [INFO] starting iso server for spec SampleSpec at port 6666
13:37:21 [DEBUG] (2) iso8583_rs::iso8583::server: Accepted new connection .. Ok(V6([::1]:58535))
13:37:21 [DEBUG] (3) iso8583_rs::iso8583::server: received request len = 54  : data = 313130307024000000000000313231323334353637383931303130303430303030303030303030303032303037373935383132323034
13:37:21 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: parsing field : message_type
13:37:21 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: parsing field : bitmap
13:37:21 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - pan
13:37:21 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - proc_code
13:37:21 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - amount
13:37:21 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - stan
13:37:21 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - expiration_date
13:37:21 [DEBUG] (3) iso8583_rs::iso8583::server: parsed incoming request - message type = "1100" successfully. 
 : parsed message: 
 --- 
 
expiration_date     : 2204 
proc_code           : 004000 
amount              : 000000000200 
bitmap              : 7024000000000000 
message_type        : 1100 
stan                : 779581 
pan                 : 123456789101  
 ----

13:37:21 [DEBUG] (3) iso8583_rs::iso8583::server: amount = 200
13:37:21 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: echoing .. 2: 123456789101
13:37:21 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: echoing .. 3: 004000
13:37:21 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: echoing .. 4: 000000000200
13:37:21 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: echoing .. 11: 779581
13:37:21 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: echoing .. 14: 2204
13:37:21 [DEBUG] (3) iso8583_rs::iso8583::server: iso_response 
 raw:: 313131307024000002000000313231323334353637383931303130303430303030303030303030303032303037373935383132323034313030, 
 parsed:: 
 
expiration_date     : 2204 
message_type        : 1110 
action_code         : 100 
pan                 : 123456789101 
proc_code           : 004000 
amount              : 000000000200 
stan                : 779581  
 
13:37:21 [INFO] client socket closed : [::1]:58535


``` 


Then, in a separate window run - src/iso8583/test_tcp_client.rs:13

```

Testing started at 19:12 ...
raw iso msg = 0036313130307024000000000000313231323334353637383931303130303430303030303030303030303030393937373935383132323034
received response:  "313131307024000006000000313231323334353637383931303130303430303030303030303030303030393937373935383132323034415050523031303030" with  63 bytes.
parsed iso-response:: 
 
message_type        : 1110 
approval_code       : APPR01 
bitmap              : 7024000006000000 
stan                : 779581 
expiration_date     : 2204 
pan                 : 123456789101 
proc_code           : 004000 
action_code         : 000 
amount              : 000000000099  

```

