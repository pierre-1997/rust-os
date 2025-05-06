#![allow(clippy::fn_to_numeric_cast)]

use crate::utils::bits::{GetBit, SetBit};
use core::{arch::asm, cell::OnceCell, fmt};

#[allow(unused)]
#[repr(align(16))]
struct AlignedGDT([SegmentDescriptor; 3]);

static mut GLOBAL_DESCRIPTOR_TABLE: AlignedGDT = AlignedGDT([
    SegmentDescriptor(0),
    SegmentDescriptor::kernel_mode_code_segment(),
    SegmentDescriptor::kernel_mode_data_segment(),
]);

/// Segment Descriptor (64bits)
///
/// |63                56|55           52|51          48|47                 40|
/// | Base (8 of 32bits) | Flags (4bits) | Limit(4bits) | Access Byte (8bits) |
/// |39                 16|15                    0|
/// | Base(24 of 32 bits) | Limit (16 of 20 bits) |
#[repr(C)]
struct SegmentDescriptor(u64);

impl fmt::Display for SegmentDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Segment at virt. {:p} (raw = {:#016x}):", self, self.0)?;
        writeln!(f, "Flags:\n{}---------", self.flags())?;
        writeln!(f, "Access Byte:\n{}----------", self.access_byte())?;
        writeln!(f, "Base: {}", self.base())?;
        writeln!(f, "Limit: {}", self.limit())?;

        Ok(())
    }
}

impl SegmentDescriptor {
    fn flags(&self) -> Flags {
        let value = self.0;

        Flags(value.get_bits(55, 4) as u8)
    }

    #[cfg(test)]
    fn flags_mut(&mut self) -> &mut Flags {
        let ptr = &mut self.0 as *mut u64 as *mut u8;

        unsafe {
            // NOTE: We are on little endian !!
            let ptr = ptr.add(6) as *mut Flags;

            &mut *ptr
        }
    }

    fn access_byte(&self) -> AccessByte {
        AccessByte((self.0 >> 40) as u8)
    }

    #[cfg(test)]
    fn access_byte_mut(&mut self) -> &mut AccessByte {
        let ptr = &mut self.0 as *mut u64 as *mut u8;

        unsafe {
            // NOTE: We are on little endian !!
            let ptr = ptr.add(5) as *mut AccessByte;
            &mut *ptr
        }
    }

    fn base(&self) -> u32 {
        let upper = self.0.get_bits(63, 8);
        let rest = self.0.get_bits(39, 24);

        ((upper << 24) | rest) as u32
    }

    /// In 64-bit mode, base is ignored.
    fn _set_base(&mut self) {
        unimplemented!()
    }

    fn limit(&self) -> u32 {
        let upper = self.0.get_bits(51, 4);
        let rest = self.0.get_bits(15, 16);

        ((upper << 16) | rest) as u32
    }

    /// In 64-bit mode, limit is ignored.
    fn _set_limit(&mut self) {
        unimplemented!()
    }

    // Pre-calculated and taken from the wiki.
    const fn kernel_mode_code_segment() -> Self {
        SegmentDescriptor(0x00A09A0000000000)
    }

    // Pre-calculated and taken from the wiki.
    const fn kernel_mode_data_segment() -> Self {
        SegmentDescriptor(0x00C0920000000000)
    }
}

/// Descriptor Privilege Level field.
///
/// These are CPU rings.
#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
enum Dpl {
    Ring0 = 0,
    Ring1,
    Ring2,
    Ring3,
}

impl TryFrom<u8> for Dpl {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Dpl::Ring0),
            1 => Ok(Dpl::Ring1),
            2 => Ok(Dpl::Ring2),
            3 => Ok(Dpl::Ring3),
            _ => Err("Totally unreachable unless GetBit is not implemented correctly."),
        }
    }
}

/// Access Byte (8bits)
///
/// |7|6 5|4|3|2 |1 |0|
/// |P|DPL|S|E|DC|RW|A|
#[repr(C)]
struct AccessByte(u8);

