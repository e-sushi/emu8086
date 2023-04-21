#![allow(bad_style)] // really dumb
#![allow(unused)] // rust seems to be unable to actually tell when something is used or unused
use std::{fs as filesystem, error::Error, fmt::format};
use fstrings::*;
use std::ops;

macro_rules! enum_display {
    ($enum_name:ident { $($variant_name:ident),+ $(,)? }) => {
        #[derive(Copy,Clone)] // necessary to use enum values in arrays
        enum $enum_name {
            $( $variant_name, )+
        }
        impl std::fmt::Display for $enum_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match *self {
                    $( $enum_name::$variant_name => write!(f, stringify!($variant_name)), )+
                }
            }
        }
    }
}

// how this is implemented is definitely overkill, but i want to explore how rust works
enum_display! {
    reg {
        ax, cx, dx, bx,
        ah, ch, dh, bh,
        al, cl, dl, bl,
        sp, bp, si, di,
    }
}

// HOLY SHIT THIS IS SO FUCKING STUPID!!!!!
struct RegTable {
    table: [[reg;8];2],
}

// I do not need to take in a tuple for this, I can just index the returned array
// but this is a more interesting use of rust
impl std::ops::Index<(bool, u8)> for RegTable {
    type Output = reg;

    fn index(&self, idx: (bool,u8)) -> &Self::Output {
        // you cannot index an array by a boolean, you cannot coerce a boolean to an integer
        // and you cannot implement the Index trait for arrays, so we have to use a match 
        // to properly index the table 
        match idx.0 {
            false => &self.table[0][idx.1 as usize],
            true => &self.table[1][idx.1 as usize]
        }
    }
}

const regtable : RegTable = RegTable{
    table: [
        [reg::al, reg::cl, reg::dl, reg::bl, reg::ah, reg::ch, reg::dh, reg::bh],
        [reg::ax, reg::cx, reg::dx, reg::bx, reg::sp, reg::bp, reg::si, reg::di]
    ]
};

// prevents from having to write out the value twice.
// extreme amount of syntax for something that is just 
// #define hasbits(bytes, value) bytes & value == value
// in C
macro_rules! hasbits { ($bytes:ident $value:expr) => { $bytes & $value == $value } }

// just a bit mask with &, but using a macro is more consistent with 'hasbits'
macro_rules! getbits { ($byte:ident $value:expr) => { $byte & $value } }


fn main() {
    let buffer = {
        // read file
        let res = filesystem::read("data/listing_0039_more_movs");
        if res.is_err() {
            println!("could not open file due to: \"{}\"", res.err().unwrap());
            return;
        }
        // get resulting vec if file is read successfully
        res.unwrap()
    };

    // use an iterator, because we may want to consume multiple bytes 
    // in a loop, but for loops don't allow that 
    let mut iter = buffer.iter();
    loop {
        let byte = *{ // TODO(sushi) we probably don't need to keep making stuff like byte2 and byte3, we just extract all the info we need from byte before continuing
            let next = iter.next();
            if next == None {break}
            next.unwrap()
        };
        let mut out = String::from("");

        if        hasbits!(byte 0b10110000) { // mov: immediate to register  
            out += "mov ";
            let w   = hasbits!(byte 0b00001000);
            let reg = regtable[(w,getbits!(byte 0b00000111))];
            let value = if w {
                let lower = *iter.next().expect("unexpected eof. wanted a lower byte for mov:imm->reg");
                let upper = *iter.next().expect("unexpected eof. wanted an upper byte for mov:imm->reg"); 
                ((upper as u16) << 8) | (lower as u16)
            }else{
                *iter.next().expect("unexpected eof. wanted immediate value for mov:imm->reg") as u16
            };
            out += &f!("{reg}, {value}");
        } else if hasbits!(byte 0b10001000) { // mov: register/memory to/from register
            out += "mov ";
            let d = hasbits!(byte 0b00000010); 
            todo!(); // TODO(sushi) handle d 
            let w = hasbits!(byte 0b00000001);
            let byte2 = *iter.next().expect("unexpected eof. wanted byte 2 for mov:reg/mem<->reg");
            let mode = getbits!(byte2 0b11000000) >> 6;
            match mode {
                0b00 => { // memory mode 
                    // TODO(sushi) combine with following case
                    let regl = regtable[(w, getbits!(byte2 0b00111000) >> 3)];
                    let rm = getbits!(byte2 0b00000111);
                    out += &f!("{regl}, ");
                    out += &match rm {
                        0b000 => f!("[bx + si]"),
                        0b001 => f!("[bx + di]"),
                        0b010 => f!("[bp + si]"),
                        0b011 => f!("[bp + di]"),
                        0b100 => f!("[si]"),
                        0b101 => f!("[di]"),
                        0b111 => f!("[bx]"),
                        0b110 => {
                            todo!();
                        }
                        _=>todo!()
                    }
                }
                0b01 | 0b10 => { // memory mode, 8/16 bit displacement follows
                    let regl = regtable[(w, getbits!(byte2 0b00111000) >> 3)];
                    out += &f!("{regl}, ");
                    let rm = getbits!(byte2 0b000000111);
                    out += &match rm {
                        0b000 => {"[bx + si"}
                        0b001 => {"[bx + di"}
                        0b010 => {"[bp + si"}
                        0b011 => {"[bp + di"}
                        0b100 => {"[si"}
                        0b101 => {"[di"}
                        0b110 => {"[bp"}
                        0b111 => {"[bx"}
                        _=>panic!()
                    };
                    if byte != 0 {
                        out += &match mode {
                            0b01 => {
                                let byte3 = *iter.next().expect("unexpected eof. wanted byte 3 for mov:reg/mem<->reg");
                                if byte3 != 0 {
                                    format!(" + {}]", byte3.to_string())
                                } else {
                                    String::from("]")
                                }
                            }
                            0b10 =>{
                                let lower = *iter.next().expect("unexpected eof. wanted lower byte for mov:reg/mem<->reg");
                                let upper = *iter.next().expect("unexpected eof. wanted upper byte for mov:reg/mem<->reg");
                                let value = ((upper as u16) << 8) | (lower as u16);
                                if value != 0 {
                                    format!(" + {}]", value.to_string())
                                } else {
                                    String::from("]")
                                }
                            }
                            _=>panic!()
                        };
                    } 
                }
                0b11 => { // register mode
                    let regl = regtable[(w,getbits!(byte2 0b00000111) >> 0)];
                    let regr = regtable[(w,getbits!(byte2 0b00111000) >> 3)];
                    out += &format!("{regl}, {regr}");
                }
                
                _ => todo!()
            }
        } else if hasbits!(byte 0b11000110) { // mov: immediate to register/memory
            todo!();
        } else if hasbits!(byte 0b10100000) { // mov: memory to acculator or accumulator to memory
            todo!();
        } else if hasbits!(byte 0b10001100) { // mov: register/memory to segment register or vice versa
            todo!();
        }


        println!("{out}");
    }

}
