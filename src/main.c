#include <stdio.h>
#include <stdlib.h>

#define DEBUG_PRINT 1

typedef unsigned char u8;
typedef unsigned short u16;
typedef unsigned long u64;

typedef signed char s8;
typedef signed short s16;
typedef signed long s64;

#define hasbits(b, m) (((b) & (m)) == (m))
#define getbits(b, m) ((b) & (m))

u8* stream;
u8 memory[0xffffffff];

typedef struct reg {
    union{
        u16 x;
        struct{
            u8 h,l;
        };
    };
} reg;



struct{
    reg a,b,c,d;
    reg sp,bp,si,di;
}registers;

u16* get_reg16(u8 r) {
    switch(r) {
        case 0b000: return &registers.a.x;
        case 0b001: return &registers.c.x;
        case 0b010: return &registers.d.x;
        case 0b011: return &registers.b.x;
        case 0b100: return &registers.sp.x;
        case 0b101: return &registers.bp.x;
        case 0b110: return &registers.si.x;
        case 0b111: return &registers.di.x;
    }
    return 0;
}

char* get_reg_str16(u8 r) {
    switch(r) {
        case 0b000: return "ax";
        case 0b001: return "cx";
        case 0b010: return "dx";
        case 0b011: return "bx";
        case 0b100: return "sp";
        case 0b101: return "bp";
        case 0b110: return "si";
        case 0b111: return "di";
    }
    return 0;
}


u8* get_reg8(u8 r) {
    switch(r) {
        case 0b000: return &registers.a.l;
        case 0b001: return &registers.c.l;
        case 0b010: return &registers.d.l;
        case 0b011: return &registers.b.l;
        case 0b100: return &registers.a.h;
        case 0b101: return &registers.c.h;
        case 0b110: return &registers.d.h;
        case 0b111: return &registers.b.h;
    }
    return 0;
}

char* get_reg_str8(u8 r) {
    switch(r) {
        case 0b000: return "al";
        case 0b001: return "cl";
        case 0b010: return "dl";
        case 0b011: return "bl";
        case 0b100: return "ah";
        case 0b101: return "ch";
        case 0b110: return "dh";
        case 0b111: return "bh";
    }
    return 0;
}

u8 get_next_byte() {
    return *(++stream);
}

u16 get_next_word() {
    u8 low = *(++stream);
    u8 high = *(++stream);
    return ((u16)high) << 8 | (u16)low;
}

u16 get_rm_displacement(u8 rm) {
    switch(rm) {
        case 0b000: return registers.b.x  + registers.si.x; 
        case 0b001: return registers.b.x  + registers.di.x; 
        case 0b010: return registers.bp.x + registers.si.x; 
        case 0b011: return registers.bp.x + registers.di.x; 
        case 0b100: return registers.si.x;                  
        case 0b101: return registers.di.x;                  
        case 0b110: return registers.bp.x;                  
        case 0b111: return registers.b.x;
    }
    __debugbreak();
    return 0;
}

char* get_rm_displacement_str(u8 rm) {
    switch(rm) {
        case 0b000: return "bx+si"; 
        case 0b001: return "bx+di"; 
        case 0b010: return "bp+si"; 
        case 0b011: return "bp+di"; 
        case 0b100: return "si";                  
        case 0b101: return "di";                  
        case 0b110: return "bp";                  
        case 0b111: return "bx";                   
    }
    __debugbreak();
    return 0;
}

#define GLUE_(a,b) a##b
#define GLUE(a,b) GLUE_(a,b)

#define side_conditional_operation(o, ltr, l, r) do { if(ltr) { *(l) GLUE(o,=) *(r); } else { *(l) GLUE(o,=) *(r); } } while(0)

void mov_regmem_tf_reg() {
    u8 byte = *stream;
    u8 d = hasbits(byte, 0b00000010);
    u8 w = hasbits(byte, 0b00000001);
    u8 byte2 = *(++stream);
    u8 mode = getbits(byte2, 0b11000000) >> 6;
    u8 reg = getbits(byte2,  0b00111000) >> 3;
    u8 rm = getbits(byte2,   0b00000111);
    if(!mode && rm == 0b110) {
        u16 addr = get_next_word();
        if(w) side_conditional_operation(, d, get_reg16(reg), (u16*)(memory + addr));
        else side_conditional_operation(, d, get_reg8(reg), memory + addr);
        #if DEBUG_PRINT
            printf("mov %s, [%d]\n", (w? get_reg_str16(reg) : get_reg_str8(reg)), addr);
        #endif
    } else switch(mode) {
        case 0b00:
        case 0b01:
        case 0b10:{ // displacement mode
            s16 disp = get_rm_displacement(rm); 
            if(mode == 0b01) disp += (s8)get_next_byte();
            else if(mode == 0b10) disp += (s16)get_next_word();
            if(w) side_conditional_operation(, d, get_reg16(reg), (u16*)(memory+disp));
            else side_conditional_operation(, d, get_reg8(reg), memory+disp);
            #if DEBUG_PRINT
                // we dont want to put more on the actual code, so we lookback here instead of saving this value 
                // before DEBUG_PRINT
                s16 actual_disp = 0;
                if(mode == 0b01) {
                    actual_disp = (s8)*stream;
                }else if(mode == 0b10){
                    u8 high = *stream;
                    u8 low = *(stream-1);
                    actual_disp = (s16)(((u16)high) << 8 | (u16)low);
                }
                if(w) {
                    if(disp < 0){
                        if(d) printf("mov %s, [%s%d]\n", get_reg_str16(reg), get_rm_displacement_str(rm), actual_disp);
                        else  printf("mov [%s%d], %s\n", get_rm_displacement_str(rm), actual_disp, get_reg_str16(reg));
                    }else if(actual_disp > 0) {
                        if(d) printf("mov %s, [%s+%d]\n", get_reg_str16(reg), get_rm_displacement_str(rm), actual_disp);
                        else  printf("mov [%s+%d], %s\n", get_rm_displacement_str(rm), actual_disp, get_reg_str16(reg));
                    }else{
                        if(d) printf("mov %s, [%s]\n", get_reg_str16(reg), get_rm_displacement_str(rm));
                        else  printf("mov [%s], %s\n", get_rm_displacement_str(rm), get_reg_str16(reg));
                    }
                } else {
                    if(actual_disp < 0){
                        if(d) printf("mov %s, [%s%d]\n", get_reg_str8(reg), get_rm_displacement_str(rm), actual_disp);
                        else  printf("mov [%s%d], %s\n", get_rm_displacement_str(rm), actual_disp, get_reg_str8(reg));
                    }else if(actual_disp > 0) {
                        if(d) printf("mov %s, [%s+%d]\n", get_reg_str8(reg), get_rm_displacement_str(rm), actual_disp);
                        else  printf("mov [%s+%d], %s\n", get_rm_displacement_str(rm), actual_disp, get_reg_str8(reg));
                    }else{
                        if(d) printf("mov %s, [%s]\n", get_reg_str8(reg), get_rm_displacement_str(rm));
                        else  printf("mov [%s], %s\n", get_rm_displacement_str(rm), get_reg_str8(reg));
                    }
                }
            #endif 
        }break;
        case 0b11:{ // register mode
            #define assign(l,r0,r1) if(w) side_conditional_operation(, d, get_reg16(l), &r0); else side_conditional_operation(, d, get_reg8(l), &r1);
            #if DEBUG_PRINT
                #define dprint(l,r0,r1)\
                    if(w) if(d) printf("mov %s, %s\n", get_reg_str16(l), r0); else printf("mov %s, %s\n", r0, get_reg_str16(l));\
                    else  if(d) printf("mov %s, %s\n", get_reg_str8(l), r1); else printf("mov %s, %s\n", r1, get_reg_str8(l));
            #else
                #define dprint(l,r0,e1)
            #endif
            switch(rm) {
                case 0b000: assign(reg, registers.a.x, registers.a.l); dprint(reg, "ax", "al"); break;
                case 0b001: assign(reg, registers.c.x, registers.c.l); dprint(reg, "cx", "cl"); break;
                case 0b010: assign(reg, registers.d.x, registers.d.l); dprint(reg, "dx", "dl"); break;
                case 0b011: assign(reg, registers.b.x, registers.b.l); dprint(reg, "bx", "bl"); break;
                case 0b100: assign(reg, registers.sp.x, registers.a.h); dprint(reg, "sp", "ah"); break;
                case 0b101: assign(reg, registers.bp.x, registers.c.h); dprint(reg, "bp", "ch"); break;
                case 0b110: assign(reg, registers.si.x, registers.d.h); dprint(reg, "si", "dh"); break;
                case 0b111: assign(reg, registers.di.x, registers.b.h); dprint(reg, "di", "bh"); break;
            }
            #undef assign
            #undef dprint
        }break;
    }
    stream++;
}


