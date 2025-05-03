use crate::utils::bits::{GetBit, SetBit};
use alloc::vec::Vec;
use core::{arch::asm, cell::OnceCell, fmt, u64};

static GDT_TABLE: GdtTable = GdtTable {
    segments: OnceCell::new(),
};

struct GdtTable {
    pub segments: OnceCell<Vec<u64>>,
}

unsafe impl Sync for GdtTable {}

/* impl Deref for GdtTable {
    type Target = Vec<u64>;

    fn deref(&self) -> &Self::Target {
        &self.segments
    }
} */

/// Segment Descriptor
/// 63   56 	55   52 	51   48 	47   40 	39   32
/// Base Flags Limit Access Byte Base
/// 31   24 	 3   0  19   16 7   0  23   16
/// 31   16 	15   0
/// Base Limit
/// 15   0 	 15   0
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
}

/// Descriptor Privilege Level field.
///
/// These are CPU rings.
#[repr(u8)]
enum DPL {
    Ring0 = 0,
    Ring1,
    Ring2,
    Ring3,
}

impl fmt::Display for DPL {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DPL::Ring0 => "0",
                DPL::Ring1 => "1",
                DPL::Ring2 => "2",
                DPL::Ring3 => "3",
            }
        )
    }
}

impl TryFrom<u8> for DPL {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(DPL::Ring0),
            1 => Ok(DPL::Ring1),
            2 => Ok(DPL::Ring2),
            3 => Ok(DPL::Ring3),
            _ => Err("Totally unreachable unless GetBit is not implemented correctly."),
        }
    }
}

#[repr(u8)]
enum AccessByteType {
    LDT,
    Available,
    Busy,
}

impl fmt::Display for AccessByteType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                AccessByteType::LDT => "LDT",
                AccessByteType::Available => "Available",
                AccessByteType::Busy => "Busy",
            }
        )
    }
}

impl TryFrom<u8> for AccessByteType {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x02 => Ok(AccessByteType::LDT),
            0x09 => Ok(AccessByteType::Available),
            0x0B => Ok(AccessByteType::Busy),
            _ => Err("Invalid AccessByteType"),
        }
    }
}

impl From<AccessByteType> for u8 {
    fn from(ttype: AccessByteType) -> Self {
        match ttype {
            AccessByteType::LDT => 0x02,
            AccessByteType::Available => 0x09,
            AccessByteType::Busy => 0x0B,
        }
    }
}

/// Access Byte
///
/// 7 	6 	5 	4 	3 	2 	1 	0
/// P 	DPL 	S 	E 	DC 	RW 	A
#[repr(C)]
struct AccessByte(u8);

impl fmt::Display for AccessByte {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "P: {}", self.p())?;
        writeln!(f, "DPL: {}", self.dpl())?;
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

    fn set_p(&mut self, value: bool) {
        self.0.set_bit(7, value);
    }

    fn dpl(&self) -> DPL {
        self.0
            .get_bits(6, 2)
            .try_into()
            .expect("Unreachable expect.")
    }

    fn set_dpl(&mut self, value: DPL) {
        self.0.set_bits(6, 2, value as u8)
    }

    fn s(&self) -> bool {
        self.0.get_bit(4)
    }

    fn set_s(&mut self, value: bool) {
        self.0.set_bit(4, value);
    }

    fn e(&self) -> bool {
        self.0.get_bit(3)
    }

    fn set_e(&mut self, value: bool) {
        self.0.set_bit(3, value);
    }

    fn dc(&self) -> bool {
        self.0.get_bit(2)
    }

    fn rw(&self) -> bool {
        self.0.get_bit(1)
    }

    fn set_rw(&mut self, value: bool) {
        self.0.set_bit(1, value);
    }

    fn a(&self) -> bool {
        self.0.get_bit(0)
    }

    /// In Long Mode, the last 4 bits of an `AccessByte` contains the type.
    fn ttype(&self) -> AccessByteType {
        self.0
            .get_bits(3, 4)
            .try_into()
            .expect("Unreachable expect.")
    }

    fn set_type(&mut self, ttype: AccessByteType) {
        self.0.set_bits(3, 4, ttype.into());
    }
}

/// Flags
/// 3 	2 	1 	0
/// G 	DB 	L 	Reserved
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

    fn set_g(&mut self, value: bool) {
        self.0.set_bit(7, value);
    }

    fn db(&self) -> bool {
        self.0.get_bit(6)
    }

    fn set_db(&mut self, value: bool) {
        self.0.set_bit(6, value)
    }

    fn l(&self) -> bool {
        self.0.get_bit(5)
    }

    fn set_l(&mut self, value: bool) {
        self.0.set_bit(5, value);
    }

    fn _reserved(&self) -> bool {
        unreachable!()
    }
}

/// Task State Segment
///
/// 64-bit System Segment Descriptor
#[allow(unused)]
struct TSS(u64, u64);

