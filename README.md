# iso8583_rs
ISO8583 library written in Rust

# This project is WIP

* A sample spec is defined in iso_spec.rs
* Only ASCII encoding is supported at this time
* Edit the raw data in main.rs and run it..

```
raw iso msg = 313130306024000000000000313231323334353637383931303130303430303037373935383132323034
parsing field : message_type
before_parse:: 313130306024000000000000313231323334353637383931303130303430303037373935383132323034
parsed-data: 31313030
parsing field : bitmap
parsed-data: bitmap := 6024000000000000
before_parse:: 313231323334353637383931303130303430303037373935383132323034
parsed-data (len-ind) : 3132
before_parse:: 30303430303037373935383132323034
parsed-data: 303034303030
before_parse:: 37373935383132323034
parsed-data: 373739353831
before_parse:: 32323034
parsed-data: 32323034

expiration_date     : 32323034
stan                : 373739353831
pan                 : 313233343536373839313031
message_type        : 31313030
proc_code           : 303034303030
bitmap              : 6024000000000000

Process finished with exit code 0

``` 