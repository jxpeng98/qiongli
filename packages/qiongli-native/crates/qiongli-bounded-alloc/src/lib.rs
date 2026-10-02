//! Track Rust heap allocations; enable a ceiling only at isolated worker startup.
//! This does not limit native mappings, stack space or the process resident set.

use std::alloc::{GlobalAlloc, Layout, System};
use std::ptr;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct BoundedAllocator {
    allocated: AtomicUsize,
    limit: AtomicUsize,
}

impl BoundedAllocator {
    pub const fn new() -> Self {
        Self {
            allocated: AtomicUsize::new(0),
            limit: AtomicUsize::new(usize::MAX),
        }
    }

    /// Call once, before starting other threads or parsing untrusted data.
    /// Failure leaves the limit in place; the worker must exit immediately.
    pub fn limit_worker_heap(&self, bytes: usize) -> bool {
        self.limit
            .compare_exchange(usize::MAX, bytes, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
            && self.allocated.load(Ordering::SeqCst) <= bytes
    }

    fn reserve(&self, bytes: usize) -> bool {
        let mut used = self.allocated.load(Ordering::Relaxed);
        loop {
            let Some(next) = used.checked_add(bytes) else {
                return false;
            };
            if next > self.limit.load(Ordering::SeqCst) {
                return false;
            }
            match self.allocated.compare_exchange_weak(
                used,
                next,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => return true,
                Err(actual) => used = actual,
            }
        }
    }
}

impl Default for BoundedAllocator {
    fn default() -> Self {
        Self::new()
    }
}

// SAFETY: System owns every returned allocation. Reservations precede allocation
// and are released exactly once on failure/deallocation. Realloc reserves the
// entire new size while the old allocation is live, bounding peak requests even
// when System must copy. No accounting path allocates, panics or unwinds.
unsafe impl GlobalAlloc for BoundedAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if !self.reserve(layout.size()) {
            return ptr::null_mut();
        }
        // SAFETY: Forward the caller's valid, nonzero layout unchanged.
        let pointer = unsafe { System.alloc(layout) };
        if pointer.is_null() {
            self.allocated.fetch_sub(layout.size(), Ordering::SeqCst);
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: GlobalAlloc requires the original pointer/layout pair; all
        // successful allocations above came from System with the same layout.
        unsafe { System.dealloc(pointer, layout) };
        self.allocated.fetch_sub(layout.size(), Ordering::SeqCst);
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        if !self.reserve(layout.size()) {
            return ptr::null_mut();
        }
        // SAFETY: Preserve System's zeroed-allocation behavior and the caller's
        // valid layout, rather than eagerly zeroing every page in parent processes.
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if pointer.is_null() {
            self.allocated.fetch_sub(layout.size(), Ordering::SeqCst);
        }
        pointer
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if !self.reserve(new_size) {
            return ptr::null_mut();
        }
        // SAFETY: Forward the valid pointer/layout and nonzero new size. System
        // may grow in place; forcing allocate/copy/free penalizes unlimited parents.
        let updated = unsafe { System.realloc(pointer, layout, new_size) };
        self.allocated.fetch_sub(
            if updated.is_null() {
                new_size
            } else {
                layout.size()
            },
            Ordering::SeqCst,
        );
        updated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_over_budget_without_losing_the_existing_allocation() {
        let allocator = BoundedAllocator::new();
        assert!(allocator.limit_worker_heap(64));
        assert!(!allocator.limit_worker_heap(128));
        let layout = Layout::from_size_align(32, 8).unwrap();
        // SAFETY: Every successful pointer is deallocated with its original
        // layout; failed realloc preserves the old allocation by contract.
        unsafe {
            let pointer = allocator.alloc_zeroed(layout);
            assert!(!pointer.is_null());
            assert_eq!(pointer.read(), 0);
            assert!(allocator.realloc(pointer, layout, 48).is_null());
            pointer.write(7);
            assert_eq!(pointer.read(), 7);
            let small = allocator.realloc(pointer, layout, 16);
            assert!(!small.is_null());
            assert_eq!(small.read(), 7);
            let grown = allocator.realloc(small, Layout::from_size_align(16, 8).unwrap(), 48);
            assert!(!grown.is_null());
            assert_eq!(grown.read(), 7);
            allocator.dealloc(grown, Layout::from_size_align(48, 8).unwrap());
            let full = Layout::from_size_align(64, 8).unwrap();
            let pointer = allocator.alloc(full);
            assert!(!pointer.is_null());
            assert!(allocator.alloc(layout).is_null());
            allocator.dealloc(pointer, full);
        }
        assert_eq!(allocator.allocated.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn concurrent_reservations_never_wrap_or_exceed_the_limit() {
        let allocator = BoundedAllocator::new();
        assert!(allocator.limit_worker_heap(64));
        std::thread::scope(|scope| {
            for _ in 0..8 {
                scope.spawn(|| {
                    for _ in 0..1_000 {
                        if allocator.reserve(16) {
                            assert!(allocator.allocated.load(Ordering::SeqCst) <= 64);
                            allocator.allocated.fetch_sub(16, Ordering::SeqCst);
                        }
                        assert!(!allocator.reserve(usize::MAX));
                    }
                });
            }
        });
        assert_eq!(allocator.allocated.load(Ordering::SeqCst), 0);
    }
}
