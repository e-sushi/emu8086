#![allow(bad_style)] // really dumb
#![allow(unused)] // rust seems to be unable to actually tell when something is used or unused
use std::fs as filesystem;
use fstrings::*;
use std::ops;

// because rust is dumb, this macro is used to generate a function that converts integers to enum values
// while the reason for needing this is stupid, i think the fact that this is possible is really cool
macro_rules! enum_from_int {
    ($enum_name:ident { $($variant_name:ident = $variant_value:expr),+ $(,)? }) => {
        #[derive(Debug)]
        enum $enum_name {
            $($variant_name = $variant_value),+
        }

        // implement getting an enum value from a number
        impl $enum_name {
            fn from(value: u8) -> Option<Self> {
                match value {
                    $( $variant_value => Some($enum_name::$variant_name), )+
                    _ => None,
                }
            }
        }

        impl std::fmt::Display for $enum_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match *self {
                    $( $enum_name::$variant_name => write!(f, stringify!($variant_name)), )+
                }
            }
        }
    };
}

macro_rules! enum_display {
    ($enum_name:ident { $($variant_name:ident),+ $(,)? }) => {
        #[derive(Copy,Clone)]
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
        let byte = *{
            let next = iter.next();
            if next == None {break}
            next.unwrap()
        };
        let mut out = String::from("");

        if        hasbits!(byte 0b10110000) { // mov: immediate to register  
            out += "mov ";
            let w   = hasbits!(byte 0b00001000);
            let reg = regtable[(w,getbits!(byte 0b00000111))];
            out += &f!("{reg}, ");
            let value = if w {
                let upper = *iter.next().expect("unexpected eof. wanted an upper byte for mov:imm->reg"); 
                let lower = *iter.next().expect("unexpected eof. wanted a lower byte for mov:imm->reg");
                ((upper as u16) << 8) | (lower as u16)
            }else{
                *iter.next().expect("unexpected eof. wanted immediate value for mov:imm->reg") as u16
            };
            out += &f!("{value}");
        } else if hasbits!(byte 0b10001000) { // mov: register/memory to/from register
            out += "mov ";
            let d = hasbits!(byte 0b00000010);
            let w = hasbits!(byte 0b00000001);
            let byte2 = *iter.next().expect("unexpected eof. wanted byte 2 for mov:reg/mem<->reg");
            let mode = getbits!(byte2 0b11000000) >> 6;
            match mode {
                0b11 => { // we are only dealing with registers 
                    let regl = regtable[(w,getbits!(byte2 0b00000111) >> 0)];
                    let regr = regtable[(w,getbits!(byte2 0b00111000) >> 3)];
                    out += &format!("{regl}, {regr}");
                }
                0b10 => {
                    let regl = regtable[(w, getbits!(byte2 0b00111000) >> 3)];
                    let rm = getbits!(byte2 0b00000111) >> 3;
                    match rm {
                        0b000 => {
                            // RETURN HERE!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
                        }
                        _=>todo!()
                    }

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