void mov_imm_to_regmem() {
    u8 byte = *stream;
    u8 w = hasbits(byte, 0b00000001);
    byte = *(++stream);
    u8 mode = getbits(byte, 0b11000000) >> 6;
    u8 rm = getbits(byte, 0b00000111);
    if(!mode && rm == 0b110) {
        u16 disp = get_next_word();
        if(w) *(u16*)(memory+disp) = get_next_word();
        else memory[disp] = get_next_byte();
    } else switch(mode) {
        case 0b00:
        case 0b01:
        case 0b10:{ // displacement mode
            s16 disp = get_rm_displacement(rm);
            if(mode == 0b01) disp += get_next_byte();
            else if(mode == 0b10) disp += get_next_word();
            if(w){
                s16 imm = get_next_word();
                *(u16*)(memory+disp) = imm;
                #if DEBUG_PRINT
                    if     (disp<0) printf("mov [%s%d], word %d\n", get_rm_displacement_str(rm), disp, imm);
                    else if(disp>0) printf("mov [%s+%d], word %d\n", get_rm_displacement_str(rm), disp, imm);
                    else            printf("mov [%s], word %d\n", get_rm_displacement_str(rm), imm);
                #endif 
            } else {
                s8 imm = get_next_byte();
                memory[disp] = imm;
                #if DEBUG_PRINT
                    if     (disp<0) printf("mov [%s%d], byte %d\n", get_rm_displacement_str(rm), disp, imm);
                    else if(disp>0) printf("mov [%s+%d], byte %d\n", get_rm_displacement_str(rm), disp, imm);
                    else            printf("mov [%s], byte %d\n", get_rm_displacement_str(rm), imm);
                #endif 
            }
        }break;
        case 0b11:{ // register mode
            __debugbreak(); //TODO
            #define assign(l,r0,r1) if(w) condassign(d, get_reg16(l), &r0); else condassign(d, get_reg8(l), &r1);
            #if DEBUG_PRINT
                #define dprint(l,r0,r1)\
                    if(w) if(d) printf("mov %s, %s\n", get_reg_str16(l), r0); else printf("mov %s, %s\n", r0, get_reg_str16(l));\
                    else  if(d) printf("mov %s, %s\n", get_reg_str8(l), r1); else printf("mov %s, %s\n", r1, get_reg_str8(l));
            #else
                #define dprint(l,r0,e1)
            #endif
            switch(rm) {
                // case 0b000: assign(reg, registers.a.x,  registers.a.l); dprint(reg, "ax", "al"); break;
                // case 0b001: assign(reg, registers.c.x,  registers.c.l); dprint(reg, "cx", "cl"); break;
                // case 0b010: assign(reg, registers.d.x,  registers.d.l); dprint(reg, "dx", "dl"); break;
                // case 0b011: assign(reg, registers.b.x,  registers.b.l); dprint(reg, "bx", "bl"); break;
                // case 0b100: assign(reg, registers.sp.x, registers.a.h); dprint(reg, "sp", "ah"); break;
                // case 0b101: assign(reg, registers.bp.x, registers.c.h); dprint(reg, "bp", "ch"); break;
                // case 0b110: assign(reg, registers.si.x, registers.d.h); dprint(reg, "si", "dh"); break;
                // case 0b111: assign(reg, registers.di.x, registers.b.h); dprint(reg, "di", "bh"); break;
            }
            #undef assign
            #undef dprint
        }break;
    }
    stream++;
}

void mov_imm_to_reg() {
    u8 byte = *stream;
    u8 w = getbits(byte, 0b00001000);
    u8 reg = getbits(byte, 0b0000111);
    if(w){
        u16 imm = get_next_word();
        *get_reg16(reg) = imm;
        #if DEBUG_PRINT
            printf("mov %s, %d\n", get_reg_str16(reg), imm);
        #endif
    }else{
        u8 imm = get_next_byte();
        *get_reg8(reg) = imm;
        #if DEBUG_PRINT
            printf("mov %s, %d\n", get_reg_str8(reg), imm);
        #endif
    }
    stream++;
}

void mov_mem_to_acc() {
    u8 byte = *stream;
    u8 w = hasbits(byte, 0b00000001);
    u16 addr;
    if(w) {
        addr = get_next_word();
        registers.a.x = *(u16*)(memory + addr);
    } else {
        addr = get_next_byte();
        registers.a.x = memory[addr];
    }
    #if DEBUG_PRINT
        printf("mov ax, [%u]\n", addr);
    #endif
    stream++;
}