#[allow(unused)]
impl TSS {
    pub fn long_mode() -> Self {
        let mut tss = TSS(0, 0);

        // Access byte = 0x89
        tss.access_byte_mut().set_p(true);
        tss.access_byte_mut().set_type(AccessByteType::Available);

        // Base = &TSS

        // Limit = sizeof(TSS) - 1

        tss
    }

    fn _reserved(&self) {
        unreachable!()
    }

    fn flags(&self) -> Flags {
        Flags((self.1 >> 48) as u8)
    }

    fn flags_mut(&mut self) -> &mut Flags {
        let ptr = self.1 as *mut u64 as *mut u8;

        unsafe {
            let ptr = ptr.add(6) as *mut Flags;

            &mut *ptr
        }
    }

    fn access_byte(&self) -> AccessByte {
        AccessByte((self.1 >> 40) as u8)
    }

    fn access_byte_mut(&mut self) -> &mut AccessByte {
        let ptr = &mut self.1 as *mut u64 as *mut u8;

        unsafe {
            let ptr = ptr.add(5) as *mut AccessByte;
            &mut *ptr
        }
    }

    fn base(&self) -> u64 {
        let upper = self.0.get_bits(31, 32);

        let lower_first = self.1.get_bits(63, 8);
        let lower_rest = self.1.get_bits(39, 24);

        (upper << 32) | ((lower_first << 24) | lower_rest)
    }

    fn set_base(&mut self, value: u64) {
        let upper = value.get_bits(63, 32);
        self.0.set_bits(31, 32, upper);

        let lower_first = value.get_bits(31, 8);
        let lower_rest = value.get_bits(23, 24);
        self.1.set_bits(63, 8, lower_first);
        self.1.set_bits(39, 24, lower_rest);
    }

    fn limit(&self) -> u32 {
        let upper = self.1.get_bits(51, 4);
        let lower = self.1.get_bits(15, 16);

        ((upper << 16) | lower) as u32
    }

    fn set_limit(&mut self, value: u32) {
        let upper = value.get_bits(19, 4) as u64;
        self.1.set_bits(51, 4, upper);

        let lower = value.get_bits(15, 16) as u64;
        self.1.set_bits(15, 16, lower);
    }
}

impl SegmentDescriptor {
    fn kernel_mode_code_segment() -> Self {
        let mut sd = SegmentDescriptor(0);

        // Access Byte = 0x9A
        sd.access_byte_mut().set_p(true);
        sd.access_byte_mut().set_s(true);
        sd.access_byte_mut().set_e(true);
        sd.access_byte_mut().set_rw(true);

        // Flags = 0xA
        sd.flags_mut().set_g(true);
        sd.flags_mut().set_l(true);

        sd
    }

    #[allow(unused)]
    fn kernel_mode_data_segment() -> Self {
        let mut sd = SegmentDescriptor(0);

        // Access Byte = 0x92
        sd.access_byte_mut().set_p(true);
        sd.access_byte_mut().set_s(true);
        sd.access_byte_mut().set_rw(true);

        // Flags = 0xC
        sd.flags_mut().set_g(true);
        sd.flags_mut().set_db(true);

        sd
    }

    #[allow(unused)]
    fn user_mode_code_segment() -> Self {
        let mut sd = SegmentDescriptor(0);

        // Access Byte = 0xFA
        sd.access_byte_mut().set_p(true);
        sd.access_byte_mut().set_dpl(DPL::Ring3);
        sd.access_byte_mut().set_s(true);
        sd.access_byte_mut().set_e(true);
        sd.access_byte_mut().set_rw(true);

        // Flags = 0xA
        sd.flags_mut().set_g(true);
        sd.flags_mut().set_l(true);

        sd
    }

    #[allow(unused)]
    fn user_mode_data_segment() -> Self {
        let mut sd = SegmentDescriptor(0);

        // Access Byte = 0xF2
        sd.access_byte_mut().set_p(true);
        sd.access_byte_mut().set_dpl(DPL::Ring3);
        sd.access_byte_mut().set_s(true);
        sd.access_byte_mut().set_rw(true);

        // Flags = 0xC
        sd.flags_mut().set_g(true);
        sd.flags_mut().set_db(true);

        sd
    }

    fn print() {}

    fn load_table() {
        unsafe {
            asm!("lgdt");
            /*
                        gdtr DW 0 ; For limit storage
                 DQ 0 ; For base storage

            setGdt:
               MOV   [gdtr], DI
               MOV   [gdtr+2], RSI
               LGDT  [gdtr]
               RET

                        */
        }
    }