impl fmt::Display for AccessByte {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "P: {}", self.p())?;
        writeln!(f, "DPL: {:?}", self.dpl())?;
        writeln!(f, "S: {}", self.s())?;
        writeln!(f, "E: {}", self.e())?;
        writeln!(f, "DC: {}", self.dc())?;
        writeln!(f, "RW: {}", self.rw())?;
        writeln!(f, "A: {}", self.a())?;

        Ok(())
    }
}

impl AccessByte {
    fn p(&self) -> bool {
        self.0.get_bit(7)
    }

    #[cfg(test)]
    fn set_p(&mut self, value: bool) {
        self.0.set_bit(7, value);
    }

    fn dpl(&self) -> Dpl {
        self.0
            .get_bits(6, 2)
            .try_into()
            .expect("Unreachable expect.")
    }

    #[cfg(test)]
    fn set_dpl(&mut self, value: Dpl) {
        self.0.set_bits(6, 2, value as u8)
    }

    fn s(&self) -> bool {
        self.0.get_bit(4)
    }

    #[cfg(test)]
    fn set_s(&mut self, value: bool) {
        self.0.set_bit(4, value);
    }

    fn e(&self) -> bool {
        self.0.get_bit(3)
    }

    #[cfg(test)]
    fn set_e(&mut self, value: bool) {
        self.0.set_bit(3, value);
    }

    fn dc(&self) -> bool {
        self.0.get_bit(2)
    }

    fn rw(&self) -> bool {
        self.0.get_bit(1)
    }

    #[cfg(test)]
    fn set_rw(&mut self, value: bool) {
        self.0.set_bit(1, value);
    }

    fn a(&self) -> bool {
        self.0.get_bit(0)
    }
}

/// Flags(4bits)
///
/// |3|2 |1|       0|
/// |G|DB|L|Reserved|
///
/// NOTE: Actually only uses the first 4 bits.
#[repr(C)]
struct Flags(u8);

impl fmt::Display for Flags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "G: {}", self.g())?;
        writeln!(f, "DB: {}", self.db())?;
        writeln!(f, "L: {}", self.l())?;

        Ok(())
    }
}

impl Flags {
    fn g(&self) -> bool {
        self.0.get_bit(7)
    }

    #[cfg(test)]
    fn set_g(&mut self, value: bool) {
        self.0.set_bit(7, value);
    }

    fn db(&self) -> bool {
        self.0.get_bit(6)
    }

    #[cfg(test)]
    fn set_db(&mut self, value: bool) {
        self.0.set_bit(6, value)
    }

    fn l(&self) -> bool {
        self.0.get_bit(5)
    }

    #[cfg(test)]
    fn set_l(&mut self, value: bool) {
        self.0.set_bit(5, value);
    }

    fn _reserved(&self) -> bool {
        unreachable!()
    }
}

/// GDT descriptor.
#[repr(C, packed)]
#[derive(Debug)]
pub struct Gdtr {
    limit: u16,
    base: u64,
}

impl Gdtr {
    /// Prints the GDT
    pub fn print(print_entries: bool) {
        let mut gdtr = Gdtr { limit: 0, base: 0 };
        unsafe {
            asm!(
                "sgdt [{gdtr}]",
                gdtr = in(reg) &mut gdtr,
                options(nostack, preserves_flags)
            );
        }

        let limit = gdtr.limit;
        let base = gdtr.base;

        println!("GDT: limit = {} + 1 bytes, base = {:#x}", limit, base);

        let mut gdt = base as *mut u64;

        // We're in 64-bit, so I'm hardcoding this 8.
        let nb_entries = (gdtr.limit + 1) / 8;
        println!("Number of entries in the GDT: {}", nb_entries);

        if print_entries {
            for i in 0..nb_entries {
                println!("Entry #{}: {:p} = {:#016X}", i, gdt, *gdt);
                println!("{}", SegmentDescriptor(*gdt));

                // TODO: The last one must be the TSS?

                // Go to the next entry
                gdt = unsafe { gdt.add(1) };
            }
        }
    }
}