void mov_acc_to_mem() {
    u8 byte = *stream;
    u8 w = hasbits(byte, 0b00000001);
    u16 addr;
    if(w) {
        addr = get_next_word();
        *(u16*)(memory + addr) = registers.a.x;
    } else {
        addr = get_next_byte();
        memory[addr] = registers.a.x;
    }
    #if DEBUG_PRINT
        printf("mov [%u], ax\n", addr);
    #endif
    stream++;
}

void mov_regmem_to_seg() {
    printf("%s\n", __func__);
}

void mov_seg_to_regmem() {
    printf("%s\n", __func__);
}

void push_inc_dec_call_jmp_regmem() {
    printf("%s\n", __func__);
}

void push_reg() {
    printf("%s\n", __func__);
}

void push_seg() {
    printf("%s\n", __func__);
}

void pop_regmem() {
    printf("%s\n", __func__);
}

void pop_reg() {
    printf("%s\n", __func__);
}

void pop_seg() {
    printf("%s\n", __func__);
}

void xchg_regmem_w_reg() {
    printf("%s\n", __func__);
}

void xchg_reg_w_acc() {
    printf("%s\n", __func__);
}

void in_fixed() {
    printf("%s\n", __func__);
}

void in_variable() {
    printf("%s\n", __func__);
}

void out_fixed() {
    printf("%s\n", __func__);
}

void out_variable() {
    printf("%s\n", __func__);
}

void out_xlat() {
    printf("%s\n", __func__);
}

void out_lea() {
    printf("%s\n", __func__);
}

void out_lds() {
    printf("%s\n", __func__);
}

void out_les() {
    printf("%s\n", __func__);
}

void out_lahf() {
    printf("%s\n", __func__);
}

void out_sahf() {
    printf("%s\n", __func__);
}

void out_pushf() {
    printf("%s\n", __func__);
}

void out_popf() {
    printf("%s\n", __func__);
}

#define condadd(ltr, left, right) do{if(ltr) {*(left)+=*(right);} else {*(right)+=*(left);}}while(0)

void add_regmem_w_reg_to_either() {
    u8 byte = *stream;
    u8 d = getbits(byte, 0b00000010);
    u8 w = getbits(byte, 0b00000001);
    byte = *(++stream);
    u8 mode = getbits(byte, 0b11000000) >> 6;
    u8 reg = getbits(byte, 0b00111000) >> 3;
    u8 rm = getbits(byte, 0b00000111);
    if(!mode && rm == 0b110) {
        u16 addr = get_next_word();
        if(w) condadd(d, get_reg16(reg), (u16*)(memory + addr));
        else condadd(d, get_reg8(reg), memory + addr);
        #if DEBUG_PRINT
            printf("add %s, [%d]\n", (w? get_reg_str16(reg) : get_reg_str8(reg)), addr);
        #endif
    } else switch(mode) {
        case 0b00:
        case 0b01:
        case 0b10:{ // displacement mode
            s16 disp = get_rm_displacement(rm); 
            if(mode == 0b01) disp += (s8)get_next_byte();
            else if(mode == 0b10) disp += (s16)get_next_word();
            if(w) side_conditional_operation(+, d, get_reg16(reg), (u16*)(memory+disp));
            else side_conditional_operation(+, d, get_reg8(reg), memory+disp);
            #if DEBUG_PRINT
                // we dont want to put more on the actual code, so we lookback here instead of saving this value 
                // before DEBUG_PRINT
                s16 actual_disp = 0;
                if     (mode == 0b01) actual_disp = (s8)*stream;
                else if(mode == 0b10) actual_disp = (s16)(((u16)*stream) << 8 | (u16)*(stream-1));
                if(disp < 0){
                    if(d) printf("add %s, [%s%d]\n", (w? get_reg_str16(reg) : get_reg_str8(reg)), get_rm_displacement_str(rm), actual_disp);
                    else  printf("add [%s%d], %s\n", get_rm_displacement_str(rm), actual_disp, (w? get_reg_str16(reg) : get_reg_str8(reg)));
                }else if(actual_disp > 0) {
                    if(d) printf("add %s, [%s+%d]\n", (w? get_reg_str16(reg) : get_reg_str8(reg)), get_rm_displacement_str(rm), actual_disp);
                    else  printf("add [%s+%d], %s\n", get_rm_displacement_str(rm), actual_disp, (w? get_reg_str16(reg) : get_reg_str8(reg)));
                }else{
                    if(d) printf("add %s, [%s]\n", (w? get_reg_str16(reg) : get_reg_str8(reg)), get_rm_displacement_str(rm));
                    else  printf("add [%s], %s\n", get_rm_displacement_str(rm), (w? get_reg_str16(reg) : get_reg_str8(reg)));
                }
            #endif 
        }break;
        case 0b11:{ // register mode
            #define add(l,r0,r1) if(w) side_conditional_operation(+, d, get_reg16(l), &r0); else side_conditional_operation(+, d, get_reg8(l), &r1);
            #if DEBUG_PRINT
                #define dprint(l,r0,r1)\
                    if(w) if(d) printf("add %s, %s\n", get_reg_str16(l), r0); else printf("add %s, %s\n", r0, get_reg_str16(l));\
                    else  if(d) printf("add %s, %s\n", get_reg_str8(l), r1); else printf("add %s, %s\n", r1, get_reg_str8(l));
            #else
                #define dprint(l,r0,e1)
            #endif
            switch(rm) {
                case 0b000: add(reg, registers.a.x,  registers.a.l); dprint(reg, "ax", "al"); break;
                case 0b001: add(reg, registers.c.x,  registers.c.l); dprint(reg, "cx", "cl"); break;
                case 0b010: add(reg, registers.d.x,  registers.d.l); dprint(reg, "dx", "dl"); break;
                case 0b011: add(reg, registers.b.x,  registers.b.l); dprint(reg, "bx", "bl"); break;
                case 0b100: add(reg, registers.sp.x, registers.a.h); dprint(reg, "sp", "ah"); break;
                case 0b101: add(reg, registers.bp.x, registers.c.h); dprint(reg, "bp", "ch"); break;
                case 0b110: add(reg, registers.si.x, registers.d.h); dprint(reg, "si", "dh"); break;
                case 0b111: add(reg, registers.di.x, registers.b.h); dprint(reg, "di", "bh"); break;
            }
            #undef add
            #undef dprint
        }break;
    }
    stream++;
}