    fn update_registers() {

        /*
               reloadSegments:
           ; Reload CS register:
           PUSH 0x08                 ; Push code segment to stack, 0x08 is a stand-in for your code segment
           LEA RAX, [rel .reload_CS] ; Load address of .reload_CS into RAX
           PUSH RAX                  ; Push this value to the stack
           RETFQ                     ; Perform a far return, RETFQ or LRETQ depending on syntax
        .reload_CS:
           ; Reload data segment registers
           MOV   AX, 0x10 ; 0x10 is a stand-in for your data segment
           MOV   DS, AX
           MOV   ES, AX
           MOV   FS, AX
           MOV   GS, AX
           MOV   SS, AX
           RET
                */
    }
}

/// GDT descriptor.
#[repr(C, packed)]
#[derive(Debug)]
pub struct GDTR {
    limit: u16,
    base: u64,
}

pub fn init() {
    // 0. Generate the table we will load, and get a pointer to it.
    // let tss = TSS::long_mode();
    if let Err(e) = GDT_TABLE.segments.set(
        [
            // The first entry is null
            0,
            SegmentDescriptor::kernel_mode_code_segment().0,
            SegmentDescriptor::kernel_mode_data_segment().0,
            // tss.0,
            // tss.1,
        ]
        .to_vec(),
    ) {
        panic!("Failed to set GDT: {e:?}");
    }
    let segments = GDT_TABLE
        .segments
        .get()
        .expect("GDT_TABLE should have been initialized by now");

    let base_ptr: *const u64 = segments.as_ptr();
    let limit = segments.len() * 8 - 1;

    let gdtr = GDTR {
        limit: limit as u16,
        base: base_ptr as u64,
        // base: 0x1122334455667788,
    };

    println!(
        "About to write: limit = {} - base: {:#X}",
        limit as u16, base_ptr as u64
    );

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

    // 2. Upload the table
    unsafe {
        asm!(
            "lgdt [{}]",
            in(reg) &gdtr, options(nostack, preserves_flags)
        );
    }

    // Read it to check that it worked.
    GDTR::print();

    // 3. Tell the CPU where the Table is
    // 4. Reload segment registers
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

    // Re-enable interrupts

    // Maybe:
    // 5. LDT
    // 6. IDT
}

impl GDTR {
    /// Prints the GDT
    pub fn print() {
        let mut gdtr = GDTR { limit: 0, base: 0 };
        unsafe {
            asm!(
                "sgdt [{}]",
                in(reg) &mut gdtr,
                options(nostack, preserves_flags)
            );
        }

        let ptr = &gdtr as *const GDTR as *const u8;
        let limit = unsafe { *(ptr as *const u16) };
        let base = unsafe { *(ptr.add(2) as *const u64) };

        println!("GDT: limit = {} bytes, base = {:#x}", limit + 1, base);

        let mut gdt = base as *mut u64;

        // We're in 64-bit, so I'm hardcoding this 8.
        let nb_words = (gdtr.limit + 1) / 8;
        println!("Number of entries in the GDT: {}", nb_words);

        for i in 0..nb_words {
            println!("Entry #{}: {:p} = {:#016X}", i, gdt, *gdt);
            println!("{}", SegmentDescriptor(*gdt));

            // TODO: The last one must be the TSS?

            // Go to the next entry
            gdt = unsafe { gdt.add(1) };
        }
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
                ab.set_dpl(DPL::Ring3);
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
                assert_eq!(f.0, 0xF0);
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
    fn test_tss_base() -> TestCase {
        TestCase {
            name: "Test getting/setting the base of a TSS",
            test: || {
                let mut tss = TSS(0x0000000012345678, 0x1200003456780000);
                assert_eq!(tss.base(), 0x1234567812345678);

                tss.set_base(0xdeadbeefdeadbeef);
                assert_eq!(tss.base(), 0xdeadbeefdeadbeef);
            },
        }
    }

    #[test_case]
    fn test_tss_limit() -> TestCase {
        TestCase {
            name: "Test getting/setting the limit of a TSS",
            test: || {
                let mut tss = TSS(0, 0x0001000000002345);
                assert_eq!(tss.limit(), 0x012345);

                tss.set_limit(0x0deadb);
                assert_eq!(tss.limit(), 0x0deadb);
            },
        }
    }

    #[test_case]
    fn test_tss_access_byte() -> TestCase {
        TestCase {
            name: "Test getting/setting the access byte of a TSS",
            test: || {
                let mut tss = TSS(0, 0x0000FF0000000000);
                assert_eq!(tss.access_byte().0, 0xFF);
                assert_eq!(tss.access_byte_mut().0, 0xFF);

                let mut tss = TSS(0, 0);
                tss.access_byte_mut().set_p(true);
                assert_eq!(tss.access_byte().p(), true);
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
                assert_eq!(
                    SegmentDescriptor::user_mode_code_segment().0,
                    0x00A0FA0000000000
                );
                assert_eq!(
                    SegmentDescriptor::user_mode_data_segment().0,
                    0x00C0F20000000000
                );
            },
        }
    }
}
