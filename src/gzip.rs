use crate::{error::Error, parse};
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

        let res = crate::deflate::Reader::read(buf)?;
        Ok(res)
    }
}

#[derive(Debug)]
pub struct Header {
    method: Method,
    flags: Flags,
    mtime: u32,
    extra_flags: ExtraFlags,
    os: OS,
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
            tuple((
                Method::parse,
                Flags::parse,
                le_u32,
                ExtraFlags::parse,
                OS::parse,
            )),
        );
        let (i, (method, flags, mtime, extra_flags, os)) = mf(i)?;

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
                extra_flags,
                os,
                extra: extra.map(|x| x.to_owned()),
                name: name.map(|x| x.to_owned()),
                comment: comment.map(|x| x.to_owned()),
                crc16,
            },
        )(i)
    }
}

#[derive(Debug)]
pub struct Trailer {
    crc32: u32,
    input_size: u32,
}

impl Trailer {
    pub fn parse<'a>(i: &'a [u8]) -> parse::Result<'a, Self> {
        map(tuple((le_u32, le_u32)), |(crc32, input_size)| Self {
            crc32,
            input_size,
        })(i)
    }
}

/// Identifies the compression method used in the file.
/// 0-7 are reserved, 8 denotes "Deflate", customarily
/// used by gzip.
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

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum OS {
    Fat,
    Amiga,
    Vms,
    Unix,
    VmCms,
    AtariTos,
    Hpfs,
    Macintosh,
    ZSystem,
    CpM,
    Tops20,
    Ntfs,
    Qdos,
    AcornRiscOs,
    Unknown,
    Other(u8),
}

impl OS {
    pub fn parse<'a>(i: &'a [u8]) -> parse::Result<'a, Self> {
        map(le_u8, |u| u.into())(i)
    }
}

impl From<u8> for OS {
    fn from(u: u8) -> Self {
        match u {
            0 => OS::Fat,
            1 => OS::Amiga,
            2 => OS::Vms,
            3 => OS::Unix,
            4 => OS::VmCms,
            5 => OS::AtariTos,
            6 => OS::Hpfs,
            7 => OS::Macintosh,
            8 => OS::ZSystem,
            9 => OS::CpM,
            10 => OS::Tops20,
            11 => OS::Ntfs,
            12 => OS::Qdos,
            13 => OS::AcornRiscOs,
            255 => OS::Unknown,
            n => OS::Other(n),
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct Flags(u8);

impl Flags {
    pub const FTEXT: Flags = Flags(0b1);
    pub const FHCRC: Flags = Flags(0b10);
    pub const FEXTRA: Flags = Flags(0b100);
    pub const FNAME: Flags = Flags(0b1000);
    pub const FCOMMENT: Flags = Flags(0b10000);

    fn has(self, flag: Flags) -> bool {
        self & flag != Flags(0)
    }

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

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct ExtraFlags(u8);

impl ExtraFlags {
    pub const SLOWEST: ExtraFlags = ExtraFlags(0b10);
    pub const FASTEST: ExtraFlags = ExtraFlags(0b100);

    fn has(self, flag: ExtraFlags) -> bool {
        self & flag != ExtraFlags(0)
    }

    pub fn parse<'a>(i: &'a [u8]) -> parse::Result<'a, Self> {
        map(le_u8, |u| u.into())(i)
    }
}

impl From<u8> for ExtraFlags {
    fn from(u: u8) -> Self {
        Self(u)
    }
}

impl fmt::Debug for ExtraFlags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ExtraFlags(")?;
        let mut first = true;
        let mut item = |s: &str| {
            if first {
                first = false;
                write!(f, "{}", s)
            } else {
                write!(f, ", {}", s)
            }
        };

        if self.has(ExtraFlags::SLOWEST) {
            item("SLOWEST")?;
        }
        if self.has(ExtraFlags::FASTEST) {
            item("FASTEST")?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

impl std::ops::BitAnd for ExtraFlags {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}