void add_imm_to_regmem() {
    u8 byte = *stream;
    u8 s = hasbits(byte, 0b00000010);
    u8 w = hasbits(byte, 0b00000001);
    byte = *(++stream);
    u8 mode = getbits(byte, 0b11000000) >> 6;
    u8 rm = getbits(byte, 0b00000111);
    if(!mode && rm == 0b110) {
        u16 disp = get_next_word();
        if(w) *(u16*)(memory+disp) = get_next_word();
        else memory[disp] = get_next_byte();
    } else switch(mode) {
        case 0b00:
        case 0b01:
        case 0b10:{ // displacement mode
            s16 disp = get_rm_displacement(rm);
            if(mode == 0b01) disp += get_next_byte();
            else if(mode == 0b10) disp += get_next_word();
            if(w){
                s16 imm = get_next_word();
                *(u16*)(memory+disp) += imm;
                #if DEBUG_PRINT
                    s16 actual_disp = 0;
                    if     (mode == 0b01) actual_disp = (s8)*stream;
                    else if(mode == 0b10) actual_disp = (s16)(((u16)*stream) << 8 | (u16)*(stream-1));
                    if     (actual_disp<0) printf("add [%s%d], word %d\n", get_rm_displacement_str(rm), actual_disp, imm);
                    else if(actual_disp>0) printf("add [%s+%d], word %d\n", get_rm_displacement_str(rm), actual_disp, imm);
                    else            printf("add [%s], word %d\n", get_rm_displacement_str(rm), imm);
                #endif 
            } else {
                s8 imm = get_next_byte();
                memory[disp] = imm;
                #if DEBUG_PRINT
                    s16 actual_disp = 0;
                    if     (mode == 0b01) actual_disp = (s8)*stream;
                    else if(mode == 0b10) actual_disp = (s16)(((u16)*stream) << 8 | (u16)*(stream-1));
                    if     (actual_disp<0) printf("add [%s%d], byte %d\n", get_rm_displacement_str(rm), actual_disp, imm);
                    else if(actual_disp>0) printf("add [%s+%d], byte %d\n", get_rm_displacement_str(rm), actual_disp, imm);
                    else            printf("add [%s], byte %d\n", get_rm_displacement_str(rm), imm);
                #endif 
            }
        }break;
        case 0b11:{ // register mode
             #define add(l,r0,r1) if(w) *get_reg16(l) +=  side_conditional_operation(+, d, get_reg16(l), &r0); else side_conditional_operation(+, d, get_reg8(l), &r1);
            #if DEBUG_PRINT
                #define dprint(l) printf("add %s, %d\n", l, imm); 
            #else
                #define dprint(l)
            #endif
            
            if(w) { 
                s16 imm = (s? get_next_byte() : get_next_word());
                switch(rm) {
                    case 0b000: registers.a.x  = imm; dprint("ax"); break;
                    case 0b001: registers.c.x  = imm; dprint("cx"); break;
                    case 0b010: registers.d.x  = imm; dprint("dx"); break;
                    case 0b011: registers.b.x  = imm; dprint("bx"); break;
                    case 0b100: registers.sp.x = imm; dprint("sp"); break;
                    case 0b101: registers.bp.x = imm; dprint("bp"); break;
                    case 0b110: registers.si.x = imm; dprint("si"); break;
                    case 0b111: registers.di.x = imm; dprint("di"); break;
                }
            }else{
                s8 imm = get_next_byte();
                switch(rm) {
                    case 0b000: registers.a.l = imm; dprint("al"); break;
                    case 0b001: registers.c.l = imm; dprint("cl"); break;
                    case 0b010: registers.d.l = imm; dprint("dl"); break;
                    case 0b011: registers.b.l = imm; dprint("bl"); break;
                    case 0b100: registers.a.h = imm; dprint("ah"); break;
                    case 0b101: registers.c.h = imm; dprint("ch"); break;
                    case 0b110: registers.d.h = imm; dprint("dh"); break;
                    case 0b111: registers.b.h = imm; dprint("bh"); break;
                }
            }
            
            #undef add
            #undef dprint
        }break;
    }
    stream++;
}

void add_imm_to_acc() {
    printf("%s\n", __func__);
}

void adc_regmem_w_reg_to_either() {
    printf("%s\n", __func__);
}

void adc_imm_to_acc() {
    printf("%s\n", __func__);
}

void inc_regmem() {
    printf("%s\n", __func__);
}

void inc_reg() {
    printf("%s\n", __func__);
}

void inc_aaa() {
    printf("%s\n", __func__);
}

void inc_daa() {
    printf("%s\n", __func__);
}

void sub_regmem_and_reg_to_either() {
    printf("%s\n", __func__);
}

void sub_or_sbb_imm_from_regmem() {
    printf("%s\n", __func__);
}

void sub_imm_from_acc() {
    printf("%s\n", __func__);
}

void sbb_regmem_from_reg_to_either() {
    printf("%s\n", __func__);
}

void sbb_imm_from_acc() {
    printf("%s\n", __func__);
}

void dec_reg() {
    printf("%s\n", __func__);
}

void neg_mul_div_not_test() {
    printf("%s\n", __func__);
}

void cmp_regmem_and_reg() {
    printf("%s\n", __func__);
}

void cmp_imm_and_acc() {
    printf("%s\n", __func__);
}

void cmp_aas() {
    printf("%s\n", __func__);
}

void cmp_das() {
    printf("%s\n", __func__);
}

void cmp_aam() {
    printf("%s\n", __func__);
}

void cmp_aad() {
    printf("%s\n", __func__);
}

void cmp_cbw() {
    printf("%s\n", __func__);
}

void cmp_cwd() {
    printf("%s\n", __func__);
}

void not() {
    printf("%s\n", __func__);
}

void shl_sal_shr_sar_rol_ror_rcl_rcr() {
    printf("%s\n", __func__);
}

void and_regmem_with_reg_to_either() {
    printf("%s\n", __func__);
}

void and_imm_to_acc() {
    printf("%s\n", __func__);
}

void test_regmem_and_reg() {
    printf("%s\n", __func__);
}

void test_imm_and_acc() {
    printf("%s\n", __func__);
}

void xor_regmem_and_reg_to_either() {
    printf("%s\n", __func__);
}

void xor_imm_to_regmem_or_acc() {
    printf("%s\n", __func__);
}

void str_rep() {
    printf("%s\n", __func__);
}

void str_movs() {
    printf("%s\n", __func__);
}

void str_cmps() {
    printf("%s\n", __func__);
}

void str_scas() {
    printf("%s\n", __func__);
}

void str_lods() {
    printf("%s\n", __func__);
}

void str_stds() {
    printf("%s\n", __func__);
}

void call_direct_within_segment() {
    printf("%s\n", __func__);
}

