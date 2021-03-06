// memory.rs - Page tables and stuff

use x86_64::{
    structures::paging::{
        OffsetPageTable,
        FrameAllocator,
        PageTable,
        PhysFrame,
        Size4KiB,
        Mapper,
        Page
    },
    VirtAddr,
    PhysAddr,
};
use bootloader::bootinfo::{
    MemoryMap,
    MemoryRegionType
};


//////////////////////////////
// Data Structures and Types
//////////////////////////////

// Frame Allocator reutrns usable frames forom the boot loaders memory map
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    // Create Frame Allocator from MemoryMap
    //
    // Unsafe! Caller must gurantee the memory map is valid! I.E. All frames 
    // that are marked as 'USABLE' must be unusued.
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0, 
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // Get usable regions
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(
            |r| r.region_type == MemoryRegionType::Usable
        );

        // Map each reagion to its address range
        let addr_ranges = usable_regions.map(
            |r| r.range.start_addr()..r.range.end_addr()
        );

        // Transform to an interator of frame start addresses
        let frame_addrs = addr_ranges.flat_map(
            |r| r.step_by(4096)
        );

        // Create Physical Frame types from start address
        frame_addrs.map(
            |addr| PhysFrame::containing_address(PhysAddr::new(addr))
        )
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}


// A FrameAllocator that always returns `None`.
pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        None
    }
}


//////////////////////////////
// API
//////////////////////////////


// Initialize a new Offset Page Table
//
// Unsafe! Caller must guarantee the the complete physical memory is mapped to 
// Virtual memory at the specified `physical_memory_offset`. 
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}


pub fn create_example_mapping(page: Page, mapper: &mut OffsetPageTable, frame_allocator: &mut impl FrameAllocator<Size4KiB>) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe {
        // FIXME: this is not safe, we do it only for testing
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    map_to_result.expect("map_to failed").flush();
}


unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (lvl_4_tbl_frame, _) = Cr3::read();

    let phys = lvl_4_tbl_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let pg_tbl_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *pg_tbl_ptr // unsafe
}

