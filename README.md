# iso8583_rs
ISO8583 library written in Rust

# This project is WIP

* A sample spec is defined in iso_spec.rs
* Only ASCII encoding is supported at this time
* Run main.rs..

```
16:51:30 [INFO] starting iso server for spec SampleSpec at port 6666
16:51:45 [DEBUG] (2) iso8583_rs::iso8583::server: Accepted new connection .. Ok(V6([::1]:59446))
16:51:45 [DEBUG] (3) iso8583_rs::iso8583::server: received request len = 42  : data = 313130306024000000000000313231323334353637383931303130303430303037373935383132323034
16:51:45 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: parsing field : message_type
16:51:45 [DEBUG] (3) iso8583_rs::iso8583::iso_spec: parsing field : bitmap
16:51:45 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - pan
16:51:45 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - proc_code
16:51:45 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - stan
16:51:45 [DEBUG] (3) iso8583_rs::iso8583::bitmap: parsing field - expiration_date
16:51:45 [DEBUG] (3) iso8583_rs::iso8583::server: parsed incoming request - message type =  successfully, 
 
expiration_date     : 2204 
message_type        : 1100 
pan                 : 123456789101 
bitmap              : 6024000000000000 
proc_code           : 004000 
stan                : 779581  
 
16:51:45 [INFO] client socket closed : [::1]:59446

``` 


Then, in a separate window run - src/iso8583/test_tcp_client.rs:13

