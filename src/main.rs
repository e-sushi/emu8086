#![allow(bad_style)] // really dumb
#![allow(unused)] // rust seems to be unable to actually tell when something is used or unused
use std::{fs as filesystem, error::Error, io};
use fstrings::*;
use tui;
use crossterm;
use trees;

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

macro_rules! get8bitdisp { ($iter:ident) => {*$iter.next().unwrap() as u16} }
macro_rules! get16bitdisp { ($iter:ident) => {((*$iter.next().unwrap() as u16) | ((*$iter.next().unwrap() as u16) << 8))} }

struct vec2{
    x:f32,
    y:f32
}

impl std::ops::Add<vec2> for vec2 {
    type Output = vec2;
    fn add(self, rhs:vec2) -> Self::Output{
        vec2{x: self.x + rhs.x, y: self.y + rhs.y}
    }
}

fn main() {
    let buffer = {
        // read file
        let res = filesystem::read("data/listing_0040_challenge_movs");
        if res.is_err() {
            println!("could not open file due to: \"{}\"", res.err().unwrap());
            return;
        }
        // get resulting vec if file is read successfully
        res.unwrap()
    };
    
    //ui();

    const rmtable : [&str;8] = [
        "[bx + si",
        "[bx + di",
        "[bp + si",
        "[bp + di",
        "[si",
        "[di",
        "[bp",
        "[bx",
    ];

    fn get_value(w:bool, mut iter: &mut std::slice::Iter<u8>) -> u16 {
        if w {
            let lower = *iter.next().expect("unexpected eof. wanted a lower byte for mov:imm->reg") as u16;
            let upper = *iter.next().expect("unexpected eof. wanted an upper byte for mov:imm->reg") as u16;
            (upper << 8) | lower
        } else {
            *iter.next().expect("") as u16
        }
    }

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
            let w   = hasbits!(byte 0b00001000);
            out += &format!("mov {}, {}", 
                regtable[(w, getbits!(byte 0b00000111))],
                get_value(w, &mut iter,),
            );
        } else if hasbits!(byte 0b10001000) { // mov: register/memory to/from register
            out += "mov ";
            let d = hasbits!(byte 0b00000010);
            let w = hasbits!(byte 0b00000001);
            let byte2 = *iter.next().expect("unexpected eof. wanted byte 2 for mov:reg/mem<->reg");
            let mode = getbits!(byte2 0b11000000) >> 6;
            match mode {
                0b00 | 0b01 | 0b10 => { // memory mode, possibly 8/16 bit displacement follows
                    let regl = regtable[(w, getbits!(byte2 0b00111000) >> 3)];
                    if d {out += &f!("{regl}, ");}
                    let rm = getbits!(byte2 0b000000111);
                    // rust has NO way to break out of a match expr, so i guess we get to use an else here and nest everything
                    // one more time for no reason!
                    if mode == 0b00 && rm == 0b110 {
                        out += &format!("[{}", get_value(w, &mut iter));
                    } else {
                        out += rmtable[rm as usize];
                        if byte != 0 {
                            let value = get_value(if mode == 0b01 {false} else if mode == 0b10 {true} else {panic!()}, &mut iter) as i16;
                            if value != 0 {
                                out += &if value < 0 {
                                    format!(" - {}", value.abs())
                                } else {
                                    format!(" + {}", value)
                                }
                            }
                        } 
                    }
                    out += "]";
                    if !d {out += &f!(", {regl}");}
                }
                0b11 => { // register mode
                    let regl = regtable[(w,getbits!(byte2 0b00000111) >> 0)];
                    let regr = regtable[(w,getbits!(byte2 0b00111000) >> 3)];
                    out += &format!("{regl}, {regr}");
                }
                _ =>panic!()
            }
        } else if hasbits!(byte 0b11000110) { // mov: immediate to register/memory
            out += "mov ";
            let w = hasbits!(byte 0b00000001);
            let byte2 = *iter.next().expect("unexpected eof. wanted byte 2 for mov:imm->reg/mem");
            let mode = getbits!(byte2 0b11000000) >> 6;
            match mode {
                0b00 => out += &format!("{}], {} {}", 
                            rmtable[getbits!(byte2 0b00000111) as usize], 
                            if w{"word"}else{"byte"}, 
                            get_value(w,&mut iter)
                        ),
                0b01 => out += &format!("{}], {} {}", 
                            rmtable[getbits!(byte2 0b00000111) as usize],
                            if w {"word "}else{"byte "},
                            get_value(w, &mut iter)
                        ),
                0b10 => out += &format!("{}], {} {} {}", // so silly, delle would love this
                            rmtable[getbits!(byte2 0b00000111) as usize],
                            {
                                let value = get16bitdisp!(iter);
                                if value != 0 {
                                    format!(" + {}", value)
                                } else {
                                    String::from("")
                                }
                            },
                            if w {", word "} else {", byte "},
                            get_value(w, &mut iter)
                        ),
                _ =>panic!()
            }
        } else if hasbits!(byte 0b10100000) { // mov: memory to acculator or accumulator to memory
            out += "mov ";
            let w = hasbits!(byte 0b00000001);
            let mta = hasbits!(byte 0b00000010);
            if !mta { out += "ax, " } 
            out += &format!("[{}]", if w {get16bitdisp!(iter)} else {get8bitdisp!(iter)});
            if mta { out += ", ax" }
        } else if hasbits!(byte 0b10001100) { // mov: register/memory to segment register or vice versa
            todo!();
        }


        

        println!("{out}");
    }

}

fn ui() -> () {
    
    crossterm::terminal::enable_raw_mode().expect("crossterm backend failed to initialize");
    
    let mut stdout = std::io::stdout();
    let mut backend = tui::backend::CrosstermBackend::new(std::io::stdout());
    let mut terminal = tui::Terminal::new(backend).expect("tui terminal failed to initialize");

    let mut frame = terminal.get_frame();

    crossterm::execute!(
        stdout, 
        crossterm::terminal::EnterAlternateScreen, 
        crossterm::event::EnableMouseCapture
    );

    {use tui::widgets::*;
        terminal.draw(|f|{
            let size = f.size();
            let block = Block::default()
                    .title("8086")
                    .borders(Borders::ALL);
            f.render_widget(block,size);
        });
    }

}