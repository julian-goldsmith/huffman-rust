use std::ops::Add;
use std::fmt::*;
use std::result::Result;
use std::io;
use std::io::Write;
use std::io::Read;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

#[derive(Clone)]
pub struct Bitstream {
    pub pos: usize,
    data: Vec<u8>,
}

impl Bitstream {
    pub fn new() -> Bitstream {
        Bitstream { pos: 0, data: vec![0; 8] }
    }

    pub fn append(&mut self, val: u8) {
        if self.pos >= self.data.len() << 3 {
            self.data.push(0);
            self.data.push(0);
            self.data.push(0);
            self.data.push(0);
        }

        let idx = (self.pos >> 3) as usize;
        let bitidx = self.pos & 0x07;

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

    pub fn get(&self, pos: usize) -> u8 {
        let idx = pos >> 3;
        let bitidx = pos & 0x07;

        let byte = self.data[idx];

        (byte >> bitidx) & 1
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

    pub fn write(&self, writer: &mut Write) -> io::Result<usize> {
        // FIXME: make this less terrible
        let byte_len = (self.pos as f32 / 8.0).ceil() as usize;

        let bytes_out =
            match writer.write_u32::<BigEndian>(self.pos as u32) {
                Err(err) => return Err(err),
                Ok(_) => 4,
            }
            +
            match writer.write(&self.data[0..byte_len]) {
                Err(err) => return Err(err),
                Ok(nb) => nb,
            };
        Ok(bytes_out)
    }

    pub fn read(reader: &mut Read) -> io::Result<Bitstream> {
        let pos = match reader.read_u32::<BigEndian>() {
            Err(err) => return Err(err),
            Ok(pos) => pos as usize,
        };

        // FIXME: make this less terrible
        let byte_len = (pos as f32 / 8.0).ceil() as usize;

        let mut retval = Bitstream { pos, data: Vec::with_capacity(byte_len) };
        unsafe {
            retval.data.set_len(byte_len);
        };
        reader.read(&mut retval.data[0..byte_len])?;

        Ok(retval)
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
