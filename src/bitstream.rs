use std::ops::{Add,Range};
use std::fmt::*;
use std::result::Result;
use std::io;
use std::io::Write;
use std::io::Read;
use std::iter;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

#[derive(Clone)]
pub struct Bitstream {
    pos: Range<usize>,
    data: Vec<u8>,
}

impl Bitstream {
    pub fn new() -> Bitstream {
        Bitstream { pos: (0..0), data: vec![0; 8] }
    }

    #[inline(always)]
    fn get_indices(pos: usize) -> (usize, usize) {
        let idx = (pos >> 3) as usize;
        let bitidx = pos & 0x07;

        (idx, bitidx)
    }

    fn extend(&mut self) {
        let add_len = self.data.len() / 2;
        self.data.extend(iter::repeat(0).take(add_len));
    }

    pub fn append(&mut self, val: u8) {
        let (idx, bitidx) = Bitstream::get_indices(self.pos.end);

        if idx >= self.data.len() {
            self.extend();
        };

        self.data[idx] = 
            self.data[idx] 
                & !(1 << bitidx) 
                | ((val & 1) << bitidx);
        self.pos.end += 1;
    }

    pub fn append_bitstream(&mut self, other: &Bitstream) {
        // FIXME: do this better
        for j in other.pos.clone() {
            self.append(other.get(j));
        }
    }

    #[inline]
    fn get(&self, pos: usize) -> u8 {
        let (idx, bitidx) = Bitstream::get_indices(pos);

        let byte = self.data[idx];

        (byte >> bitidx) & 1
    }

    pub fn pop(&mut self) -> Option<u8> {
        if self.pos.end == 0 {
            None
        } else {
            self.pos.end -= 1;
            Some(self.get(self.pos.end))
        }
    }

    pub fn pop_start(&mut self) -> Option<u8> {
        if self.pos.start == self.pos.end {
            None
        } else {
            self.pos.start += 1;
            Some(self.get(self.pos.start - 1))
        }
    }

    fn ceil_div(num: usize, denom: usize) -> usize {
        (num + denom - 1) / denom
    }

    pub fn write(&self, writer: &mut Write) -> io::Result<usize> {
        // FIXME: make this less terrible
        let byte_len = Bitstream::ceil_div(self.pos.end, 8);

        // TODO: handle this
        assert_eq!(self.pos.start, 0);

        let bytes_out =
            match writer.write_u32::<BigEndian>(self.pos.end as u32) {
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

    pub fn read(reader: &mut Read) -> io::Result<Option<Bitstream>> {
        let pos = match reader.read_u32::<BigEndian>() {
            Err(err) => return Ok(None),        // FIXME: handle non-EOF
            Ok(pos) => pos as usize,
        };

        // FIXME: make this less terrible
        let byte_len = (pos as f32 / 8.0).ceil() as usize;

        let mut retval = Bitstream { pos: (0..pos), data: Vec::with_capacity(byte_len) };
        unsafe {
            retval.data.set_len(byte_len);
        };
        reader.read(&mut retval.data[0..byte_len])?;

        Ok(Some(retval))
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
        for i in self.pos.clone() {
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