extern "x86-interrupt" fn interrupt_handler() {
    unsafe {
        asm!("nop");
    }
    println!("INTERRRRUPPPPTTT");
    panic!("INTERRRRUPPPPTTT");
}

// FIXME: Set at compile time, is it correct ?
static INTERRUPT_DESCRIPTOR_TABLE: Idt = Idt {
    handlers: OnceCell::new(),
};

struct Idt {
    handlers: OnceCell<[GateDescriptor; 256]>,
}
// Safety: We're in a single-threaded environment for now.
unsafe impl Sync for Idt {}

// Interrupt Table Descriptor
#[repr(C, packed)]
pub struct Idtr {
    limit: u16,
    base: u64,
}

impl Idtr {
    pub fn print() {
        let mut idtr = Idtr { limit: 0, base: 0 };
        unsafe {
            asm!(
                    "sidt [{idtr}]",
                    idtr = in(reg) &mut idtr,
                options(nostack, preserves_flags)
            );
        }

        // let ptr = &idtr as *const IDTR as *const u8;
        // let limit = unsafe { *(ptr as *const u16) };
        let limit = idtr.limit;
        let base = idtr.base;
        // let base = unsafe { *(ptr.add(2) as *const u64) };

        println!("IDT: limit = {} + 1 bytes, base = {:#X}", limit, base);
        let nb_entries = (idtr.limit + 1) / 16;
        println!("Number of entries in the IDT: {}", nb_entries);
    }
}

/// These are 2 kinds of interrupts.
#[derive(Debug, PartialEq, Eq)]
enum GateType {
    Interrupt,
    Trap,
}

impl TryFrom<u8> for GateType {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x0E => Ok(GateType::Interrupt),
            0x0F => Ok(GateType::Trap),
            _ => Err("Invalid value for a GateType."),
        }
    }
}

impl From<GateType> for u8 {
    fn from(value: GateType) -> Self {
        match value {
            GateType::Interrupt => 0x0E,
            GateType::Trap => 0x0F,
        }
    }
}

/// A entry in the IDTR that gives points to the function to run on interrupt.
///
/// First u64:
/// |127            96|95                 64|
/// |Reserved (32bits)|Offset (32 of 64bits)|
///
/// Second u64:
/// |63                 48|47|46       45|44|43             40|39            35|34       32|
/// |Offset (16 of 64bits)|P |DPL (2bits)|0 |Gate Type (4bits)|Reserved (5bits)|IST (3bits)|
/// |31                     16|15                  0|
/// |Segment Selector (16bits)|Offset (16 of 64bits)|
#[derive(Debug, Default, Clone, Copy)]
struct GateDescriptor(u64, u64);

impl fmt::Display for GateDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Raw {:#X},{:#X}", self.0, self.1)?;
        writeln!(f, "Offset: {:#X}", self.offset())?;
        writeln!(f, "P: {}", self.p())?;
        writeln!(f, "DPL: {:?}", self.dpl())?;
        writeln!(f, "Gate Type: {:?}", self.gate_type())?;
        writeln!(f, "IST: {:#X}", self.ist())?;
        writeln!(f, "Segment Selector: {:#X}", self.selector())?;

        Ok(())
    }
}

impl GateDescriptor {
    fn new(fn_ptr: u64, selector: u16, dpl: Dpl, gtype: GateType) -> Self {
        let mut descriptor = GateDescriptor(0, 0);

        descriptor.set_offset(fn_ptr);
        descriptor.set_selector(selector);
        descriptor.set_dpl(dpl);
        descriptor.set_gate_type(gtype);
        descriptor.set_p(true);

        descriptor
    }

    fn offset(&self) -> u64 {
        let lower_first = self.0.get_bits(63, 16);
        let lower_rest = self.0.get_bits(15, 16);
        let upper = self.1.get_bits(31, 32);

        (upper << 32) | (lower_first << 16) | lower_rest
    }

