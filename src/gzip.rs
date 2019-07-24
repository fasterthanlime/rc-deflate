use crate::{error::Error, parse};
// use bitvec::prelude::*;
use nom::{
    bytes::streaming::{tag, take_till},
    combinator::{cond, map},
    multi::length_data,
    number::streaming::{le_u16, le_u32, le_u8},
    sequence::{preceded, terminated, tuple},
};
use std::fmt;

pub struct Reader {}

impl Reader {
    pub fn read(buf: &[u8]) -> Result<Vec<u8>, Error> {
        let (buf, header) = match Header::parse(buf) {
            Ok(r) => r,
            Err(nom::Err::Failure(err)) | Err(nom::Err::Error(err)) => Err(err)?,
            Err(nom::Err::Incomplete(_)) => Err(Error::IncompleteInput)?,
        };
        println!("header = {:#?}", header);

        // let slice: &BitSlice = (&buf[..]).as_bitslice::<BigEndian>();
        // println!("first few bits = {:?}", &slice[..8]);

        unimplemented!()
    }
}

#[derive(Debug)]
pub struct Header {
    method: Method,
    flags: Flags,
    mtime: u32,
    xfl: u8,
    os: u8,
    extra: Option<Vec<u8>>,
    name: Option<Vec<u8>>,
    comment: Option<Vec<u8>>,
    crc16: Option<u16>,
}

impl Header {
    const SIGNATURE: [u8; 2] = [0x1f, 0x8b];

    pub fn parse<'a>(i: &'a [u8]) -> parse::Result<'a, Self> {
        let mf = preceded(
            tag(Self::SIGNATURE),
            tuple((Method::parse, Flags::parse, le_u32, le_u8, le_u8)),
        );
        let (i, (method, flags, mtime, xfl, os)) = mf(i)?;

        let null_terminated = || terminated(take_till(|b| b == 0), tag(&[0u8]));

        map(
            tuple((
                cond(flags.has(Flags::FEXTRA), length_data(le_u16)),
                cond(flags.has(Flags::FNAME), null_terminated()),
                cond(flags.has(Flags::FCOMMENT), null_terminated()),
                cond(flags.has(Flags::FHCRC), le_u16),
            )),
            move |(extra, name, comment, crc16)| Self {
                method,
                flags,
                mtime,
                xfl,
                os,
                extra: extra.map(|x| x.to_owned()),
                name: name.map(|x| x.to_owned()),
                comment: comment.map(|x| x.to_owned()),
                crc16,
            },
        )(i)
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum Method {
    Deflate,
    Unknown(u8),
}

impl Method {
    pub fn parse<'a>(i: &'a [u8]) -> parse::Result<'a, Self> {
        map(le_u8, |u| u.into())(i)
    }
}

impl From<u8> for Method {
    fn from(u: u8) -> Self {
        match u {
            8 => Method::Deflate,
            n => Method::Unknown(n),
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct Flags(u8);

impl Flags {
    pub const FTEXT: Flags = Flags(0b1);
    pub const FHCRC: Flags = Flags(0b01);
    pub const FEXTRA: Flags = Flags(0b001);
    pub const FNAME: Flags = Flags(0b0001);
    pub const FCOMMENT: Flags = Flags(0b00001);

    fn has(self, flag: Flags) -> bool {
        self & flag != Flags(0)
    }
}

impl Flags {
    pub fn parse<'a>(i: &'a [u8]) -> parse::Result<'a, Self> {
        map(le_u8, |u| u.into())(i)
    }
}

impl From<u8> for Flags {
    fn from(u: u8) -> Self {
        Self(u)
    }
}

impl fmt::Debug for Flags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Flags(")?;
        let mut first = true;
        let mut item = |s: &str| {
            if first {
                first = false;
                write!(f, "{}", s)
            } else {
                write!(f, ", {}", s)
            }
        };

        if self.has(Flags::FTEXT) {
            item("TEXT")?;
        }
        if self.has(Flags::FHCRC) {
            item("HCRC")?;
        }
        if self.has(Flags::FEXTRA) {
            item("EXTRA")?;
        }
        if self.has(Flags::FNAME) {
            item("NAME")?;
        }
        if self.has(Flags::FCOMMENT) {
            item("COMMENT")?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

impl std::ops::BitAnd for Flags {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}
