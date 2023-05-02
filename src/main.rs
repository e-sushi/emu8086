#![allow(bad_style)] // really dumb
#![allow(unused)] // rust seems to be unable to actually tell when something is used or unused
#![feature(iter_intersperse)]
use std::{fs as filesystem, error::Error, io, rc::Rc, cell::RefCell};
use fstrings::*;
use tui;
use crossterm;
use itertools::Itertools;

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

// fn parse<'a>(path:String) -> trees::Tree<Type<'a>> {
//     use trees::*;
//     let mut buffer = {
//         let res = filesystem::read(path);
//         if res.is_err() {
//             println!("unable to open file due to: \"{}\"", res.err().unwrap());
//             panic!();
//         }
//         String::from_utf8(res.unwrap()).unwrap()
//     };
//     let mut chars = buffer.chars().peekable();
    
//     fn eat_word(mut iter: &mut std::iter::Peekable<std::str::Chars>) -> String {
//         let mut out = String::new();
//         loop {
//             match iter.peek() {
//                 Some(c) => {
//                     if !c.is_alphanumeric() {break;}
//                     out.push(iter.next().unwrap())
//                 }
//                 None => {println!("unexpected end of file."); panic!();}
//             }   
//         }
//         out
//     }
    


//     fn eat_whitespace(mut iter: &mut std::iter::Peekable<std::str::Chars>) {
//         match iter.peek() {
//             Some(c) => {if c.is_whitespace() {return;}}
//             None => {println!("unexpected eof"); panic!();}
//         }
//         loop {
//             match iter.next(){
//                 Some(c) => {if c.is_whitespace() {break;}}
//                 None => { println!("unexpected eof"); panic!(); }
//             }
//         }
//     }

//     enum Node {
//         next(Rc<Node>),
//         prev(Rc<Node>),
//         first_child(Rc<Node>)

//     }

//     enum Token<'a> {
//         BitStream(&'a str),
//         Conditional,
//         QuestionMark,
//         Equals,
//         OrBar,
//         OpenBrace,
//         CloseBrace,
//         Number(u8),
//         Identifier(&'a str),
//     }

//     impl<'a> std::fmt::Display for Token<'a> {
//         fn fmt(&self, f: &mut rust::Formatter<'_>) -> std::fmt::Result {
//             match self {
//                 Token::BitStream(s) => write!(f, "BitStream: {}", s),
//                 Token::Conditional => write!(f, "Conditional"),
//                 Token::QuestionMark => write!(f, "QuestionMark"),
//                 Token::Equals => write!(f, "Equals"),
//                 Token::OrBar => write!(f, "OrBar"),
//                 Token::OpenBrace => write!(f, "OpenBrace"),
//                 Token::CloseBrace => write!(f, "CloseBrace"),
//                 Token::Number(n) => write!(f, "Number: {}", n),
//                 Token::Identifier(str) => write!(f, "Identifier: {}", str)
//             }
//         }
//     }

    // enum ParseError<'a> {
    //     InvalidToken(Token<'a>, Token<'a>),
    //     NumberTooLarge(u64),
    // }

//     impl<'a> std::fmt::Display for ParseError<'a> {
//         fn fmt(&self, f: &mut rust::Formatter<'_>) -> std::fmt::Result {
//             use ParseError::*;
//             match self {
//                 InvalidToken(got,wanted) => write!(f, "InvalidToken: wanted {}, got {}", wanted, got),
//                 NumberTooLarge(n) => write!(f, "NumberTooLarge: got {}, max is 255", n)
//             }
//         }
//     }

//     let blocks = buffer.split(&['(',')']).filter(|s|!s.is_empty());

//     let mut out = Tree::new(Type::root);
//     for mut chunk in &blocks.chunks(2) {
//         let name = chunk.next().expect("FUCK!!!").trim();
//         let lines = chunk.next().expect("RAAGAAGHAGAHGAH!!!!").lines();
//         for line in lines {
//             let mut parts = line.split(',');
//             let mut tokens : Vec<Token> = Vec::new();
//             for part in parts{
//                 if part.starts_with(&['1','0']) {
                    
//                 }
//             }
//             let out = parse_line(&tokens.iter());
//             match out {
//                 Ok(r) => {

//                 }
//                 Err(e) => panic!("{}", e)
//             }
//         }
//     }

//     fn parse_line<'a>(mut root: &Tree<Type>, mut tokens: &std::slice::Iter<Token>) -> Result<Tree<Type<'a>>, ParseError<'a>>{
//         let stream = tokens.next().expect("");
//         match stream {
//             Token::BitStream(s) => {
//                 let mut chars = s.chars();
//                 let first = chars.next().unwrap();
//                 let mut out = Tree::new(if first == '0' {Type::bit(false)} else {Type::bit(true)});
//                 for c in chars {
//                     match c {
//                         '0' => {
//                             out.push_back(Tree::new(Type::bit(false)));
//                             out = out.
//                         }
//                         '1' => out.push_back(Tree::new(Type::bit(true)))
//                     }
//                 }
//                 return Ok(out);
//             }
//             e => return Err(ParseError::InvalidToken(*e, Token::BitStream("")))
//         }

