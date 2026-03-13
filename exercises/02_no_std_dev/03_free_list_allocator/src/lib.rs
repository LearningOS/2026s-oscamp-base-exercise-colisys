//! # 空闲链表分配器
//!
//! 在 bump 分配器的基础上，实现一个支持内存回收的空闲链表分配器。
//!
//! ## 工作原理
//!
//! 空闲链表分配器使用链表来跟踪所有已释放的内存块。
//! 分配时，它首先在链表中搜索合适的块（首次适应策略）；
//! 如果没找到，则从未分配区域分配。
//! 释放时，将块插入链表头部。
//!
//! ```text
//! free_list -> [块 A: 64B] -> [块 B: 128B] -> [块 C: 32B] -> null
//! ```
//!
//! 每个空闲块在其头部存储一个 `FreeBlock` 结构（包含块大小和下一个指针）。
//!
//! ## 任务
//!
//! 实现 `FreeListAllocator` 的 `alloc` 和 `dealloc` 方法：
//!
//! ### alloc
//! 1. 遍历 free_list，找到第一个满足 `size >= layout.size()` 且对齐正确的块（首次适应）
//! 2. 如果找到，从链表中移除并返回
//! 3. 如果没找到，从 `bump` 区域分配（与 bump 分配器相同）
//!
//! ### dealloc
//! 1. 在释放的块处写入 `FreeBlock` 头信息
//! 2. 将其插入 free_list 头部
//!
//! ## 核心概念
//!
//! - 侵入式链表
//! - `*mut T` 读/写：`ptr.write(val)` / `ptr.read()`
//! - 内存对齐检查

#![cfg_attr(not(test), no_std)]

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

/// 空闲块头部，存储在每个空闲内存块的开头
struct FreeBlock {
    size: usize,
    next: *mut FreeBlock,
}

pub struct FreeListAllocator {
    heap_start: usize,
    heap_end: usize,
    /// Bump 指针：未分配区域从这里开始
    bump_next: core::sync::atomic::AtomicUsize,
    /// 空闲链表头（测试中用 Mutex 保护，其他情况用 UnsafeCell）
    #[cfg(test)]
    free_list: std::sync::Mutex<*mut FreeBlock>,
    #[cfg(not(test))]
    free_list: core::cell::UnsafeCell<*mut FreeBlock>,
}

#[cfg(test)]
unsafe impl Send for FreeListAllocator {}
#[cfg(test)]
unsafe impl Sync for FreeListAllocator {}
#[cfg(not(test))]
unsafe impl Send for FreeListAllocator {}
#[cfg(not(test))]
unsafe impl Sync for FreeListAllocator {}

impl FreeListAllocator {
    /// # 安全性
    /// `heap_start..heap_end` 必须是一个有效可读可写的内存区域。
    pub unsafe fn new(heap_start: usize, heap_end: usize) -> Self {
        Self {
            heap_start,
            heap_end,
            bump_next: core::sync::atomic::AtomicUsize::new(heap_start),
            #[cfg(test)]
            free_list: std::sync::Mutex::new(null_mut()),
            #[cfg(not(test))]
            free_list: core::cell::UnsafeCell::new(null_mut()),
        }
    }

    #[cfg(test)]
    fn free_list_head(&self) -> *mut FreeBlock {
        *self.free_list.lock().unwrap()
    }

    #[cfg(test)]
    fn set_free_list_head(&self, head: *mut FreeBlock) {
        *self.free_list.lock().unwrap() = head;
    }

    #[cfg(not(test))]
    fn free_list_head(&self) -> *mut FreeBlock {
        unsafe { *self.free_list.get() }
    }

    #[cfg(not(test))]
    fn set_free_list_head(&self, head: *mut FreeBlock) {
        unsafe { *self.free_list.get() = head }
    }

    unsafe fn alloc_from_bump(&self, layout: Layout) -> *mut u8 {
        // 检查当前偏移
        let current_offset = self.bump_next.load(core::sync::atomic::Ordering::SeqCst);
        // 获得对齐
        let align = layout.align();
        // 计算真实的偏移
        let aligned_offset = (current_offset + align - 1) & !(align - 1);

        // 获得申请单元的长度
        let size = layout.size();

        let next = aligned_offset + size;
        // 申请该长度会导致堆溢出，返回 null_mut
        if next > self.heap_end {
            return null_mut();
        };

        loop {
            if let Ok(_) = self.bump_next.compare_exchange(
                current_offset,
                next,
                core::sync::atomic::Ordering::SeqCst,
                core::sync::atomic::Ordering::Relaxed,
            ) {
                break;
            }
        }

        aligned_offset as *mut u8
    }
}

