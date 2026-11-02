//! Global allocator: Talc, with a memory source that grows the WebAssembly memory in large steps.
//!
//! Both dlmalloc (the default) and Talc's own `WasmGrowAndExtend` grow the memory by just what
//! the failing allocation needs. Each `memory.grow` is expensive for the browser, and Flex
//! applications such as Oxalis grow the heap to hundreds of megabytes: a CPU profile showed
//! two thirds of the time in these growths while Oxalis loaded a module and filled its home screen.

use std::alloc::Layout;
use std::arch::wasm32::{memory_grow, memory_size};
use std::ptr::NonNull;
use talc::base::binning::Binning;
use talc::base::{CHUNK_UNIT, Talc};
use talc::cell::TalcSyncCell;
use talc::wasm::WasmBinning;

/// Size of a WebAssembly memory page.
const PAGE_SIZE: usize = 64 * 1024;

/// Smallest growth step (16 MiB).
const MIN_STEP_PAGES: usize = 256;

/// Largest growth step (128 MiB), beyond what the allocation itself needs.
const MAX_STEP_PAGES: usize = 2048;

#[global_allocator]
static TALC: TalcSyncCell<WasmGrowInSteps, WasmBinning> =
    TalcSyncCell::new_wasm(WasmGrowInSteps { end: None });

/// Talc memory source that grows the memory by half its current size (between 16 and 128 MiB),
/// or by what the allocation needs if that is more, and extends the heap when it can.
#[derive(Debug)]
struct WasmGrowInSteps {
    end: Option<NonNull<u8>>,
}

// SAFETY: does not use the global allocator, and only manages the memory it grows itself.
unsafe impl talc::source::Source for WasmGrowInSteps {
    fn acquire<B: Binning>(talc: &mut Talc<Self, B>, layout: Layout) -> Result<(), ()> {
        // Chunk overhead and alignment padding, with room to spare
        let needed = (layout.size() + layout.align() + 4 * CHUNK_UNIT).div_ceil(PAGE_SIZE) + 1;
        let step = (memory_size::<0>() / 2).clamp(MIN_STEP_PAGES, MAX_STEP_PAGES);
        let mut delta = needed.max(step);
        let mut previous_pages = memory_grow::<0>(delta);
        if previous_pages == usize::MAX && delta > needed {
            // Near the memory limit: grow by what is needed only
            delta = needed;
            previous_pages = memory_grow::<0>(delta);
        }
        if previous_pages == usize::MAX {
            return Err(());
        }

        let new_base = (previous_pages * PAGE_SIZE) as *mut u8;
        let new_bytes = delta * PAGE_SIZE;
        let new_end = (new_base as usize).saturating_add(new_bytes) as *mut u8;
        if let Some(old_end) = talc.source.end.take()
            && old_end.as_ptr() == new_base
        {
            // SAFETY: the memory between the old and the new end was just grown for this heap.
            talc.source.end = Some(unsafe { talc.extend(old_end, new_end) });
            return Ok(());
        }
        // SAFETY: the memory was just grown and is used by nothing else.
        talc.source.end = unsafe { talc.claim(new_base, new_bytes) };
        Ok(())
    }
}
