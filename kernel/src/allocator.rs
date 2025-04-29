//!
//!
//! We are allocating "From the back".
//! 1. Find the last `FreeSegment` in memory that can hold the allocation.
//! 2. Compute these three values, starting from the end of the free segment:
//!   - Size of the padding: (end of free segment - size_of(UsedSegment)) % alignment
//!   - Location of the new used segment header: end of free segment - (size_of(UsedSegment) + padding_size).
//!   - Location of the start of the allocated data: location of header - layout.size()
//! 3. Write the `UsedSegment` at its location
//! 4. Return a pointer to the location of the beginning of the newly allocated data.
//!
//! TODO::
//! - Explore how we could improve performances. Running through the list of free segments might take long.

use core::{
    alloc::GlobalAlloc,
    sync::atomic::{AtomicPtr, Ordering},
};

use bootloader_api::info::MemoryRegionKind;

/// This is the header stored memory in order to track a segment of unused memory.
#[repr(C)]
struct FreeSegment {
    /// Size of the free segment. This excludes the size of the `Self` struct itself.
    size: usize,

    /// Pointer to the next free segment in memory.
    next_free: *mut FreeSegment,
}

impl FreeSegment {
    /// Returns the end of the free segment.
    ///
    /// This is the address of `self` + the size of `Self` + self.size
    pub fn get_end(&self) -> *const u8 {
        unsafe { ((self as *const FreeSegment).add(1) as *const u8).add(self.size) }
    }
}

///
/// In memory, we will store this struct like so:
/// | ... | data | UsedSegment | Padding | ... |
/// | ... ^      |
///     (ptr)
///
/// When in `dealloc()`, we receive a pointer to the beginning of the allocated data. We can then
/// just add `layout.size()` to get to the actual `UsedSegment` stored in memory.
#[repr(C)]
struct UsedSegment {
    /// Size of the used segment. This excludes the size of the `Self` struct itself.
    size: usize,

    /// Size of the padding used to respect alignment.
    align_padding: usize,
}

impl UsedSegment {
    /// Returns the size of the whole used segment as a number of bytes.
    ///
    /// The size is: size of the data + size of the `UsedSegment` struct + size of the padding
    pub fn whole_size(&self) -> usize {
        self.size + core::mem::size_of::<Self>() + self.align_padding
    }
}

/// NOTE: We might need to add a lock to this struct to make it thread-safe.
pub struct Allocator {
    first_free: AtomicPtr<FreeSegment>,
}

#[global_allocator]
static ALLOC: Allocator = Allocator::new();

impl Allocator {
    pub const fn new() -> Self {
        Self {
            first_free: AtomicPtr::new(core::ptr::null_mut()),
        }
    }
}

/// This runs through the mapped memory regions in order to find the biggest one that we can use
/// in our allocator.
pub fn init(boot_info: &bootloader_api::BootInfo) {
    assert_eq!(
        core::mem::size_of::<FreeSegment>(),
        core::mem::size_of::<UsedSegment>()
    );
    let mut head: *mut FreeSegment = core::ptr::null_mut();
    let mut tail: *mut FreeSegment = core::ptr::null_mut();

    // We only work using mapped physical memory.
    let bootloader_api::info::Optional::Some(physical_memory_offset) =
        boot_info.physical_memory_offset
    else {
        panic!("Physical memory is not mapped !!");
    };

    println!("\n----- Allocator Initialization -----");
    println!("Physical memory offset: {}", physical_memory_offset);

    // Get the kernel section because we can't use memory that overlaps with it.
    let kernel_start = boot_info.kernel_addr;
    let kernel_len = boot_info.kernel_len;
    println!(
        "[{} -> {} ({} Mb)] Kernel",
        kernel_start,
        kernel_start + kernel_len,
        kernel_len / 1024 / 1024
    );

    for region in boot_info.memory_regions.iter() {
        // Only consider usable memory regions
        if region.kind != MemoryRegionKind::Usable {
            continue;
        }

        // Skip the region if it collides with the region used by the kernel.
        if region.end <= (kernel_start + kernel_len) {
            println!(
                "[{} -> {} ({} Mb)] kind: {:?} - Collides with kernel, skipping...",
                region.start,
                region.end,
                (region.end - region.start) / 1024 / 1024,
                region.kind
            );
            continue;
        }

        println!(
            "[{} -> {} ({} Mb)] kind: {:?}",
            region.start,
            region.end,
            (region.end - region.start) / 1024 / 1024,
            region.kind
        );

        // Write a `FreeSegment` to the region we found.
        let segment: *mut FreeSegment = (region.start + physical_memory_offset) as *mut FreeSegment;
        unsafe {
            segment.write(FreeSegment {
                size: (region.end - region.start) as usize - core::mem::size_of::<FreeSegment>(),
                next_free: core::ptr::null_mut(),
            });
        }

        // Insert at the end of the linked list.
        if head.is_null() {
            head = segment;
            tail = segment;
        } else {
            unsafe {
                assert!(
                    segment > (*tail).next_free,
                    "Wtf, memory regions are not ordered"
                );
                (*tail).next_free = segment;
            }
            tail = segment;
        }
    }

    // FIXME: Here, we make sure we found a single memory region that we can use.
    unsafe {
        assert!((*head).next_free.is_null());
    }

    println!("Allocator Initialization done. HEAD = {:?}\n", head);

    ALLOC.first_free.store(head, Ordering::Relaxed);
}