    fn set_offset(&mut self, offset: u64) {
        self.1.set_bits(31, 32, offset >> 32);
        self.0.set_bits(63, 16, offset.get_bits(31, 16));
        self.0.set_bits(15, 16, offset.get_bits(15, 16));
    }

    fn p(&self) -> bool {
        self.0.get_bit(47)
    }

    fn set_p(&mut self, value: bool) {
        self.0.set_bit(47, value);
    }

    fn dpl(&self) -> Dpl {
        (self.0.get_bits(46, 2) as u8)
            .try_into()
            .expect("Invalid DPL found in GateDescriptor.")
    }

    fn set_dpl(&mut self, dpl: Dpl) {
        self.0.set_bits(46, 2, dpl as u64);
    }

    fn gate_type(&self) -> GateType {
        GateType::try_from(self.0.get_bits(43, 4) as u8)
            .expect("Invalid GateType found in GateDescriptor.")
    }

    fn set_gate_type(&mut self, gtype: GateType) {
        self.0.set_bits(43, 4, u8::from(gtype) as u64);
    }

    /// Offset to the IST (Interrupt Stack Table) stored in the TSS (Task State Segment). If set
    /// to 0, means the IST is not used.
    fn ist(&self) -> u8 {
        self.0.get_bits(34, 3) as u8
    }

    #[cfg(test)]
    fn set_ist(&mut self, value: u8) {
        self.0.set_bits(34, 3, value as u64);
    }

    fn selector(&self) -> u16 {
        self.0.get_bits(31, 16) as u16
    }

    fn set_selector(&mut self, selector: u16) {
        self.0.set_bits(31, 16, selector as u64);
    }
}

