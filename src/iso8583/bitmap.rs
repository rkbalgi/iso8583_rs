use std::fmt::format;

#[derive(Debug)]
pub struct Bitmap {
    p_bmp: u64,
    s_bmp: u64,
    t_bmp: u64,
}

pub fn new_bmp(b1: u64, b2: u64, b3: u64) -> Bitmap {
    Bitmap {
        p_bmp: b1,
        s_bmp: b2,
        t_bmp: b3,
    }
}

impl Bitmap {
    pub fn is_on(self: &Bitmap, pos: u32) -> bool {
        assert!(pos > 0 && pos <= 192);
        //println!("{:0x}{}", self.p_bmp >> 8, self.p_bmp >> ((64 as u32) - pos) as u64);

        if pos < 65 {
            self.p_bmp >> ((64 as u32) - pos) as u64 & 0x01 == 0x01
        } else if pos > 64 && pos < 129 {
            self.s_bmp >> ((64 as u32) - (pos - 64)) as u64 & 0x01 == 0x01
        } else {
            self.t_bmp >> ((64 as u32) - (pos - 128)) as u64 & 0x01 == 0x01
        }
    }

    pub fn hex_string(self: &Bitmap) -> String {
        format!("{:16.0x}{:016.0x}{:16.0x}",self.p_bmp,self.s_bmp,self.t_bmp)
    }
}