void call_indirect_within_segment() {
    printf("%s\n", __func__);
}

void call_direct_intersegment() {
    printf("%s\n", __func__);
}

void call_indirect_intersegment() {
    printf("%s\n", __func__);
}

void jmp_direct_within_segment() {
    printf("%s\n", __func__);
}

void jmp_direct_within_segment_short() {
    printf("%s\n", __func__);
}

void jmp_direct_intersegment() {
    printf("%s\n", __func__);
}

void ret_within_segment() {
    printf("%s\n", __func__);
}

void ret_within_segment_add_imm_to_sp() {
    printf("%s\n", __func__);
}

void ret_intersegment() {
    printf("%s\n", __func__);
}

void ret_intersegment_add_imm_to_sp() {
    printf("%s\n", __func__);
}

// NOTE(sushi) the jump opcodes expect the  to have been incremented already
void je_jz() {
    printf("%s\n", __func__);
}

void jl_jnge() {
    printf("%s\n", __func__);
}

void jle_jng() {
    printf("%s\n", __func__);
}

void jb_jnae() {
    printf("%s\n", __func__);
}

void jbe_jna() {
    printf("%s\n", __func__);
}

void jp_jpe() {
    printf("%s\n", __func__);
}

void jo() {
    printf("%s\n", __func__);
}

void js() {
    printf("%s\n", __func__);
}

void jne_jnz() {
    printf("%s\n", __func__);
}

void jnl_jge() {
    printf("%s\n", __func__);
}

void jnle_jg() {
    printf("%s\n", __func__);
}

void jnb_jae() {
    printf("%s\n", __func__);
}

void jnbe_ja() {
    printf("%s\n", __func__);
}

void jnp_jpo() {
    printf("%s\n", __func__);
}

void jno() {
    printf("%s\n", __func__);
}

void jns() {
    printf("%s\n", __func__);
}

void loop_() {
    printf("%s\n", __func__);
}

void loopz_loope() {
    printf("%s\n", __func__);
}

void loopnz_loopne() {
    printf("%s\n", __func__);
}

void jcxz() {
    printf("%s\n", __func__);
}

void interrupt_typed() {
    printf("%s\n", __func__);
}

void interrupt_type_3() {
    printf("%s\n", __func__);
}

void interrupt_on_overflow() {
    printf("%s\n", __func__);
}

void interrupt_return() {
    printf("%s\n", __func__);
}

void clear_carry() {
    printf("%s\n", __func__);
}

void complement_carry() {
    printf("%s\n", __func__);
}

void set_carry() {
    printf("%s\n", __func__);
}

void clear_direction() {
    printf("%s\n", __func__);
}

void set_direction() {
    printf("%s\n", __func__);
}

void clear_interrupt() {
    printf("%s\n", __func__);
}

void set_interrupt() {
    printf("%s\n", __func__);
}

void halt() {
    printf("%s\n", __func__);
}

void wait() {
    printf("%s\n", __func__);
}

void esc() {
    printf("%s\n", __func__);
} 

void lock() {
    printf("%s\n", __func__);
}

void segment() {
    printf("%s\n", __func__);
}

void add_adc_sub_sbb_cmp_and_or_imm_to_regmem() {
    u8 type = getbits(*(stream + 1), 0b00111000);
    switch(type) {
        case 0b000: add_imm_to_regmem(); break;
        default: __debugbreak();
    }
}