pub fn init() {
    // 1. Disable interrupts
    unsafe {
        asm!("cli", options(nostack, preserves_flags));
    }

    // Check that interrupts were correctly disabled
    unsafe {
        let mut cpu_flags: i32;
        asm!(
            "pushf",
            "pop {cpu_flags:r}",
            "push {cpu_flags:r}",
            "popf",
            cpu_flags = out(reg) cpu_flags
        );

        assert_eq!(
            (cpu_flags >> 9) & 1,
            0,
            "Disabling interrupts did not work."
        );
    }

    // 2. Tell the CPU where the Global Descriptor Table (GDT) is
    let gdtr = Gdtr {
        limit: (3 * 8 - 1) as u16,
        base: &raw const GLOBAL_DESCRIPTOR_TABLE as *const _ as u64,
    };
    unsafe {
        asm!(
            "lgdt [{}]",
            in(reg) &gdtr, options(nostack, preserves_flags)
        );
    }

    // Read it to check that it worked.
    Gdtr::print(false);

    // 3. Reload segment registers
    unsafe {
        asm!(
            // Reload the CS (Code Segment) register:
            // 0x08 is the selector for the code segment
            "push 0x08",
            "lea rax, [rip + 2f]",
            "push rax",
            "retfq",
            // Reload the other segments:
            "2:",
            // 0x10 is the selector for the data segment
            "mov ax, 0x10",
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            "mov ss, ax",
            options(nostack, preserves_flags)
        );
    };

    // 4. Disable PICs handling interrupts (because they were set in BIOS and conflict)
    unsafe {
        crate::io::outb(0x21, 0xFF); // PIC1 mask all
        crate::io::outb(0xA1, 0xFF); // PIC2 mask all
    }

    // 5. Initialize the starting Interrupt Descriptor Table (IDT)
    let _ = INTERRUPT_DESCRIPTOR_TABLE
        .handlers
        .set(core::array::from_fn(|i| {
            if i == 2 {
                GateDescriptor::new(
                    interrupt_handler as u64,
                    0x08,
                    Dpl::Ring0,
                    GateType::Interrupt,
                )
            } else if i == 3 {
                GateDescriptor::new(interrupt_handler as u64, 0x08, Dpl::Ring0, GateType::Trap)
            } else {
                GateDescriptor::default()
            }
        }));

    // 7. Tell the CPU where the Interrupt Descriptor Table (IDT) is
    let handlers = INTERRUPT_DESCRIPTOR_TABLE
        .handlers
        .get()
        .expect("INTERRUPT_DESCRIPTOR_TABLE should have been set by now");
    let idtr = Idtr {
        limit: (handlers.len() * 16 - 1) as u16,
        base: handlers.as_ptr() as *const u64 as u64,
    };
    unsafe {
        asm!(
            "lidt [{idt_ptr}]",
            idt_ptr = in(reg) &idtr,
            options(nostack, preserves_flags)
        );
    }

    // Print it to check that it worked
    Idtr::print();

    // 7. Re-enable interrupts
    unsafe {
        asm!("sti", options(nostack, preserves_flags));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::TestCase;

    #[test_case]
    fn test_set_flags() -> TestCase {
        TestCase {
            name: "Test set flags",
            test: || {
                let mut f = Flags(0);
                f.set_g(true);
                assert_eq!(f.0, 0x80);
                f.set_db(true);
                assert_eq!(f.0, 0xC0);
                f.set_l(true);
                assert_eq!(f.0, 0xE0);
            },
        }
    }

    #[test_case]
    fn test_set_access_byte() -> TestCase {
        TestCase {
            name: "Test set AccessByte",
            test: || {
                let mut ab = AccessByte(0);
                ab.set_p(true);
                assert_eq!(ab.0, 0x80);
                ab.set_dpl(Dpl::Ring3);
                assert_eq!(ab.0, 0xE0);
                ab.set_s(true);
                assert_eq!(ab.0, 0xF0);
            },
        }
    }

    #[test_case]
    fn test_set_access_byte() -> TestCase {
        TestCase {
            name: "Test getting Flags and AccessByte from SegmentDescriptor",
            test: || {
                let mut sd = SegmentDescriptor(0x00F0000000000000);
                let f = sd.flags();
                assert_eq!(f.0, 0x0F);
                let fmut = sd.flags_mut();
                assert_eq!(fmut.0, 0xF0);

                let mut sd = SegmentDescriptor(0x0000FF0000000000);
                let ab = sd.access_byte();
                assert_eq!(ab.0, 0xFF);
                let abmut = sd.access_byte_mut();
                assert_eq!(abmut.0, 0xFF);
            },
        }
    }

    #[test_case]
    fn test_init_gdt() -> TestCase {
        TestCase {
            name: "Test GDT initialization",
            test: || {
                assert_eq!(
                    SegmentDescriptor::kernel_mode_code_segment().0,
                    0x00A09A0000000000
                );
                assert_eq!(
                    SegmentDescriptor::kernel_mode_data_segment().0,
                    0x00C0920000000000
                );
            },
        }
    }

    #[test_case]
    fn test_gate_descriptor() -> TestCase {
        TestCase {
            name: "Test GateDescriptor by setting/getting fields",
            test: || {
                // Test offset
                let mut gd = GateDescriptor(0, 0);
                gd.set_offset(0x0123456789ABCDEF);
                assert_eq!(gd.offset(), 0x0123456789ABCDEF);

                let mut gd = GateDescriptor(0xFFFF00000000FFFF, 0xFFFFFFFF);
                assert_eq!(gd.offset(), 0xFFFFFFFFFFFFFFFF);

                // Test p
                gd.set_p(true);
                assert_eq!(gd.p(), true);

                // Test DPL
                gd.set_dpl(Dpl::Ring3);
                assert_eq!(gd.dpl(), Dpl::Ring3);

                // Test GateType
                gd.set_gate_type(GateType::Trap);
                assert_eq!(gd.gate_type(), GateType::Trap);

                // Test IST
                gd.set_ist(0x07);
                assert_eq!(gd.ist(), 0x07);

                // Test Segment Selector
                gd.set_selector(0xFFFF);
                assert_eq!(gd.selector(), 0xFFFF);
            },
        }
    }
}
