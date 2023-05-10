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
fn main() {

    let buffer = &match filesystem::read("data/listing_0040_challenge_movs") {
        Ok(r) => r,
        Err(e) => {
            println!("could not open file due to: \"{}\"", e);
            return;
        }
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

    const srtable : [&str;4] = [
        "es",
        "cs",
        "ss",
        "ds",
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
        let mut byte = *{ // TODO(sushi) we probably don't need to keep making stuff like byte2 and byte3, we just extract all the info we need from byte before continuing
            let next = iter.next();
            if next == None {break}
            next.unwrap()
        };

        let mut out = String::from("");

        // counting from LEFT
        let b0 = (byte >> 7) & 1 == 1;
        let b1 = (byte >> 6) & 1 == 1;
        let b2 = (byte >> 5) & 1 == 1;
        let b3 = (byte >> 4) & 1 == 1;
        let b4 = (byte >> 3) & 1 == 1;
        let b5 = (byte >> 2) & 1 == 1;
        let b6 = (byte >> 1) & 1 == 1;
        let b7 = (byte >> 0) & 1 == 1;

        // expects that the current byte is the opcode byte
        fn eat_displacement(w:bool,d:bool,sr:bool,mut iter: &mut std::slice::Iter<u8>) -> String {
            let out = String::new();

            // let byte = *iter.next().unwrap();
            // let mode = getbits!(byte 0b11000000) >> 6;
            // let reg = if sr {
                
            // };

            out
        }

        // this humongous if statement is meant to act like a binary search for decoding
        // instructions. I didn't like the idea of have a very large if ladder for decoding the
        // first 4-6 bits of an instruction. I have no idea if this is more efficient or not, but
        // it's a fun way to do it regardless. 
        // the way I see it is that this should have complexity O(log(n)) for any instruction
        // whereas trying to decode with a huge if/else ladder would be a complexity of 
        // O(n). If we say that n is the number of possible opcodes uniquely identifying a
        // variant of an instruction, n would be 2^6 different possible instructions, which means that
        // in the worst case, O(n) would be 64 if statements, and of course the best being 1 if statement.
        // in this way you could organize the if/else ladder to put more common instructions first.
        // With a binary search sort of thing, log(2^6) is 1.8, but I'm not totally sure how to 
        // interpret this number. I think that with this, there should be at most 6 checks for
        // finding the proper instruction in any case. Even less for some instruction. I'm not totally
        // sure though. If it is log base 2, then yes it makes sense that it would result in 6
        if b0 { // ------------------------------------------------------------- 1xxxxxxx
            if b1 { // --------------------------------------------------------- 11xxxxxx
                if b2 { // ----------------------------------------------------- 111xxxxx
                    if b3 { // ------------------------------------------------- 1111xxxx
                        if b4 { // --------------------------------------------- 11111xxx
                            if b5 { // ----------------------------------------- 111111xx
                                if b6 { // ------------------------------------- 1111111x
                                    push_inc_dec_call_jmp_regmem(byte, &mut iter);
                                } else { // ------------------------------------ 1111110x
                                    if b7 { // --------------------------------- 11111101
                                        set_direction();
                                    } else { // -------------------------------- 11111100
                                        clear_direction();
                                    }
                                }
                            } else { // ---------------------------------------- 111110xx
                                if b6 { // ------------------------------------- 1111101x
                                    if b7 { // --------------------------------- 11111011
                                        set_interrupt();
                                    } else { // -------------------------------- 11111010
                                        clear_interrupt();
                                    }
                                } else { // ------------------------------------ 1111100x
                                    if b7 { // --------------------------------- 11111001
                                        set_carry();
                                    } else { // -------------------------------- 11111000
                                        clear_carry();
                                    }
                                }
                            }
                        } else { // -------------------------------------------- 11110xxx
                            if b5 { // ----------------------------------------- 111101xx
                                if b6 { // ------------------------------------- 1111011x
                                    neg_mul_div_not_test(byte, &mut iter);
                                } else { // ------------------------------------ 1111010x
                                    if b7 { // --------------------------------- 11110101
                                        complement_carry();
                                    } else { // -------------------------------- 11110100
                                        halt();
                                    }
                                }
                            } else { // ---------------------------------------- 111100xx
                                if b6 { // ------------------------------------- 1111001x
                                    if b7 { // --------------------------------- 11110011
                                        str_rep(byte);
                                    } else { // -------------------------------- 11110010
                                        todo!();
                                    }
                                } else { // ------------------------------------ 1111000x
                                    if b7 { // --------------------------------- 11110001
                                        todo!();
                                    } else { // -------------------------------- 11110000
                                        lock();
                                    }
                                }
                            }
                        }
                    } else { // ------------------------------------------------ 1110xxxx
                        if b4 { // --------------------------------------------- 11101xxx
                            if b5 { // ----------------------------------------- 111011xx
                                if b6 { // ------------------------------------- 1110111x
                                    out_variable(byte);
                                } else { // ------------------------------------ 1110110x
                                    in_variable(byte);
                                }
                            } else { // ---------------------------------------- 111010xx
                                if b6 { // ------------------------------------- 1110101x
                                    if b7 { // --------------------------------- 11101011
                                        jmp_direct_within_segment_short(&mut iter);
                                    } else { // -------------------------------- 11101010
                                        jmp_direct_intersegment(&mut iter);
                                    }
                                } else { // ------------------------------------ 1110100x
                                    if b7 { // --------------------------------- 11101001
                                        jmp_direct_within_segment(&mut iter);
                                    } else { // -------------------------------- 11101000
                                        call_direct_within_segment(&mut iter);
                                    }
                                }
                            }
                        } else { // -------------------------------------------- 11100xxx
                            if b5 { // ----------------------------------------- 111001xx
                                if b6 { // ------------------------------------- 1110011x
                                    out_fixed(byte, &mut iter);
                                } else { // ------------------------------------ 1110010x
                                   in_fixed(byte, &mut iter);
                                }
                            } else { // ---------------------------------------- 111000xx
                                if b6 { // ------------------------------------- 1110001x
                                    if b7 { // --------------------------------- 11100011
                                        jcxz(*iter.next().expect("wanted next byte for loop"));
                                    } else { // -------------------------------- 11100010
                                        loop_(*iter.next().expect("wanted next byte for loop"))
                                    }
                                } else { // ------------------------------------ 1110000x
                                    if b7 { // --------------------------------- 11100001
                                        loopz_loope(*iter.next().expect("wanted next byte for loop"));
                                    } else { // -------------------------------- 11100000
                                        loopnz_loopne(*iter.next().expect("wanted next byte for loop"));
                                    }
                                }
                            }
                        }
                    }
                } else { // ---------------------------------------------------- 110xxxxx
                    if b3 { // ------------------------------------------------- 1101xxxx
                        if b4 { // --------------------------------------------- 11011xxx
                            esc(byte, &mut iter);
                        } else { // -------------------------------------------- 11010xxx
                            if b5 { // ----------------------------------------- 110101xx
                                if b6 { // ------------------------------------- 1101011x
                                    if b7 { // --------------------------------- 11010111
                                        out_xlat();
                                    } else { // -------------------------------- 11010110
                                        todo!();
                                    }
                                } else { // ------------------------------------ 1101010x
                                    if b7 { // --------------------------------- 11010101
                                        cmp_aad(&mut iter);
                                    } else { // -------------------------------- 11010100
                                        cmp_aam(&mut iter);
                                    }
                                }
                            } else { // ---------------------------------------- 110100xx
                                shl_sal_shr_sar_rol_ror_rcl_rcr(byte, &mut iter);
                            }
                        }
                    } else { // ------------------------------------------------ 1100xxxx
                        if b4 { // --------------------------------------------- 11001xxx
                            if b5 { // ----------------------------------------- 110011xx
                                if b6 { // ------------------------------------- 1100111x
                                    if b7 { // --------------------------------- 11001111
                                        interrupt_return();
                                    } else { // -------------------------------- 11001110
                                        interrupt_on_overflow();
                                    }
                                } else { // ------------------------------------ 1100110x
                                    if b7 { // --------------------------------- 11001101
                                        interrupt_typed(*iter.next().expect("expected byte for interrupt."));
                                    } else { // -------------------------------- 11001100
                                        interrupt_type_3();
                                    }
                                }
                            } else { // ---------------------------------------- 110010xx
                                if b6 { // ------------------------------------- 1100101x
                                    if b7 { // --------------------------------- 11001011
                                        ret_intersegment();
                                    } else { // -------------------------------- 11001010
                                        ret_intersegment_add_imm_to_sp(&mut iter);
                                    }
                                } else { // ------------------------------------ 1100100x
                                    if b7 { // --------------------------------- 11001001
                                        todo!();
                                    } else { // -------------------------------- 11001000
                                        todo!();
                                    }
                                }
                            }
                        } else { // -------------------------------------------- 11000xxx
                            if b5 { // ----------------------------------------- 110001xx
                                if b6 { // ------------------------------------- 1100011x --- mov: imm->reg/mem
                                    mov_imm_to_regmem(byte, &mut iter);
                                } else { // ------------------------------------ 1100010x
                                    if b7 { // --------------------------------- 11000101
                                        out_lds(&mut iter);
                                    } else { // -------------------------------- 11000100
                                        out_les(&mut iter);
                                    }
                                }
                            } else { // ---------------------------------------- 110000xx
                                if b6 { // ------------------------------------- 1100001x
                                    if b7 { // --------------------------------- 11000011
                                        ret_within_segment();
                                    } else { // -------------------------------- 11000010
                                        ret_within_segment_add_imm_to_sp(&mut iter);
                                    }
                                } else { // ------------------------------------ 1100000x
                                    if b7 { // --------------------------------- 11000001
                                        todo!();
                                    } else { // -------------------------------- 11000000
                                        todo!();
                                    }
                                }
                            }
                        }
                    }
                }
            } else { // -------------------------------------------------------- 10xxxxxx
                if b2 { // ----------------------------------------------------- 101xxxxx
                    if b3 { // ------------------------------------------------- 1011xxxx --- mov: imm->reg
                        mov_imm_to_reg(byte, &mut iter);
                    } else { // ------------------------------------------------ 1010xxxx
                        if b4 { // --------------------------------------------- 10101xxx
                            if b5 { // ----------------------------------------- 101011xx
                                if b6 { // ------------------------------------- 1010111x
                                    str_scas(byte);
                                } else { // ------------------------------------ 1010110x
                                    str_lods(byte);
                                }
                            } else { // ---------------------------------------- 101010xx
                                if b6 { // ------------------------------------- 1010101x
                                    str_stds(byte);
                                } else { // ------------------------------------ 1010100x
                                    test_imm_and_acc(byte, &mut iter);
                                }
                            }
                        } else { // -------------------------------------------- 10100xxx
                            if b5 { // ----------------------------------------- 101001xx
                                if b6 { // ------------------------------------- 1010011x
                                    str_cmps(byte);
                                } else { // ------------------------------------ 1010010x
                                    str_movs(byte);
                                }
                            } else { // ---------------------------------------- 101000xx
                                if b6 { // ------------------------------------- 1010001x --- mov: acc->mem
                                    mov_acc_to_mem(byte, &mut iter);
                                } else { // ------------------------------------ 1010000x --- mov: mem->acc
                                    mov_mem_to_acc(byte, &mut iter);
                                }
                            }
                        }
                    }
                } else { // ---------------------------------------------------- 100xxxxx
                    if b3 { // ------------------------------------------------- 1001xxxx
                        if b4 { // --------------------------------------------- 10011xxx
                            if b5 { // ----------------------------------------- 100111xx
                                if b6 { // ------------------------------------- 1001111x
                                    if b7 { // --------------------------------- 10011111
                                        out_lahf();
                                    } else { // -------------------------------- 10011110
                                        out_sahf();
                                    }
                                } else { // ------------------------------------ 1001110x
                                    if b7 { // --------------------------------- 10011101
                                        out_popf();
                                    } else { // -------------------------------- 10011100
                                        out_pushf();
                                    }
                                }
                            } else { // ---------------------------------------- 100110xx
                                if b6 { // ------------------------------------- 1001101x
                                    if b7 { // --------------------------------- 10011011
                                        wait();
                                    } else { // -------------------------------- 10011010
                                        call_direct_intersegment(&mut iter);
                                    }
                                } else { // ------------------------------------ 1001100x
                                    if b7 { // --------------------------------- 10011001
                                        cmp_cwd();
                                    } else { // -------------------------------- 10011000
                                        cmp_cbw();
                                    }
                                }
                            }
                        } else { // -------------------------------------------- 10010xxx
                            xchg_reg_w_acc(byte);
                        }
                    } else { // ------------------------------------------------ 1000xxxx
                        if b4 { // --------------------------------------------- 10001xxx
                            if b5 { // ----------------------------------------- 100011xx
                                if b6 { // ------------------------------------- 1000111x
                                    if b7 { // --------------------------------- 10001111
                                        pop_regmem(&mut iter);
                                    } else { // -------------------------------- 10001110 --- mov: reg/mem->seg
                                        mov_regmem_to_seg(&mut iter);
                                    }   
                                } else { // ------------------------------------ 1000110x
                                    if b7 { // --------------------------------- 10001101
                                        out_lea(&mut iter);
                                    } else { // -------------------------------- 10001100 --- mov: seg->reg/mem
                                        mov_seg_to_regmem(&mut iter);
                                    }
                                }
                            } else { // ---------------------------------------- 100010xx --- mov: reg/mem<->reg
                                mov_regmem_tf_reg(byte, &mut iter);
                            }
                        } else { // -------------------------------------------- 10000xxx
                            if b5 { // ----------------------------------------- 100001xx
                                if b6 { // ------------------------------------- 1000011x
                                    xchg_regmem_w_reg(byte, &mut iter);
                                } else { // ------------------------------------ 1000010x
                                    if b7 { // --------------------------------- 10000101
                                        todo!();
                                    } else { // -------------------------------- 10000100
                                        todo!();
                                    }
                                }
                            } else { // ---------------------------------------- 100000xx 
                                add_adc_sub_sbb_cmp_and_or_imm_to_regmem(byte, &mut iter);
                            }
                        }
                    }
                }
            }
        } else { // ------------------------------------------------------------ 0xxxxxxx
            if b1 { // --------------------------------------------------------- 01xxxxxx
                if b2 { // ----------------------------------------------------- 011xxxxx
                    if b3 { // ------------------------------------------------- 0111xxxx
                        if b4 { // --------------------------------------------- 01111xxx
                            if b5 { // ----------------------------------------- 011111xx
                                if b6 { // ------------------------------------- 0111111x
                                    if b7 { // --------------------------------- 01111111
                                        jnle_jg(*iter.next().expect("wanted next byte for jump"));
                                    } else { // -------------------------------- 01111110
                                        jle_jng(*iter.next().expect("wanted next byte for jump"));
                                    }
                                } else { // ------------------------------------ 0111110x
                                    if b7 { // --------------------------------- 01111101
                                        todo!();
                                    } else { // -------------------------------- 01111100
                                        jl_jnge(*iter.next().expect("wanted next byte for jump"));
                                    }
                                }
                            } else { // ---------------------------------------- 011110xx
                                if b6 { // ------------------------------------- 0111101x
                                    if b7 { // --------------------------------- 01111011
                                        jnp_jpo(*iter.next().expect("wanted next byte for jump"));
                                    } else { // -------------------------------- 01111010
                                        jp_jpe(*iter.next().expect("wanted next byte for jump"));
                                    }
                                } else { // ------------------------------------ 0111100x
                                    if b7 { // --------------------------------- 01111001
                                        jns(*iter.next().expect("wanted next byte for jump"));
                                    } else { // -------------------------------- 01111000
                                        js(*iter.next().expect("wanted next byte for jump"));
                                    }
                                }
                            }
                        } else { // -------------------------------------------- 01110xxx
                            if b5 { // ----------------------------------------- 011101xx
                                if b6 { // ------------------------------------- 0111011x
                                    if b7 { // --------------------------------- 01110111
                                        jnbe_ja(*iter.next().expect("wanted next byte for jump"));
                                    } else { // -------------------------------- 01110110
                                        jbe_jna(*iter.next().expect("wanted next byte for jump"));
                                    }
                                } else { // ------------------------------------ 0111010x
                                    if b7 { // --------------------------------- 01110101
                                        jne_jnz(*iter.next().expect("wanted next byte for jump"));
                                    } else { // -------------------------------- 01110100
                                        je_jz(*iter.next().expect("wanted next byte for jump"));
                                    }
                                }
                            } else { // ---------------------------------------- 011100xx
                                if b6 { // ------------------------------------- 0111001x
                                    if b7 { // --------------------------------- 01110011
                                        jnb_jae(*iter.next().expect("wanted next byte for jump"));
                                    } else { // -------------------------------- 01110010
                                        jb_jnae(*iter.next().expect("wanted next byte for jump"));
                                    }
                                } else { // ------------------------------------ 0111000x
                                    if b7 { // --------------------------------- 01110001
                                        jno(*iter.next().expect("wanted next byte for jump"));
                                    } else { // -------------------------------- 01110000
                                        jo(*iter.next().expect("wanted next byte for jump"));
                                    }
                                }
                            }
                        }
                    } else { // ------------------------------------------------ 0110xxxx
                        if b4 { // --------------------------------------------- 01101xxx
                            if b5 { // ----------------------------------------- 011011xx
                                if b6 { // ------------------------------------- 0110111x
                                    if b7 { // --------------------------------- 01101111
                                        todo!();
                                    } else { // -------------------------------- 01101110
                                        todo!();
                                    }
                                } else { // ------------------------------------ 0110110x
                                    if b7 { // --------------------------------- 01101101
                                        todo!();
                                    } else { // -------------------------------- 01101100
                                        todo!();
                                    }
                                }
                            } else { // ---------------------------------------- 011010xx
                                if b6 { // ------------------------------------- 0110101x
                                    if b7 { // --------------------------------- 01101011
                                        todo!();
                                    } else { // -------------------------------- 01101010
                                        todo!();
                                    }
                                } else { // ------------------------------------ 0110100x
                                    if b7 { // --------------------------------- 01101001
                                        todo!();
                                    } else { // -------------------------------- 01101000
                                        todo!();
                                    }
                                }
                            }
                        } else { // -------------------------------------------- 01100xxx
                            if b5 { // ----------------------------------------- 011001xx
                                if b6 { // ------------------------------------- 0110011x
                                    if b7 { // --------------------------------- 01100111
                                        todo!();
                                    } else { // -------------------------------- 01100110
                                        todo!();
                                    }
                                } else { // ------------------------------------ 0110010x
                                    if b7 { // --------------------------------- 01100101
                                        todo!();
                                    } else { // -------------------------------- 01100100
                                        todo!();
                                    }
                                }
                            } else { // ---------------------------------------- 011000xx
                                if b6 { // ------------------------------------- 0110001x
                                    if b7 { // --------------------------------- 01100011
                                        todo!();
                                    } else { // -------------------------------- 01100010
                                        todo!();
                                    }
                                } else { // ------------------------------------ 0110000x
                                    if b7 { // --------------------------------- 01100001
                                        todo!();
                                    } else { // -------------------------------- 01100000
                                        todo!();
                                    }
                                }
                            }
                        }
                    }
                } else { // ---------------------------------------------------- 010xxxxx
                    if b3 { // ------------------------------------------------- 0101xxxx
                        if b4 { // --------------------------------------------- 01011xxx
                            pop_reg(byte, &mut iter);
                        } else { // -------------------------------------------- 01010xxx
                            push_reg(byte, &mut iter);
                        }
                    } else { // ------------------------------------------------ 0100xxxx
                        if b4 { // --------------------------------------------- 01001xxx
                            dec_reg(byte);
                        } else { // -------------------------------------------- 01000xxx
                            inc_reg(byte);
                        }
                    }
                }
            } else { // -------------------------------------------------------- 00xxxxxx
                if b2 { // ----------------------------------------------------- 001xxxxx
                    if b3 { // ------------------------------------------------- 0011xxxx
                        if b4 { // --------------------------------------------- 00111xxx
                            if b5 { // ----------------------------------------- 001111xx
                                if b6 { // ------------------------------------- 0011111x
                                    if b7 { // --------------------------------- 00111111
                                        cmp_aas();
                                    } else { // -------------------------------- 00111110
                                        todo!();
                                    }
                                } else { // ------------------------------------ 0011110x
                                    cmp_imm_and_acc(byte, &mut iter);
                                }
                            } else { // ---------------------------------------- 001110xx
                                cmp_regmem_and_reg(byte, &mut iter);
                            }
                        } else { // -------------------------------------------- 00110xxx
                            if b5 { // ----------------------------------------- 001101xx
                                if b6 { // ------------------------------------- 0011011x
                                    if b7 { // --------------------------------- 00110111
                                        inc_aaa();
                                    } else { // -------------------------------- 00110110
                                        todo!();
                                    }
                                } else { // ------------------------------------ 0011010x
                                    xor_imm_to_regmem_or_acc(byte, &mut iter);
                                }
                            } else { // ---------------------------------------- 001100xx
                                xor_regmem_and_reg_to_either(byte, &mut iter);
                            }
                        }
                    } else { // ------------------------------------------------ 0010xxxx
                        if b4 { // --------------------------------------------- 00101xxx
                            if b5 { // ----------------------------------------- 001011xx
                                if b6 { // ------------------------------------- 0010111x
                                    if b7 { // --------------------------------- 00101111
                                        cmp_das();
                                    } else { // -------------------------------- 00101110
                                        todo!();
                                    }
                                } else { // ------------------------------------ 0010110x
                                    sub_imm_from_acc(byte, &mut iter);
                                }
                            } else { // ---------------------------------------- 001010xx
                                sub_regmem_and_reg_to_either(byte, &mut iter);
                            }
                        } else { // -------------------------------------------- 00100xxx
                            if b5 { // ----------------------------------------- 001001xx
                                if b6 { // ------------------------------------- 0010011x
                                    if b7 { // --------------------------------- 00100111
                                        inc_daa();
                                    } else { // -------------------------------- 00100110
                                        todo!();
                                    }
                                } else { // ------------------------------------ 0010010x
                                    and_imm_to_acc(byte, &mut iter);
                                }
                            } else { // ---------------------------------------- 001000xx
                                and_regmem_with_reg_to_either(byte, &mut iter);
                            }
                        }
                    }
                } else { // ---------------------------------------------------- 000xxxxx
                    // special case where we must check the last 3 bits for a couple patterns
                    match getbits!(byte 0b00000111) {
                        0b110 => {
                            push_seg(byte, &mut iter);
                        }
                        0b111 => {
                            pop_seg(byte, &mut iter);
                        }
                        _ => {
                            if b3 { // ------------------------------------------------- 0001xxxx
                                if b4 { // --------------------------------------------- 00011xxx
                                    if b5 { // ----------------------------------------- 000111xx
                                        if b6 { // ------------------------------------- 0001111x
                                            if b7 { // --------------------------------- 00011111
                                                todo!();
                                            } else { // -------------------------------- 00011110
                                                todo!();
                                            }
                                        } else { // ------------------------------------ 0001110x
                                            sbb_imm_from_acc(byte, &mut iter);
                                        }
                                    } else { // ---------------------------------------- 000110xx
                                        sbb_regmem_from_reg_to_either(byte, &mut iter);
                                    }
                                } else { // -------------------------------------------- 00010xxx
                                    if b5 { // ----------------------------------------- 000101xx
                                        if b6 { // ------------------------------------- 0001011x
                                            if b7 { // --------------------------------- 00010111
                                                todo!();
                                            } else { // -------------------------------- 00010110
                                                todo!();
                                            }
                                        } else { // ------------------------------------ 0001010x
                                            adc_imm_to_acc(byte, &mut iter);
                                        }
                                    } else { // ---------------------------------------- 000100xx
                                        adc_regmem_w_reg_to_either(byte, &mut iter);
                                    }
                                }
                            } else { // ------------------------------------------------ 0000xxxx
                                if b4 { // --------------------------------------------- 00001xxx
                                    if b5 { // ----------------------------------------- 000011xx
                                        if b6 { // ------------------------------------- 0000111x
                                            if b7 { // --------------------------------- 00001111
                                                todo!();
                                            } else { // -------------------------------- 00001110
                                                todo!();
                                            }
                                        } else { // ------------------------------------ 0000110x
                                            if b7 { // --------------------------------- 00001101
                                                todo!();
                                            } else { // -------------------------------- 00001100
                                                todo!();
                                            }
                                        }
                                    } else { // ---------------------------------------- 000010xx
                                        if b6 { // ------------------------------------- 0000101x
                                            if b7 { // --------------------------------- 00001011
                                                todo!();
                                            } else { // -------------------------------- 00001010
                                                todo!();
                                            }
                                        } else { // ------------------------------------ 0000100x
                                            if b7 { // --------------------------------- 00001001
                                                todo!();
                                            } else { // -------------------------------- 00001000
                                                todo!();
                                            }
                                        }
                                    }
                                } else { // -------------------------------------------- 00000xxx
                                    if b5 { // ----------------------------------------- 000001xx
                                        if b6 { // ------------------------------------- 0000011x
                                            if b7 { // --------------------------------- 00000111
                                                todo!();
                                            } else { // -------------------------------- 00000110
                                                todo!();
                                            }
                                        } else { // ------------------------------------ 0000010x
                                            add_imm_to_acc(byte, &mut iter)
                                        }
                                    } else { // ---------------------------------------- 000000xx
                                        add_regmem_w_reg_to_either(byte, &mut iter);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        #[inline(always)]
        fn mov_regmem_tf_reg(byte: u8, mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn mov_imm_to_regmem(byte: u8, mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn mov_imm_to_reg(byte: u8, mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn mov_mem_to_acc(byte: u8, mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn mov_acc_to_mem(byte: u8, mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn mov_regmem_to_seg(mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn mov_seg_to_regmem(mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn push_inc_dec_call_jmp_regmem(byte: u8, mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn push_reg(byte: u8, mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn push_seg(byte: u8, mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn pop_regmem(mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn pop_reg(byte: u8, mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn pop_seg(byte: u8, mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn xchg_regmem_w_reg(byte: u8, mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn xchg_reg_w_acc(byte: u8) {
            todo!()
        }

        #[inline(always)]
        fn in_fixed(byte: u8, mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn in_variable(byte: u8) {
            todo!()
        }

        #[inline(always)]
        fn out_fixed(byte: u8, mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn out_variable(byte: u8) {
            todo!()
        }

        #[inline(always)]
        fn out_xlat() {
            todo!()
        }

        #[inline(always)]
        fn out_lea(mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn out_lds(mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn out_les(mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn out_lahf() {
            todo!()
        }

        #[inline(always)]
        fn out_sahf() {
            todo!()
        }

        #[inline(always)]
        fn out_pushf() {
            todo!()
        }

        #[inline(always)]
        fn out_popf() {
            todo!()
        }

        #[inline(always)]
        fn add_regmem_w_reg_to_either(byte: u8, mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn add_adc_sub_sbb_cmp_and_or_imm_to_regmem(byte: u8, mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }
        
        #[inline(always)]
        fn add_imm_to_acc(byte: u8, mut iter: &mut std::slice::Iter<u8>) {
           todo!()
        }

        #[inline(always)]
        fn adc_regmem_w_reg_to_either(byte: u8, mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn adc_imm_to_acc(byte: u8, mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }
        
        #[inline(always)]
        fn inc_regmem(byte: u8, mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn inc_reg(byte: u8) {
            todo!()
        }

        #[inline(always)]
        fn inc_aaa() {
            todo!()
        }

        #[inline(always)]
        fn inc_daa() {
            todo!()
        }

        #[inline(always)]
        fn sub_regmem_and_reg_to_either(byte: u8, mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn sub_or_sbb_imm_from_regmem(byte: u8, mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn sub_imm_from_acc(byte: u8, mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn sbb_regmem_from_reg_to_either(byte: u8, mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn sbb_imm_from_acc(byte: u8, mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn dec_reg(byte: u8) {
            todo!()
        }

        #[inline(always)]
        fn neg_mul_div_not_test(byte: u8, mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn cmp_regmem_and_reg(byte: u8, mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn cmp_imm_and_acc(byte: u8, mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn cmp_aas() {
            todo!()
        }

        #[inline(always)]
        fn cmp_das() {
            todo!()
        }

        #[inline(always)]
        fn cmp_aam(mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn cmp_aad(mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn cmp_cbw() {
            todo!()
        }

        #[inline(always)]
        fn cmp_cwd() {
            todo!()
        }

        #[inline(always)]
        fn not(byte: u8, mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn shl_sal_shr_sar_rol_ror_rcl_rcr(byte: u8, mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn and_regmem_with_reg_to_either(byte: u8, mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn and_imm_to_acc(byte: u8, mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn test_regmem_and_reg(byte: u8, mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn test_imm_and_acc(byte: u8, mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn xor_regmem_and_reg_to_either(byte: u8, mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn xor_imm_to_regmem_or_acc(byte: u8, mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn str_rep(byte: u8) {
            todo!()
        }
        
        #[inline(always)]
        fn str_movs(byte: u8) {
            todo!()
        }
        
        #[inline(always)]
        fn str_cmps(byte: u8) {
            todo!()
        }
        
        #[inline(always)]
        fn str_scas(byte: u8) {
            todo!()
        }
        
        #[inline(always)]
        fn str_lods(byte: u8) {
            todo!()
        }
        
        #[inline(always)]
        fn str_stds(byte: u8) {
            todo!()
        }

        #[inline(always)]
        fn call_direct_within_segment(mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn call_indirect_within_segment(mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn call_direct_intersegment(mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }
        
        #[inline(always)]
        fn call_indirect_intersegment(mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn jmp_direct_within_segment(mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn jmp_direct_within_segment_short(mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn jmp_direct_intersegment(mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn ret_within_segment() {
            todo!()
        }

        #[inline(always)]
        fn ret_within_segment_add_imm_to_sp(mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        #[inline(always)]
        fn ret_intersegment() {
            todo!()
        }

        #[inline(always)]
        fn ret_intersegment_add_imm_to_sp(mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        }

        // NOTE(sushi) the jump opcodes expect the byte to have been incremented already
        #[inline(always)]
        fn je_jz(byte: u8) {
            todo!()
        }

        #[inline(always)]
        fn jl_jnge(byte: u8) {
            todo!()
        }

        #[inline(always)]
        fn jle_jng(byte: u8) {
            todo!()
        }

        #[inline(always)]
        fn jb_jnae(byte: u8) {
            todo!()
        }

        #[inline(always)]
        fn jbe_jna(byte: u8) {
            todo!()
        }

        #[inline(always)]
        fn jp_jpe(byte: u8) {
            todo!()
        }

        #[inline(always)]
        fn jo(byte: u8) {
            todo!()
        }

        #[inline(always)]
        fn js(byte: u8) {
            todo!()
        }

        #[inline(always)]
        fn jne_jnz(byte: u8) {
            todo!()
        }

        #[inline(always)]
        fn jnl_jge(byte: u8) {
            todo!()
        }

        #[inline(always)]
        fn jnle_jg(byte: u8) {
            todo!()
        }

        #[inline(always)]
        fn jnb_jae(byte: u8) {
            todo!()
        }

        #[inline(always)]
        fn jnbe_ja(byte: u8) {
            todo!()
        }

        #[inline(always)]
        fn jnp_jpo(byte: u8) {
            todo!()
        }

        #[inline(always)]
        fn jno(byte: u8) {
            todo!()
        }

        #[inline(always)]
        fn jns(byte: u8) {
            todo!()
        }

        #[inline(always)]
        fn loop_(byte: u8) {
            todo!()
        }

        #[inline(always)]
        fn loopz_loope(byte: u8) {
            todo!()
        }

        #[inline(always)]
        fn loopnz_loopne(byte: u8) {
            todo!()
        }

        #[inline(always)]
        fn jcxz(byte: u8) {
            todo!()
        }

        #[inline(always)]
        fn interrupt_typed(byte: u8) {
            todo!()
        }

        #[inline(always)]
        fn interrupt_type_3() {
            todo!()
        }

        #[inline(always)]
        fn interrupt_on_overflow() {
            todo!()
        }

        #[inline(always)]
        fn interrupt_return() {
            todo!()
        }

        #[inline(always)]
        fn clear_carry() {
            todo!()
        }

        #[inline(always)]
        fn complement_carry() {
            todo!()
        }

        #[inline(always)]
        fn set_carry() {
            todo!()
        }

        #[inline(always)]
        fn clear_direction() {
            todo!()
        }

        #[inline(always)]
        fn set_direction() {
            todo!()
        }

        #[inline(always)]
        fn clear_interrupt() {
            todo!()
        }

        #[inline(always)]
        fn set_interrupt() {
            todo!()
        }

        #[inline(always)]
        fn halt() {
            todo!()
        }

        #[inline(always)]
        fn wait() {
            todo!()
        }

        #[inline(always)]
        fn esc(byte: u8, mut iter: &mut std::slice::Iter<u8>) {
            todo!()
        } 

        #[inline(always)]
        fn lock() {
            todo!()
        }

        #[inline(always)]
        fn segment(byte: u8) {
            todo!()
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