void decode() {
    u8 byte = *stream;
    u8 
    b0 = (byte >> 7) & 1,
    b1 = (byte >> 6) & 1,
    b2 = (byte >> 5) & 1,
    b3 = (byte >> 4) & 1,
    b4 = (byte >> 3) & 1,
    b5 = (byte >> 2) & 1,
    b6 = (byte >> 1) & 1,
    b7 = (byte >> 0) & 1;

    if(b0) { // ------------------------------------------------------------- 1xxxxxxx
        if(b1) { // --------------------------------------------------------- 11xxxxxx
            if(b2) { // ----------------------------------------------------- 111xxxxx
                if(b3) { // ------------------------------------------------- 1111xxxx
                    if(b4) { // --------------------------------------------- 11111xxx
                        if(b5) { // ----------------------------------------- 111111xx
                            if(b6) { // ------------------------------------- 1111111x
                                push_inc_dec_call_jmp_regmem();
                            } else { // ------------------------------------ 1111110x
                                if(b7) { // --------------------------------- 11111101
                                    set_direction();
                                } else { // -------------------------------- 11111100
                                    clear_direction();
                                }
                            }
                        } else { // ---------------------------------------- 111110xx
                            if(b6) { // ------------------------------------- 1111101x
                                if(b7) { // --------------------------------- 11111011
                                    set_interrupt();
                                } else { // -------------------------------- 11111010
                                    clear_interrupt();
                                }
                            } else { // ------------------------------------ 1111100x
                                if(b7) { // --------------------------------- 11111001
                                    set_carry();
                                } else { // -------------------------------- 11111000
                                    clear_carry();
                                }
                            }
                        }
                    } else { // -------------------------------------------- 11110xxx
                        if(b5) { // ----------------------------------------- 111101xx
                            if(b6) { // ------------------------------------- 1111011x
                                neg_mul_div_not_test();
                            } else { // ------------------------------------ 1111010x
                                if(b7) { // --------------------------------- 11110101
                                    complement_carry();
                                } else { // -------------------------------- 11110100
                                    halt();
                                }
                            }
                        } else { // ---------------------------------------- 111100xx
                            if(b6) { // ------------------------------------- 1111001x
                                if(b7) { // --------------------------------- 11110011
                                    str_rep();
                                } else { // -------------------------------- 11110010
                                    __debugbreak();
                                }
                            } else { // ------------------------------------ 1111000x
                                if(b7) { // --------------------------------- 11110001
                                    __debugbreak();
                                } else { // -------------------------------- 11110000
                                    lock();
                                }
                            }
                        }
                    }
                } else { // ------------------------------------------------ 1110xxxx
                    if(b4) { // --------------------------------------------- 11101xxx
                        if(b5) { // ----------------------------------------- 111011xx
                            if(b6) { // ------------------------------------- 1110111x
                                out_variable();
                            } else { // ------------------------------------ 1110110x
                                in_variable();
                            }
                        } else { // ---------------------------------------- 111010xx
                            if(b6) { // ------------------------------------- 1110101x
                                if(b7) { // --------------------------------- 11101011
                                    jmp_direct_within_segment_short();
                                } else { // -------------------------------- 11101010
                                    jmp_direct_intersegment();
                                }
                            } else { // ------------------------------------ 1110100x
                                if(b7) { // --------------------------------- 11101001
                                    jmp_direct_within_segment();
                                } else { // -------------------------------- 11101000
                                    call_direct_within_segment();
                                }
                            }
                        }
                    } else { // -------------------------------------------- 11100xxx
                        if(b5) { // ----------------------------------------- 111001xx
                            if(b6) { // ------------------------------------- 1110011x
                                out_fixed();
                            } else { // ------------------------------------ 1110010x
                                in_fixed();
                            }
                        } else { // ---------------------------------------- 111000xx
                            if(b6) { // ------------------------------------- 1110001x
                                if(b7) { // --------------------------------- 11100011
                                    jcxz();
                                } else { // -------------------------------- 11100010
                                    loop_();
                                }
                            } else { // ------------------------------------ 1110000x
                                if(b7) { // --------------------------------- 11100001
                                    loopz_loope();
                                } else { // -------------------------------- 11100000
                                    loopnz_loopne();
                                }
                            }
                        }
                    }
                }
            } else { // ---------------------------------------------------- 110xxxxx
                if(b3) { // ------------------------------------------------- 1101xxxx
                    if(b4) { // --------------------------------------------- 11011xxx
                        esc();
                    } else { // -------------------------------------------- 11010xxx
                        if(b5) { // ----------------------------------------- 110101xx
                            if(b6) { // ------------------------------------- 1101011x
                                if(b7) { // --------------------------------- 11010111
                                    out_xlat();
                                } else { // -------------------------------- 11010110
                                    __debugbreak();
                                }
                            } else { // ------------------------------------ 1101010x
                                if(b7) { // --------------------------------- 11010101
                                    cmp_aad();
                                } else { // -------------------------------- 11010100
                                    cmp_aam();
                                }
                            }
                        } else { // ---------------------------------------- 110100xx
                            shl_sal_shr_sar_rol_ror_rcl_rcr();
                        }
                    }
                } else { // ------------------------------------------------ 1100xxxx
                    if(b4) { // --------------------------------------------- 11001xxx
                        if(b5) { // ----------------------------------------- 110011xx
                            if(b6) { // ------------------------------------- 1100111x
                                if(b7) { // --------------------------------- 11001111
                                    interrupt_return();
                                } else { // -------------------------------- 11001110
                                    interrupt_on_overflow();
                                }
                            } else { // ------------------------------------ 1100110x
                                if(b7) { // --------------------------------- 11001101
                                    interrupt_typed();
                                } else { // -------------------------------- 11001100
                                    interrupt_type_3();
                                }
                            }
                        } else { // ---------------------------------------- 110010xx
                            if(b6) { // ------------------------------------- 1100101x
                                if(b7) { // --------------------------------- 11001011
                                    ret_intersegment();
                                } else { // -------------------------------- 11001010
                                    ret_intersegment_add_imm_to_sp();
                                }
                            } else { // ------------------------------------ 1100100x
                                if(b7) { // --------------------------------- 11001001
                                    __debugbreak();
                                } else { // -------------------------------- 11001000
                                    __debugbreak();
                                }
                            }
                        }
                    } else { // -------------------------------------------- 11000xxx
                        if(b5) { // ----------------------------------------- 110001xx
                            if(b6) { // ------------------------------------- 1100011x --- mov: imm->reg/mem
                                mov_imm_to_regmem();
                            } else { // ------------------------------------ 1100010x
                                if(b7) { // --------------------------------- 11000101
                                    out_lds();
                                } else { // -------------------------------- 11000100
                                    out_les();
                                }
                            }
                        } else { // ---------------------------------------- 110000xx
                            if(b6) { // ------------------------------------- 1100001x
                                if(b7) { // --------------------------------- 11000011
                                    ret_within_segment();
                                } else { // -------------------------------- 11000010
                                    ret_within_segment_add_imm_to_sp();
                                }
                            } else { // ------------------------------------ 1100000x
                                if(b7) { // --------------------------------- 11000001
                                    __debugbreak();
                                } else { // -------------------------------- 11000000
                                    __debugbreak();
                                }
                            }
                        }
                    }
                }
            }
        } else { // -------------------------------------------------------- 10xxxxxx
            if(b2) { // ----------------------------------------------------- 101xxxxx
                if(b3) { // ------------------------------------------------- 1011xxxx --- mov: imm->reg
                    mov_imm_to_reg();
                } else { // ------------------------------------------------ 1010xxxx
                    if(b4) { // --------------------------------------------- 10101xxx
                        if(b5) { // ----------------------------------------- 101011xx
                            if(b6) { // ------------------------------------- 1010111x
                                str_scas();
                            } else { // ------------------------------------ 1010110x
                                str_lods();
                            }
                        } else { // ---------------------------------------- 101010xx
                            if(b6) { // ------------------------------------- 1010101x
                                str_stds();
                            } else { // ------------------------------------ 1010100x
                                test_imm_and_acc();
                            }
                        }
                    } else { // -------------------------------------------- 10100xxx
                        if(b5) { // ----------------------------------------- 101001xx
                            if(b6) { // ------------------------------------- 1010011x
                                str_cmps();
                            } else { // ------------------------------------ 1010010x
                                str_movs();
                            }
                        } else { // ---------------------------------------- 101000xx
                            if(b6) { // ------------------------------------- 1010001x --- mov: acc->mem
                                mov_acc_to_mem();
                            } else { // ------------------------------------ 1010000x --- mov: mem->acc
                                mov_mem_to_acc();
                            }
                        }
                    }
                }
            } else { // ---------------------------------------------------- 100xxxxx
                if(b3) { // ------------------------------------------------- 1001xxxx
                    if(b4) { // --------------------------------------------- 10011xxx
                        if(b5) { // ----------------------------------------- 100111xx
                            if(b6) { // ------------------------------------- 1001111x
                                if(b7) { // --------------------------------- 10011111
                                    out_lahf();
                                } else { // -------------------------------- 10011110
                                    out_sahf();
                                }
                            } else { // ------------------------------------ 1001110x
                                if(b7) { // --------------------------------- 10011101
                                    out_popf();
                                } else { // -------------------------------- 10011100
                                    out_pushf();
                                }
                            }
                        } else { // ---------------------------------------- 100110xx
                            if(b6) { // ------------------------------------- 1001101x
                                if(b7) { // --------------------------------- 10011011
                                    wait();
                                } else { // -------------------------------- 10011010
                                    call_direct_intersegment();
                                }
                            } else { // ------------------------------------ 1001100x
                                if(b7) { // --------------------------------- 10011001
                                    cmp_cwd();
                                } else { // -------------------------------- 10011000
                                    cmp_cbw();
                                }
                            }
                        }
                    } else { // -------------------------------------------- 10010xxx
                        xchg_reg_w_acc();
                    }
                } else {  // ------------------------------------------------ 1000xxxx
                    if(b4) { // --------------------------------------------- 10001xxx
                        if(b5) { // ----------------------------------------- 100011xx
                            if(b6) { // ------------------------------------- 1000111x
                                if(b7) { // --------------------------------- 10001111
                                    pop_regmem();
                                } else { // -------------------------------- 10001110 --- mov: reg/mem->seg
                                    mov_regmem_to_seg();
                                }   
                            } else {  // ------------------------------------ 1000110x
                                if(b7) { // --------------------------------- 10001101
                                    out_lea();
                                } else { // -------------------------------- 10001100 --- mov: seg->reg/mem
                                    mov_seg_to_regmem();
                                }
                            }
                        } else { // ---------------------------------------- 100010xx --- mov: reg/mem<->reg
                            mov_regmem_tf_reg();
                        }
                    } else {  // -------------------------------------------- 10000xxx
                        if(b5) { // ----------------------------------------- 100001xx
                            if(b6) { // ------------------------------------- 1000011x
                                xchg_regmem_w_reg();
                            } else {  // ------------------------------------ 1000010x
                                if(b7) { // --------------------------------- 10000101
                                    __debugbreak();
                                } else {  // -------------------------------- 10000100
                                    __debugbreak();
                                }
                            }
                        } else { // ---------------------------------------- 100000xx 
                            switch(getbits(*(stream+1), 0b00111000)) {
                                case 0b000: add_imm_to_regmem(); break;
                                default: __debugbreak();
                            }
                            
                        }
                    }
                }
            }
        }
    } else {  // ------------------------------------------------------------ 0xxxxxxx
        if(b1) { // --------------------------------------------------------- 01xxxxxx
            if(b2) { // ----------------------------------------------------- 011xxxxx
                if(b3) { // ------------------------------------------------- 0111xxxx
                    if(b4) { // --------------------------------------------- 01111xxx
                        if(b5) { // ----------------------------------------- 011111xx
                            if(b6) { // ------------------------------------- 0111111x
                                if(b7) { // --------------------------------- 01111111
                                    jnle_jg();
                                } else { // -------------------------------- 01111110
                                    jle_jng();
                                }
                            } else { // ------------------------------------ 0111110x
                                if(b7) { // --------------------------------- 01111101
                                    __debugbreak();
                                } else { // -------------------------------- 01111100
                                    jl_jnge();
                                }
                            }
                        } else { // ---------------------------------------- 011110xx
                            if(b6) { // ------------------------------------- 0111101x
                                if(b7) { // --------------------------------- 01111011
                                    jnp_jpo();
                                } else { // -------------------------------- 01111010
                                    jp_jpe();
                                }
                            } else { // ------------------------------------ 0111100x
                                if(b7) { // --------------------------------- 01111001
                                    jns();
                                } else { // -------------------------------- 01111000
                                    js();
                                }
                            }
                        }
                    } else { // -------------------------------------------- 01110xxx
                        if(b5) { // ----------------------------------------- 011101xx
                            if(b6) { // ------------------------------------- 0111011x
                                if(b7) { // --------------------------------- 01110111
                                    jnbe_ja();
                                } else { // -------------------------------- 01110110
                                    jbe_jna();
                                }
                            } else { // ------------------------------------ 0111010x
                                if(b7) { // --------------------------------- 01110101
                                    jne_jnz();
                                } else { // -------------------------------- 01110100
                                    je_jz();
                                }
                            }
                        } else { // ---------------------------------------- 011100xx
                            if(b6) { // ------------------------------------- 0111001x
                                if(b7) { // --------------------------------- 01110011
                                    jnb_jae();
                                } else { // -------------------------------- 01110010
                                    jb_jnae();
                                }
                            } else { // ------------------------------------ 0111000x
                                if(b7) { // --------------------------------- 01110001
                                    jno();
                                } else { // -------------------------------- 01110000
                                    jo();
                                }
                            }
                        }
                    }
                } else { // ------------------------------------------------ 0110xxxx
                    if(b4) { // --------------------------------------------- 01101xxx
                        if(b5) { // ----------------------------------------- 011011xx
                            if(b6) { // ------------------------------------- 0110111x
                                if(b7) { // --------------------------------- 01101111
                                    __debugbreak();
                                } else { // -------------------------------- 01101110
                                    __debugbreak();
                                }
                            } else { // ------------------------------------ 0110110x
                                if(b7) { // --------------------------------- 01101101
                                    __debugbreak();
                                } else { // -------------------------------- 01101100
                                    __debugbreak();
                                }
                            }
                        } else { // ---------------------------------------- 011010xx
                            if(b6) { // ------------------------------------- 0110101x
                                if(b7) { // --------------------------------- 01101011
                                    __debugbreak();
                                } else { // -------------------------------- 01101010
                                    __debugbreak();
                                }
                            } else { // ------------------------------------ 0110100x
                                if(b7) { // --------------------------------- 01101001
                                    __debugbreak();
                                } else { // -------------------------------- 01101000
                                    __debugbreak();
                                }
                            }
                        }
                    } else { // -------------------------------------------- 01100xxx
                        if(b5) { // ----------------------------------------- 011001xx
                            if(b6) { // ------------------------------------- 0110011x
                                if(b7) { // --------------------------------- 01100111
                                    __debugbreak();
                                } else { // -------------------------------- 01100110
                                    __debugbreak();
                                }
                            } else { // ------------------------------------ 0110010x
                                if(b7) { // --------------------------------- 01100101
                                    __debugbreak();
                                } else { // -------------------------------- 01100100
                                    __debugbreak();
                                }
                            }
                        } else { // ---------------------------------------- 011000xx
                            if(b6) { // ------------------------------------- 0110001x
                                if(b7) { // --------------------------------- 01100011
                                    __debugbreak();
                                } else { // -------------------------------- 01100010
                                    __debugbreak();
                                }
                            } else { // ------------------------------------ 0110000x
                                if(b7) { // --------------------------------- 01100001
                                    __debugbreak();
                                } else { // -------------------------------- 01100000
                                    __debugbreak();
                                }
                            }
                        }
                    }
                }
            } else { // ---------------------------------------------------- 010xxxxx
                if(b3) { // ------------------------------------------------- 0101xxxx
                    if(b4) { // --------------------------------------------- 01011xxx
                        pop_reg();
                    } else { // -------------------------------------------- 01010xxx
                        push_reg();
                    }
                } else { // ------------------------------------------------ 0100xxxx
                    if(b4) { // --------------------------------------------- 01001xxx
                        dec_reg();
                    } else { // -------------------------------------------- 01000xxx
                        inc_reg();
                    }
                }
            }
        } else { // -------------------------------------------------------- 00xxxxxx
            if(b2) { // ----------------------------------------------------- 001xxxxx
                if(b3) { // ------------------------------------------------- 0011xxxx
                    if(b4) { // --------------------------------------------- 00111xxx
                        if(b5) { // ----------------------------------------- 001111xx
                            if(b6) { // ------------------------------------- 0011111x
                                if(b7) { // --------------------------------- 00111111
                                    cmp_aas();
                                } else { // -------------------------------- 00111110
                                    __debugbreak();
                                }
                            } else { // ------------------------------------ 0011110x
                                cmp_imm_and_acc();
                            }
                        } else { // ---------------------------------------- 001110xx
                            cmp_regmem_and_reg();
                        }
                    } else { // -------------------------------------------- 00110xxx
                        if(b5) { // ----------------------------------------- 001101xx
                            if(b6) { // ------------------------------------- 0011011x
                                if(b7) { // --------------------------------- 00110111
                                    inc_aaa();
                                } else { // -------------------------------- 00110110
                                    __debugbreak();
                                }
                            } else { // ------------------------------------ 0011010x
                                xor_imm_to_regmem_or_acc();
                            }
                        } else { // ---------------------------------------- 001100xx
                            xor_regmem_and_reg_to_either();
                        }
                    }
                } else { // ------------------------------------------------ 0010xxxx
                    if(b4) { // --------------------------------------------- 00101xxx
                        if(b5) { // ----------------------------------------- 001011xx
                            if(b6) { // ------------------------------------- 0010111x
                                if(b7) { // --------------------------------- 00101111
                                    cmp_das();
                                } else { // -------------------------------- 00101110
                                    __debugbreak();
                                }
                            } else { // ------------------------------------ 0010110x
                                sub_imm_from_acc();
                            }
                        } else { // ---------------------------------------- 001010xx
                            sub_regmem_and_reg_to_either();
                        }
                    } else { // -------------------------------------------- 00100xxx
                        if(b5) { // ----------------------------------------- 001001xx
                            if(b6) { // ------------------------------------- 0010011x
                                if(b7) { // --------------------------------- 00100111
                                    inc_daa();
                                } else { // -------------------------------- 00100110
                                    __debugbreak();
                                }
                            } else { // ------------------------------------ 0010010x
                                and_imm_to_acc();
                            }
                        } else { // ---------------------------------------- 001000xx
                            and_regmem_with_reg_to_either();
                        }
                    }
                }
            } else { // ---------------------------------------------------- 000xxxxx
                // special case where we must check the last 3 bits for a couple patterns
                switch(byte & 0b00000111) {
                    case 0b110: {
                        push_seg();

                    }break;
                    case 0b111:{
                        pop_seg();
                    }break;
                    default:{
                        if(b3) { // ------------------------------------------------- 0001xxxx
                            if(b4) { // --------------------------------------------- 00011xxx
                                if(b5) { // ----------------------------------------- 000111xx
                                    if(b6) { // ------------------------------------- 0001111x
                                        if(b7) { // --------------------------------- 00011111
                                            __debugbreak();
                                        } else { // -------------------------------- 00011110
                                            __debugbreak();
                                        }
                                    } else { // ------------------------------------ 0001110x
                                        sbb_imm_from_acc();
                                    }
                                } else { // ---------------------------------------- 000110xx
                                    sbb_regmem_from_reg_to_either();
                                }
                            } else { // -------------------------------------------- 00010xxx
                                if(b5) { // ----------------------------------------- 000101xx
                                    if(b6) { // ------------------------------------- 0001011x
                                        if(b7) { // --------------------------------- 00010111
                                            __debugbreak();
                                        } else { // -------------------------------- 00010110
                                            __debugbreak();
                                        }
                                    } else { // ------------------------------------ 0001010x
                                        adc_imm_to_acc();
                                    }
                                } else { // ---------------------------------------- 000100xx
                                    adc_regmem_w_reg_to_either();
                                }
                            }
                        } else { // ------------------------------------------------ 0000xxxx
                            if(b4) { // --------------------------------------------- 00001xxx
                                if(b5) { // ----------------------------------------- 000011xx
                                    if(b6) { // ------------------------------------- 0000111x
                                        if(b7) { // --------------------------------- 00001111
                                            __debugbreak();
                                        } else { // -------------------------------- 00001110
                                            __debugbreak();
                                        }
                                    } else { // ------------------------------------ 0000110x
                                        if(b7) { // --------------------------------- 00001101
                                            __debugbreak();
                                        } else { // -------------------------------- 00001100
                                            __debugbreak();
                                        }
                                    }
                                } else { // ---------------------------------------- 000010xx
                                    if(b6) { // ------------------------------------- 0000101x
                                        if(b7) { // --------------------------------- 00001011
                                            __debugbreak();
                                        } else { // -------------------------------- 00001010
                                            __debugbreak();
                                        }
                                    } else { // ------------------------------------ 0000100x
                                        if(b7) { // --------------------------------- 00001001
                                            __debugbreak();
                                        } else { // -------------------------------- 00001000
                                            __debugbreak();
                                        }
                                    }
                                }
                            } else { // -------------------------------------------- 00000xxx
                                if(b5) { // ----------------------------------------- 000001xx
                                    if(b6) { // ------------------------------------- 0000011x
                                        if(b7) { // --------------------------------- 00000111
                                            __debugbreak();
                                        } else { // -------------------------------- 00000110
                                            __debugbreak();
                                        }
                                    } else { // ------------------------------------ 0000010x
                                        add_imm_to_acc();
                                    }
                                } else { // ---------------------------------------- 000000xx
                                    add_regmem_w_reg_to_either();
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

int main() {
    FILE* file;
    fopen_s(&file, "computer_enhance/perfaware/part1/listing_0041_add_sub_cmp_jnz", "rb");
    if(!file) {
        printf("couldn't open file.");
    }

    // Determine the file size
    fseek(file, 0, SEEK_END);
    u64 file_size = ftell(file);
    rewind(file);

    stream = (u8*)malloc(file_size);
    if (!stream) {
        printf("Failed to allocate memory for the buffer.\n");
        fclose(file);
        return 1;
    }

    size_t result = fread(stream, 1, file_size, file);
    if (result != file_size) {
        printf("Failed to read the file.\n");
        fclose(file);
        free(stream);
        return 1;
    }

    u8* stream_start = stream;
    while(stream - stream_start < file_size){
        decode();
    }
}