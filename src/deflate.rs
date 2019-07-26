use crate::error::Error;
use bitvec::prelude::*;
use hex_fmt::HexFmt;

struct BitReader<'a> {
    n: usize,
    buf: &'a [u8],
    slice: &'a BitSlice<LittleEndian>,
}

impl<'a> BitReader<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Self {
            n: 0,
            buf,
            slice: buf.as_bitslice(),
        }
    }

    fn align(self) -> &'a [u8] {
        let boundary = ((self.n + 8) % 8) / 8;
        &self.buf[boundary..]
    }

    fn read_bit(&mut self) -> usize {
        let res = self.slice[self.n];
        self.n += 1;
        if res {
            1
        } else {
            0
        }
    }

    fn read_2bits(&mut self) -> usize {
        self.read_bit() + (self.read_bit() << 1)
    }

    fn read_4bits(&mut self) -> usize {
        self.read_bit() + (self.read_bit() << 1) + (self.read_bit() << 2) + (self.read_bit() << 3)
    }

    fn read_5bits(&mut self) -> usize {
        self.read_bit()
            + (self.read_bit() << 1)
            + (self.read_bit() << 2)
            + (self.read_bit() << 3)
            + (self.read_bit() << 4)
    }
}

pub struct Reader {}

impl Reader {
    pub fn read(buf: &[u8]) -> Result<Vec<u8>, Error> {
        println!("start of slice = {}", HexFmt(&buf[..16]));
        let mut buf = buf;
        let mut reader = BitReader::new(buf);

        loop {
            let _bfinal = reader.read_bit();
            let btype = reader.read_2bits();

            match btype {
                0b00 => {
                    println!("no compression");
                    buf = reader.align();
                    println!("{:b}", buf[0]);
                    println!("{:x?}", &buf[1..5]);
                    let a = buf[1] as u16 + (buf[2] as u16) << 8;
                    let b = buf[3] as u16 + (buf[4] as u16) << 8;
                    println!("a = {}, b = {}, !b = {}", a, b, !b);

                    buf = &buf[4..];
                    reader = BitReader::new(buf);
                    unimplemented!();
                }
                0b01 => {
                    println!("fixed huffman");
                    let mut bits_per_value = [0usize; 288];
                    for i in 0..=143 {
                        bits_per_value[i] = 8;
                    }
                    for i in 144..=255 {
                        bits_per_value[i] = 9;
                    }
                    for i in 256..=279 {
                        bits_per_value[i] = 7;
                    }
                    for i in 280..=287 {
                        bits_per_value[i] = 8;
                    }

                    const MAX_BITS: usize = 14;

                    let mut bl_count = [0usize; MAX_BITS + 1];
                    for &bits in &bits_per_value[..] {
                        bl_count[bits] += 1;
                    }

                    let mut next_code = [0usize; MAX_BITS + 1];
                    let mut code = 0;
                    bl_count[0] = 0;
                    for bits in 1..=MAX_BITS {
                        code = (code + bl_count[bits - 1]) << 1;
                        next_code[bits] = code;
                    }

                    {
                        use tabular::{Row, Table};

                        struct Streak {
                            bits: usize,
                            lits: std::ops::RangeInclusive<usize>,
                            codes: std::ops::RangeInclusive<usize>,
                        };

                        let mut table = Table::new("{:>}    {:^}    {:<}");
                        table.add_row(
                            Row::new()
                                .with_cell("Lit Value")
                                .with_cell("Bits")
                                .with_cell("Codes"),
                        );
                        table.add_row(
                            Row::new()
                                .with_cell("---------")
                                .with_cell("----")
                                .with_cell("-----"),
                        );

                        let mut add_row = |streak: &Streak| {
                            use left_pad::leftpad_with;

                            let min_code = leftpad_with(
                                format!("{:b}", *streak.codes.start()),
                                streak.bits,
                                '0',
                            );
                            let max_code = leftpad_with(
                                format!("{:b}", *streak.codes.end()),
                                streak.bits,
                                '0',
                            );

                            table.add_row(
                                Row::new()
                                    .with_cell(format!(
                                        "{} - {}",
                                        streak.lits.start(),
                                        streak.lits.end()
                                    ))
                                    .with_cell(format!("{}", streak.bits))
                                    .with_cell(format!("{} through", min_code)),
                            );

                            table.add_row(
                                Row::new()
                                    .with_cell("")
                                    .with_cell("")
                                    .with_cell(format!("{}", max_code)),
                            );
                        };

                        let mut current: Option<Streak> = None;
                        for (lit, &bits) in bits_per_value.iter().enumerate() {
                            if let Some(current) = current.as_mut() {
                                if current.bits == bits {
                                    let code = next_code[bits];
                                    next_code[bits] += 1;
                                    current.codes = *current.codes.start()..=code;
                                    current.lits = *current.lits.start()..=lit;
                                    continue;
                                } else {
                                    add_row(current);
                                }
                            }

                            let code = next_code[bits];
                            next_code[bits] += 1;
                            current = Some(Streak {
                                bits,
                                lits: lit..=lit,
                                codes: code..=code,
                            })
                        }
                        if let Some(current) = current {
                            add_row(&current);
                        }

                        println!("{}", table)
                    }

                    unimplemented!();
                }
                0b10 => {
                    println!("dynamic huffman");
                    let hlit = reader.read_5bits() + 257;
                    let hdist = reader.read_5bits() + 1;
                    let hclen = reader.read_4bits() + 4;
                    dbg!(hlit);
                    dbg!(hdist);
                    dbg!(hclen);

                    unimplemented!();
                }
                0b11 => return Err(Error::InvalidBlockBtype11),
                _ => unreachable!(),
            }
        }

        unimplemented!()
    }
}