//     }

//     fn parse_part(part: &str){
//         if part.chars().all(|c|(c=='0'||c=='1')) {
//             for char in part.chars() {
//                 tree.push_back(Tree::new(Type::bit(char == '1')));
//             }
//         } else if part.starts_with('?') {
//             parse_conditional(part, tree);
//         }
//     }

//     fn parse_conditional(cond: &str, mut tree: &mut Tree<Type>){
//         if cond.starts_with("?") {
//             parse_or(&cond[1..cond.find('{').expect("expected '{' for conditional")], tree);
//         } else {
//             parse_factor(cond, tree);
//         }
//     }

//     fn parse_or(or: &str, mut tree: &mut Tree<Type>){
//         let last : Option<Node<Type>> = None;
//         for s in or.split('|') {
//             parse_equality(s, tree);
//             // match last {
//             //     Some(mut t) => {
//             //         let mut ornode = Tree::new(Type::or);
//             //         ornode.push_back(t.detach());
//             //         ornode.push_back(tree.pop_back().expect("where is that damn node"));
//             //         tree.push_back(ornode)
//             //     }
//             //     None => {}
//             // }
//         }
//         parse_equality(or, tree);
//     }

//     fn parse_equality(equal: &str, mut tree: &mut Tree<Type>) {

//     }

//     fn parse_factor(factor: &str, mut tree: &mut Tree<Type>){
//         if factor.starts_with(|c:char|c.is_digit(10)) {
//             tree.push_back(
//                 Tree::new(
//                     Type::number(
//                         u16::from_str_radix(factor, 10).expect("failed to turn factor into u8")
//                     )
//                 )
//             );
//         } else {
//             // tree.push_back(
//             //     Tree::new(
//             //         Type::id(factor)
//             //     )
//             // );
//         }
//     }

//     fn parse_id(id: &str, mut tree: &mut Tree<Type>){
//         match id {
//             "d"   => tree.push_back(Tree::new(Type::d)),
//             "w"   => tree.push_back(Tree::new(Type::w)),
//             "rm"  => tree.push_back(Tree::new(Type::rm)),
//             "mod" => tree.push_back(Tree::new(Type::mode)),
//             "data-lo" | "data-hi" | "disp-lo" | "disp-hi" =>
//                 tree.push_back(Tree::new(Type::byte)),
//             _ => todo!()
//         }
//     }



//     let mut eat_group_name = true;
//     loop {
//         if(eat_group_name){
//             let name = {
//                 let out = eat_word(&mut chars);
//                 eat_whitespace(&mut chars);
//                 let c = chars.next();
//                 match c {
//                     Some('{') => { eat_group_name = false; continue; }
//                     Some(_) => {
//                         println!("expected '{{' after group name.");
//                         panic!();
//                     }
//                     None => {
//                         println!("unexpected eof.");
//                         panic!();
//                     }
//                 }
//             };
//         }else{
//             match chars.next() {
//                 Some(c) => {
//                     match c {
//                         '0' => out.push_back(Tree::new(Type::bit(false))),
//                         '1' => out.push_back(Tree::new(Type::bit(true))),
//                          _  => {
//                             if c.is_alphabetic() {
//                                 let word = eat_word(&mut chars);
//                                 match word.as_str() {
//                                     "d" => out.push_back(Tree::new(Type::d)),
//                                     "w" => out.push_back(Tree::new(Type::w)),
//                                     "mod" => out.push_back(Tree::new(Type::mode)),
//                                     "reg" => out.push_back(Tree::new(Type::reg)),
//                                     "data-lo" => out.push_back(Tree::new(Type::byte)),

                                    
//                                     _ => {
//                                         println!("unexpected string.");
//                                         panic!();
//                                     }
//                                 }
//                             }
//                         }
//                     }
//                 }
//                 None => {
//                     println!("unexpected eof."); panic!();
//                 }
//             }
//         }
//     }
// }

fn main() {

    test();
    // let tree = parse(String::from("src/instruction_table.sh"));

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
        
    let mut iter = buffer.iter();
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
    loop {
        let byte = *{ // TODO(sushi) we probably don't need to keep making stuff like byte2 and byte3, we just extract all the info we need from byte before continuing
            let next = iter.next();
            if next == None {break}
            next.unwrap()
        };
        let mut out = String::from("");
        // these if statements can probably be replaced by binary searching the bits 
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