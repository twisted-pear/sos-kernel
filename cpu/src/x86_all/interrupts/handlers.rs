//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015-2016 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
use context::InterruptFrame;
use super::pics::end_pic_interrupt;

use vga::{CONSOLE, Color};

use core::fmt;
use core::fmt::Write;

pub type InterruptHandler = extern "C" fn (*const InterruptFrame);
pub type ErrorCodeHandler = extern "C" fn (*const InterruptFrame, usize);

#[derive(Debug)]
pub struct ExceptionInfo { pub name: &'static str
                         , pub mnemonic: &'static str
                         , pub irq_type: &'static str
                         , pub source: &'static str
                         }

/// x86 exceptions.
///
/// Taken from the list at
/// [http://wiki.osdev.org/Exceptions](http://wiki.osdev.org/Exceptions)
pub static EXCEPTIONS: [ExceptionInfo; 20]
    = [ ExceptionInfo { name: "Divide-By-Zero Error"
                      , mnemonic: "#DE", irq_type: "Fault"
                      , source: "DIV or IDIV instruction" }
      , ExceptionInfo { name: "RESERVED"
                      , mnemonic: "#DB", irq_type: "Fault/trap"
                      , source: "Reserved for Intel use" }
      , ExceptionInfo { name: "Non-Maskable Interrupt"
                      , mnemonic: "NMI", irq_type: "Interrupt"
                      , source: "Non-maskable external interrupt" }
      , ExceptionInfo { name: "Breakpoint"
                      , mnemonic: "#BP", irq_type: "Trap"
                      , source: "INT 3 instruction" }
      , ExceptionInfo { name: "Overflow"
                      , mnemonic: "#OF", irq_type: "Trap"
                      , source: "INTO instruction" }
      , ExceptionInfo { name: "BOUND Range Exceeded"
                      , mnemonic: "#BR", irq_type: "Fault"
                      , source: "BOUND instruction" }
      , ExceptionInfo { name: "Undefined Opcode"
                     , mnemonic: "#UD", irq_type: "Fault"
                     , source: "UD2 instruction or reserved opcode" }
      , ExceptionInfo { name: "Device Not Available"
                      , mnemonic: "#NM", irq_type: "Fault"
                      , source: "Floating-point or WAIT/FWAIT instruction\
                                 (no math coprocessor)" }
      , ExceptionInfo { name: "Double Fault"
                      , mnemonic: "#DF", irq_type: "Abort"
                      , source: "Any instruction that can generate an\
                                 exception, a NMI, or an INTR" }
      , ExceptionInfo { name: "Coprocessor Segment Overrun"
                      , mnemonic: "", irq_type: "Fault"
                      , source: "Any floating-point instruction" }
      , ExceptionInfo { name: "Invalid TSS"
                      , mnemonic: "#TS", irq_type: "Fault"
                      , source: "Task switch or TSS access" }
      , ExceptionInfo { name: "Segment Not Present"
                      , mnemonic: "#NP", irq_type: "Fault"
                      , source: "Loading segment registers or accessing\
                                 system segments" }
      , ExceptionInfo { name: "Stack-Segment Fault"
                      , mnemonic: "#SS", irq_type: "Fault"
                      , source: "Stack operations and SS register loads" }
      , ExceptionInfo { name: "General Protection"
                      , mnemonic: "#GP", irq_type: "Fault"
                      , source: "Any memory reference or other protection\
                                 checks" }
      , ExceptionInfo { name: "Page Fault"
                      , mnemonic: "#PF", irq_type: "Fault"
                      , source: "Any memory reference" }
      , ExceptionInfo { name: "RESERVED"
                      , mnemonic: "", irq_type: ""
                      , source: "RESERVED FOR INTEL USE \n This should never \
                                 happen. Something is very wrong." }
      , ExceptionInfo { name: "x87 FPU Floating-Point Error (Math Fault)"
                      , mnemonic: "#MF", irq_type: "Fault"
                      , source: "x87 FPU floating-point or WAIT/FWAIT\
                                 instruction" }
      , ExceptionInfo { name: "Alignment Check"
                      , mnemonic: "#AC", irq_type: "Fault"
                      , source: "Any data reference in memory" }
      , ExceptionInfo { name: "Machine Check"
                      , mnemonic: "#MC", irq_type: "Abort"
                      , source: "Model-dependent" }
      , ExceptionInfo { name: "SIMD Floating-Point Exception"
                      , mnemonic: "#XM", irq_type: "Fault"
                      , source: "SSE/SSE2/SSE3 floating-point instructions" }
       ];


