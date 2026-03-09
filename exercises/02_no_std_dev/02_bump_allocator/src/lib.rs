//! # Bump 分配器 (no_std)
//!
//! 实现最简单的堆内存分配器：Bump 分配器（指针碰撞分配器）。
//!
//! ## 工作原理
//!
//! Bump 分配器维护一个指针 `next`，指向"下一个可用地址"。
//! 每次分配时，它将 `next` 对齐到请求的对齐边界，然后向前推进 `size` 字节。
//! 它不支持释放单个对象（`dealloc` 是空操作）。
//!
//! ```text
//! heap_start                              heap_end
//! |----[已分配]----[已分配]----| next |---[空闲]---|
//!                                        ^
//!                                    下次分配从这里开始
//! ```
//!
//! ## 任务
//!
//! 实现 `BumpAllocator` 的 `GlobalAlloc::alloc` 方法：
//! 1. 将当前 `next` 向上对齐到 `layout.align()`
//!    提示：align_up(addr, align) = (addr + align - 1) & !(align - 1)
//! 2. 检查对齐后的地址加上 `layout.size()` 是否超过 `heap_end`
//! 3. 如果超过，返回 `null_mut()`；否则使用 `compare_exchange` 原子更新 `next`
//!
//! ## 核心概念
//!
//! - `core::alloc::{GlobalAlloc, Layout}`
//! - 内存对齐计算
//! - `AtomicUsize` 和 `compare_exchange`（CAS 循环）

#![cfg_attr(not(test), no_std)]

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use core::sync::atomic::{AtomicUsize, Ordering};

pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: AtomicUsize,
}

impl BumpAllocator {
    /// 创建一个新的 Bump 分配器。
    ///
    /// # 安全性
    /// `heap_start..heap_end` 必须是一个有效、可读可写的内存区域，
    /// 且在此分配器的生命周期内不能被其他代码使用。
    pub const unsafe fn new(heap_start: usize, heap_end: usize) -> Self {
        Self {
            heap_start,
            heap_end,
            next: AtomicUsize::new(heap_start),
        }
    }

    /// 重置分配器（释放所有已分配的内存）。
    pub fn reset(&self) {
        self.next.store(self.heap_start, Ordering::SeqCst);
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // TODO: 实现 bump 分配
        //
        // 步骤：
        // 1. 加载当前 next（使用 Ordering::SeqCst）
        // 2. 将 next 向上对齐到 layout.align()
        //    提示：align_up(addr, align) = (addr + align - 1) & !(align - 1)
        // 3. 计算分配结束位置 = aligned + layout.size()
        // 4. 如果 end > heap_end，返回 null_mut()
        // 5. 使用 compare_exchange 原子地将 next 更新为 end
        //    （如果 CAS 失败，说明其他线程竞争——在循环中重试）
        // 6. 将对齐后的地址作为指针返回
        todo!()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump 分配器不回收单个对象——留空即可
    }
}

// ============================================================
// 测试
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;

    const HEAP_SIZE: usize = 4096;

    fn make_allocator() -> (BumpAllocator, Vec<u8>) {
        let mut heap = vec![0u8; HEAP_SIZE];
        let start = heap.as_mut_ptr() as usize;
        let alloc = unsafe { BumpAllocator::new(start, start + HEAP_SIZE) };
        (alloc, heap)
    }

    #[test]
    fn test_alloc_basic() {
        let (alloc, _heap) = make_allocator();
        let layout = Layout::from_size_align(16, 8).unwrap();
        let ptr = unsafe { alloc.alloc(layout) };
        assert!(!ptr.is_null(), "allocation should succeed");
    }

    #[test]
    fn test_alloc_alignment() {
        let (alloc, _heap) = make_allocator();
        for align in [1, 2, 4, 8, 16, 64] {
            let layout = Layout::from_size_align(1, align).unwrap();
            let ptr = unsafe { alloc.alloc(layout) };
            assert!(!ptr.is_null());
            assert_eq!(
                ptr as usize % align,
                0,
                "returned address must satisfy align={align}"
            );
        }
    }

    #[test]
    fn test_alloc_no_overlap() {
        let (alloc, _heap) = make_allocator();
        let layout = Layout::from_size_align(64, 8).unwrap();
        let p1 = unsafe { alloc.alloc(layout) } as usize;
        let p2 = unsafe { alloc.alloc(layout) } as usize;
        assert!(
            p1 + 64 <= p2 || p2 + 64 <= p1,
            "two allocations must not overlap"
        );
    }

    #[test]
    fn test_alloc_oom() {
        let (alloc, _heap) = make_allocator();
        let layout = Layout::from_size_align(HEAP_SIZE + 1, 1).unwrap();
        let ptr = unsafe { alloc.alloc(layout) };
        assert!(ptr.is_null(), "should return null when exceeding heap");
    }

    #[test]
    fn test_alloc_fill_heap() {
        let (alloc, _heap) = make_allocator();
        let layout = Layout::from_size_align(256, 1).unwrap();
        for i in 0..16 {
            let ptr = unsafe { alloc.alloc(layout) };
            assert!(!ptr.is_null(), "allocation #{i} should succeed");
        }
        let ptr = unsafe { alloc.alloc(layout) };
        assert!(ptr.is_null(), "should return null when heap is full");
    }

    #[test]
    fn test_reset() {
        let (alloc, _heap) = make_allocator();
        let layout = Layout::from_size_align(HEAP_SIZE, 1).unwrap();
        let p1 = unsafe { alloc.alloc(layout) };
        assert!(!p1.is_null());
        alloc.reset();
        let p2 = unsafe { alloc.alloc(layout) };
        assert!(!p2.is_null(), "should be able to allocate after reset");
        assert_eq!(
            p1, p2,
            "address after reset should match the first allocation"
        );
    }
}