unsafe impl GlobalAlloc for FreeListAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // 确保块至少能容纳 FreeBlock 头部（用于后续的 dealloc）
        let size = layout.size().max(core::mem::size_of::<FreeBlock>());
        let align = layout.align().max(core::mem::align_of::<FreeBlock>());

        // TODO: 步骤 1 —— 遍历 free_list，找到合适的块（首次适应）
        //
        // 提示：
        // - 使用 prev_ptr 和 curr 遍历链表
        // - 检查 curr 地址是否满足对齐要求，且 (*curr).size >= size
        // - 如果找到，从链表中移除（更新 prev 的 next 或 free_list 头）
        // - 将 curr 作为 *mut u8 返回

        // TODO: 步骤 2 —— free_list 中没有合适的块，从 bump 区域分配
        //
        // 与 02_bump_allocator 的 alloc 逻辑相同
        // todo!();

        // 指向 free_list 的指针
        let mut free_list_ptr = self.free_list_head();
        // free_list_ptr 的前驱指针
        let mut prev_block = null_mut() as *mut FreeBlock;
        // 基于堆起始地址的累积偏移量
        let mut offset = self.heap_start;
        // 通过指针遍历链表
        loop {
            if free_list_ptr.is_null() {
                // free_list_ptr 指向了空，直接从 bump 中申请空间
                return self.alloc_from_bump(layout);
            } else {
                let free_block = free_list_ptr.read();
                let aligned_offset = (offset + align - 1) & !(align - 1);
                #[cfg(test)]
                println!(
                    "aligned_offset:{}, offset:{}, offset%align={}",
                    aligned_offset,
                    offset,
                    offset.rem_euclid(align)
                );
                if aligned_offset == offset && free_block.size >= size {
                    // 从链表中摘除节点
                    if prev_block.is_null() {
                        self.set_free_list_head(free_block.next);
                    } else {
                        prev_block.read().next = free_block.next;
                    }
                    // 可以安全的将偏移量作为地址返回
                    return offset as *mut u8;
                }
                // 记录偏移量
                offset += free_block.size;
                // 向后移动
                prev_block = free_list_ptr;
                free_list_ptr = free_block.next;
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size = layout.size().max(core::mem::size_of::<FreeBlock>());

        // TODO: 将释放的块插入 free_list 头部
        //
        // 步骤：
        // 1. 将 ptr 转换为 *mut FreeBlock
        // 2. 写入 FreeBlock { size, next: 当前链表头 }
        // 3. 将 free_list 头更新为 ptr
        // todo!();

        // 这里的思想是，释放的区块里面的内容已经作废，
        // 可以安全的在那个地方写入 FreeBlock，并且
        // 将指针加入 FreeList 中
        // 遍历 FreeList 实际上是在跳跃访问 heap 空间
        let ptr = ptr as *mut FreeBlock;
        ptr.write(FreeBlock {
            size,
            next: self.free_list_head(),
        });
        self.set_free_list_head(ptr);
    }
}

// ============================================================
// 测试
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;

    const HEAP_SIZE: usize = 4096;

    fn make_allocator() -> (FreeListAllocator, Vec<u8>) {
        let mut heap = vec![0u8; HEAP_SIZE];
        let start = heap.as_mut_ptr() as usize;
        let alloc = unsafe { FreeListAllocator::new(start, start + HEAP_SIZE) };
        (alloc, heap)
    }

    #[test]
    fn test_alloc_basic() {
        let (alloc, _heap) = make_allocator();
        let layout = Layout::from_size_align(32, 8).unwrap();
        let ptr = unsafe { alloc.alloc(layout) };
        assert!(!ptr.is_null());
    }

    #[test]
    fn test_alloc_alignment() {
        let (alloc, _heap) = make_allocator();
        for align in [1, 2, 4, 8, 16] {
            let layout = Layout::from_size_align(8, align).unwrap();
            let ptr = unsafe { alloc.alloc(layout) };
            assert!(!ptr.is_null());
            assert_eq!(ptr as usize % align, 0, "align={align}");
        }
    }

    #[test]
    fn test_dealloc_and_reuse() {
        let (alloc, _heap) = make_allocator();
        let layout = Layout::from_size_align(64, 8).unwrap();

        let p1 = unsafe { alloc.alloc(layout) };
        assert!(!p1.is_null());

        // 释放后，下次分配应该复用同一个块
        unsafe { alloc.dealloc(p1, layout) };
        let p2 = unsafe { alloc.alloc(layout) };
        assert!(!p2.is_null());
        assert_eq!(p1, p2, "should reuse the freed block");
    }

    #[test]
    fn test_multiple_alloc_dealloc() {
        let (alloc, _heap) = make_allocator();
        let layout = Layout::from_size_align(128, 8).unwrap();

        let p1 = unsafe { alloc.alloc(layout) };
        let p2 = unsafe { alloc.alloc(layout) };
        let p3 = unsafe { alloc.alloc(layout) };
        assert!(!p1.is_null() && !p2.is_null() && !p3.is_null());

        unsafe { alloc.dealloc(p2, layout) };
        unsafe { alloc.dealloc(p1, layout) };

        let q1 = unsafe { alloc.alloc(layout) };
        let q2 = unsafe { alloc.alloc(layout) };
        assert!(!q1.is_null() && !q2.is_null());
    }

    #[test]
    fn test_oom() {
        let (alloc, _heap) = make_allocator();
        let layout = Layout::from_size_align(HEAP_SIZE + 1, 1).unwrap();
        let ptr = unsafe { alloc.alloc(layout) };
        assert!(ptr.is_null(), "should return null when exceeding heap");
    }
}
