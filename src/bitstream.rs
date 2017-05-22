use std::ops::Add;
use std::fmt::*;
use std::result::Result;

#[derive(Clone)]
pub struct Bitstream {
    pub cap: u32,
    pub pos: u32,
    data: Vec<u8>,
}

impl Bitstream {
    pub fn new() -> Bitstream {
        Bitstream { cap: 8 * 8, pos: 0, data: vec![0; 8] }
    }

    pub fn append(&mut self, val: u8) {
        if self.pos >= self.cap {
            self.data.push(0);
            self.data.push(0);
            self.data.push(0);
            self.data.push(0);
            self.cap += 8 * 4;
        }

        let idx = (self.pos / 8) as usize;
        let bitidx = self.pos % 8;

        self.data[idx] = 
            self.data[idx] 
                & !(1 << bitidx) 
                | ((val & 1) << bitidx);
        self.pos += 1;
    }

    pub fn append_bitstream(&mut self, other: &Bitstream) {
        // FIXME: do this better
        for j in 0..other.pos {
            self.append(other.get(j));
        }
    }

    pub fn get(&self, pos: u32) -> u8 {
        let idx = (pos / 8) as usize;
        let bitidx = pos %8;

        let byte = self.data[idx];

        (byte & (1 << bitidx)) >> bitidx
    }

    pub fn pop(&mut self) -> Option<u8> {
        if self.pos == 0 {
            None
        } else {
            self.pos -= 1;
            Some(self.get(self.pos))
        }
    }

    pub fn reverse(&self) -> Bitstream {
        // FIXME: see if there's a better way to do this
        let mut bs = Bitstream::new();

        for i in 0..self.pos {
            bs.append(self.get(self.pos - i - 1));
        };

        bs
    }
}

impl Add for Bitstream {
    type Output = Bitstream;

    fn add(mut self, other: Bitstream) -> Bitstream {
        self.append_bitstream(&other);
        self
    }
}

impl<'a> Add<&'a Bitstream> for Bitstream {
    type Output = Bitstream;

    fn add(mut self, other: &'a Bitstream) -> Bitstream {
        self.append_bitstream(other);
        self
    }
}

impl Add<u8> for Bitstream {
    type Output = Bitstream;

    fn add(mut self, other: u8) -> Bitstream {
        self.append(other);
        self
    }
}

impl Display for Bitstream {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        for i in 0..self.pos {
            let bitstr = match self.get(i) {
                0 => "0",
                1 => "1",
                _ => panic!("Bad value from Bitstream.get"),
            };
            
            let _ = f.write_str(bitstr);
        };
        Ok(())
    }
}

impl Debug for Bitstream {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        (self as &Display).fmt(f)
    }
}
