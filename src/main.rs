mod iso8583;

fn main() {
    //let testvec = vec![1, 2, 3];

    println!("Hello, world!");
    // println!("{:?} {:?}", testvec, iso8583::bitmap::new_bmp());

    let bmp = &iso8583::bitmap::new_bmp(0x7e02030400000001, 0, 0x8000000000000001);
    for i in 1..193 {
        println!("{} {}", i, bmp.is_on(i))
    }
    println!("{}",bmp.hex_string())
}