pub fn print_free_segments() {
    let mut count = 0;
    let mut cursor: *mut FreeSegment = ALLOC.first_free.load(Ordering::Relaxed);

    println!("----- List of Mapped FreeSegment -----");
    while !cursor.is_null() {
        count += 1;
        println!(
            "Region #{}: [{:?} -> {:?} ({} Mb)] Mapped & free",
            count,
            cursor,
            (*cursor).get_end(),
            (*cursor).size / 1024 / 1024
        );

        unsafe {
            cursor = (*cursor).next_free;
        }
    }

    println!("Total number of mapped regions: {}\n", count);
}

unsafe fn clean_free_segment_list(head: *mut FreeSegment) {
    let mut cursor = head;

    while !cursor.is_null() {
        if core::ptr::eq((*cursor).get_end(), (*cursor).next_free as *const u8) {
            cursor.write(FreeSegment {
                size: (*cursor).size
                    + core::mem::size_of::<FreeSegment>()
                    // Safety: `cursor.next_free` is not null.
                    + (*(*cursor).next_free).size,
                next_free: (*(*cursor).next_free).next_free,
            });

            continue;
        }

        cursor = (*cursor).next_free;
    }
}

unsafe fn insert_new_segment(head: *mut FreeSegment, new_segment: *mut FreeSegment) {
    let mut cursor = head;

    while !cursor.is_null() {
        assert!(cursor < new_segment);

        if (*cursor).next_free.is_null() || new_segment < (*cursor).next_free {
            (*new_segment).next_free = (*cursor).next_free;
            (*cursor).next_free = new_segment;
            return;
        }

        cursor = (*cursor).next_free;
    }

    // We didn't insert before so we must have a new head.
    assert!(head.is_null());
    ALLOC.first_free.store(new_segment, Ordering::Relaxed);
}

unsafe fn find_last_big_enough(
    head: *mut FreeSegment,
    layout: core::alloc::Layout,
) -> Option<*mut FreeSegment> {
    let mut cursor = head;
    let mut last = core::ptr::null_mut();

    while !cursor.is_null() {
        let segment_end = (*cursor).get_end();
        let data_start = segment_end.sub(layout.size());
        let padding_size = (data_start as usize) % layout.align();
        let segment_start = data_start.sub(padding_size + core::mem::size_of::<UsedSegment>());

        // We found a big enough segment
        if segment_start < (*cursor).get_end() {
            last = cursor;
        }

        cursor = (*cursor).next_free;
    }

    if !last.is_null() {
        return Some(last);
    }

    None
}

/// Returns the start of the newly allocated memory.
///
/// `free_segment` points to the beginning of the big-enough free memory region that we'll use.
///
unsafe fn write_used_segment(
    free_segment: *mut FreeSegment,
    layout: core::alloc::Layout,
) -> *mut u8 {
    let header_start = (*free_segment)
        .get_end()
        .sub(core::mem::size_of::<UsedSegment>());

    let padding_size = (header_start.sub(layout.size()) as usize) % layout.align();
    let header_start = header_start.sub(padding_size);

    let data_start = header_start.sub(layout.size());

    let used = header_start as *mut UsedSegment;
    (*used) = UsedSegment {
        size: layout.size(),
        align_padding: padding_size,
    };

    (*free_segment).size -= (*used).whole_size();

    data_start as *mut u8
}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let Some(last_big) = find_last_big_enough(self.first_free.load(Ordering::Relaxed), layout)
        else {
            panic!("No free memory found.")
        };

        write_used_segment(last_big, layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let used = (ptr.add(layout.size())) as *mut UsedSegment;

        let new_free = FreeSegment {
            size: (*used).size + (*used).align_padding,
            next_free: core::ptr::null_mut(),
        };
        let ptr = ptr as *mut FreeSegment;
        ptr.write(new_free);

        insert_new_segment(self.first_free.load(Ordering::Relaxed), ptr);

        clean_free_segment_list(self.first_free.load(Ordering::Relaxed));
    }
}