bitflags! {
   flags PageFaultErrorCode: u32 {
       /// If 1, the error was caused by a page that was present.
       /// Otherwise, the page was non-present.
       const PRESENT = 1 << 0
     , /// If 1, the error was caused by a read. If 0, the cause was a write.
       const READ_WRITE = 1 << 1
     , /// If 1, the error was caused during user-mode execution.
       /// If 0, the processor was in kernel mode.
       const USER_MODE = 1 << 2
     , /// If 1, the fault was caused by reserved bits set to 1 during a fetch.
       const RESERVED = 1 << 3
     , /// If 1, the fault was caused during an instruction fetch.
       const INST_FETCH = 1 << 4
     , /// If 1, there was a protection key violation.
       const PROTECTION = 1 << 5
   }
}

impl fmt::Display for PageFaultErrorCode {
   fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       write!( f, "Caused by {}{}{} during a {}{} executing in {} mode."
             , if self.contains(PRESENT) { "a present page" }
               else { "a non-present page" }
             , if self.contains(PROTECTION) { " protection-key violation" }
               else { "" }
             , if self.contains(RESERVED) { " reserved bits set to one "}
               else { "" }
             , if self.contains(READ_WRITE) { "read" } else { "write" }
             , if self.contains(INST_FETCH) { " in an instruction fetch"}
               else { "" }
             , if self.contains(USER_MODE) { "user" } else { "kernel" }            )
   }
}

pub extern "C" fn timer(_frame: *const InterruptFrame) {
    // do nothing, just signal the pics to end the IRQ
    // println!("timer!");
    unsafe { end_pic_interrupt(0x21); }
}



/// Handles page fault exceptions
#[no_mangle] #[inline(never)]
pub extern "C" fn page_fault( frame: *const InterruptFrame, error_code: usize) {
   unsafe {
       let _ = write!( CONSOLE.lock()
                          .set_colors(Color::White, Color::Blue)
                       //   .clear()
                 , "IT'S NOT MY FAULT: Page Fault at {:p} \
                    \nError code: {:#x}\n\n{}\n{:?}"
                 , (*frame).rip
                 , error_code
                 , PageFaultErrorCode::from_bits_truncate(error_code as u32)
                 , *frame
                 );
    }
   // TODO: stack dumps please

   loop { }
}

#[no_mangle] #[inline(never)]
pub extern "C" fn test(_frame: *const InterruptFrame) {
   // assert_eq!(state.int_id, 0x80);
   kinfoln!(dots: " . . ", target: "Testing interrupt handling:", "[ OKAY ]");
   // send the PICs the end interrupt signal
   unsafe {
       end_pic_interrupt(0xff);
   }
}


#[no_mangle] #[inline(never)]
pub extern "C" fn empty_handler(_frame: *const InterruptFrame) {
   // assert_eq!(state.int_id, 0x80);
   println!("interrupt");
   // send the PICs the end interrupt signal
   // unsafe {
   //     end_pic_interrupt(0xff);
   // }
}

macro_rules! make_handlers {
    ( $(ex $ex_num:expr, $name:ident),+ ) => {
        $(
            #[no_mangle]
            pub extern "C" fn $name(frame: *const InterruptFrame) {
                unsafe {
                    let ex_info = &EXCEPTIONS[$ex_num];
                    // let cr_state = control_regs::dump();
                    let _ = write!( CONSOLE.lock()
                                           .set_colors(Color::White, Color::Blue)
                                  , "EVERYTHING IS FINE: {}{} at {:p}\n\
                                     Exception on vector {}.\n\
                                     Source: {}.\nThis is fine.\n\n\
                                     {:?}"
                                     , ex_info.name, ex_info.irq_type
                                     , (*frame).rip
                                     , $ex_num
                                     , ex_info.source
                                     , *frame);
                    loop { }
                }
            }
        )+
    };
    ( $(err $ex_num:expr, $name:ident) ,+ ) => {
        $(
            #[no_mangle]
            pub extern "C" fn $name( frame: *const InterruptFrame
                                   , err_code: usize) {
                unsafe {
                    let ex_info = &EXCEPTIONS[$ex_num];
                    // let cr_state = control_regs::dump();
                    let _ = write!( CONSOLE.lock()
                                           .set_colors(Color::White, Color::Blue)
                                  , "EVERYTHING IS FINE: {}{} at {:p}\n\
                                     Exception on vector {} with error code {:#x}.\n\
                                     Source: {}.\nThis is fine.\n\n\
                                     {:?}"
                                  , ex_info.name, ex_info.irq_type
                                  , (*frame).rip
                                  , $ex_num, err_code
                                  , ex_info.source
                                  , *frame);
                    loop { }
                }
            }
        )+
    };
}
make_handlers! { ex 0, ex0
               , ex 1, ex1
               , ex 2, ex2
                // ex 3 is breakpoint
               , ex 4, ex4
               , ex 5, ex5
               , ex 6, ex6
               , ex 7, ex7
               , ex 16, ex16
               , ex 18, ex18
               , ex 19, ex19
               }
make_handlers! { err 8, ex8
               , err 10, ex10
               , err 11, ex11
               , err 12, ex12
               , err 13, ex13
               , err 17, ex17 }
