use crate::gdt;
use crate::println;
use lazy_static::lazy_static;
use x86_64::structures::idt::{
    InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode,
};

// The IDT must remain in memory for the entire life of the OS; CPU depends on
// this memory being initialized to know the locations of handler functions for
// various interrupts
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt
    };
}

// Intialize the Interrupt Descriptor Table with the addresses of our interrupt
// handler functions using the lidt instruction.
pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn divide_error(stack_frame: &mut InterruptStackFrame) {
    // kill the process that did the bad thing
}

// The very convenient but very magical "x86-interrupt" calling convention. This
// calling convention causes LLVM to identify which registers are used in the
// implementation of the interrupt function and take care of saving those
// registers on the stack so that we may restore them later before returning
// from the interrupt. LLVM also uses the special `iret` instruction to
// conditionally restore the stack of the interrupted function.
extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: &mut InterruptStackFrame,
) {
    println!("CPU EXCEPTION CAUGHT: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn invalid_opcode_handler(
    stack_frame: &mut InterruptStackFrame,
) {
    // kill the process that did the bad thing
}

extern "x86-interrupt" fn general_protection_fault(
    stack_frame: &mut InterruptStackFrame,
    error_code: u64,
) {
    // kill the process that did the bad thing
}

extern "x86-interrupt" fn page_fault(
    stack_frame: &mut InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    // TODO: I think I need another case here to handle the MMU not having a
    // mapping for a virtual page but allocating it from available physical
    // memory instead.
    // This would make it similar to Linux's "overcommitting" behavior,
    // mapping virtual pages to physical memory on demand.
    // Alternatively we can have some fixed amount of memory for each process,
    // potentially with control over the data segment via some (s)brk-like
    // syscall
    if error_code.contains(PageFaultErrorCode::PROTECTION_VIOLATION) {
        // page fault caused by protection violation, kill the process
    } else {
        // read the page in from disk
    }
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

#[test_case]
fn test_breakpoint_exception() {
    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
}
