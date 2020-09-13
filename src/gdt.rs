use lazy_static::lazy_static;
use x86_64::structures::gdt::{
    Descriptor, GlobalDescriptorTable, SegmentSelector,
};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();

        // Set up some memory to use as stack space for handling double faults.
        // The double fault entry in the IDT specifies DOUBLE_FAULT_IST_INDEX
        // as where the CPU can memory to push the interrupt handler's stack.
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            // we need static mut to make sure STACK isn't mapped to a read-only
            // elf segment and subsequently to a read-only memory page by the
            // bootloader
            // rust cannot guarantee race freedom in this case (i.e. 
            // it's a mutable global variable not protected via lock, prime real
            // estate for race conditions)

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;

            // stack grows down
            stack_end
        };
        tss
    };
}

// create a GDT
// - set our TSS segment in it with a separate stack for handling double faults
// - create a new GDT entry with permissions for running kernel code, then set
//   the CS register with selectors that point to our new GDT entry
lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (
            gdt,
            Selectors {
                code_selector,
                tss_selector,
            },
        )
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::segmentation::set_cs;
    use x86_64::instructions::tables::load_tss;

    let (gdt, selectors) = &*GDT;
    gdt.load();

    // the GDT has changed, so we need to make sure segment registers
    // point to the right descriptors in our (new) GDT
    // (the bootloader might give us some good defaults to start with)
    unsafe {
        set_cs(selectors.code_selector);
        load_tss(selectors.tss_selector);
    }
}
