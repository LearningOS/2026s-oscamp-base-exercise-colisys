//! # 自旋锁
//!
//! 在本练习中，你将实现一个基本的自旋锁。
//! 自旋锁是操作系统内核中最基础的同步原语之一。
//!
//! ## 核心概念
//! - 自旋锁使用忙等待来获取锁
//! - `AtomicBool` 的 `compare_exchange` 实现锁获取
//! - `core::hint::spin_loop` 减少 CPU 功耗
//! - `UnsafeCell` 提供内部可变性

use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, Ordering};

/// 基本自旋锁
pub struct SpinLock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Sync for SpinLock<T> {}
unsafe impl<T: Send> Send for SpinLock<T> {}

impl<T> SpinLock<T> {
    pub fn new(data: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    /// 获取锁，返回内部数据的可变引用。
    ///
    /// TODO: 使用 compare_exchange 自旋直到获取锁
    /// 1. 在循环中，尝试将 locked 从 false 改为 true
    /// 2. 成功使用 Acquire 顺序，失败使用 Relaxed
    /// 3. 失败时调用 `core::hint::spin_loop()` 提示 CPU
    /// 4. 成功时返回 `&mut *self.data.get()`
    ///
    /// # 安全性
    /// 调用者必须确保在使用完数据后调用 `unlock`。
    pub fn lock(&self) -> &mut T {
        // TODO
        todo!()
    }

    /// 释放锁。
    ///
    /// TODO: 将 locked 设置为 false（使用 Release 顺序）
    pub fn unlock(&self) {
        // TODO
        todo!()
    }

    /// 尝试获取锁而不自旋。
    /// 成功返回 Some(&mut T)，锁忙时返回 None。
    pub fn try_lock(&self) -> Option<&mut T> {
        // TODO: 单次 compare_exchange 尝试
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_basic_lock_unlock() {
        let lock = SpinLock::new(0u32);
        {
            let data = lock.lock();
            *data = 42;
            lock.unlock();
        }
        let data = lock.lock();
        assert_eq!(*data, 42);
        lock.unlock();
    }

    #[test]
    fn test_try_lock() {
        let lock = SpinLock::new(0u32);
        assert!(lock.try_lock().is_some());
        lock.unlock();
    }

    #[test]
    fn test_concurrent_counter() {
        let lock = Arc::new(SpinLock::new(0u64));
        let mut handles = vec![];

        for _ in 0..10 {
            let l = Arc::clone(&lock);
            handles.push(thread::spawn(move || {
                for _ in 0..1000 {
                    let data = l.lock();
                    *data += 1;
                    l.unlock();
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        let data = lock.lock();
        assert_eq!(*data, 10000);
        lock.unlock();
    }

    #[test]
    fn test_lock_protects_data() {
        let lock = Arc::new(SpinLock::new(Vec::new()));
        let mut handles = vec![];

        for i in 0..5 {
            let l = Arc::clone(&lock);
            handles.push(thread::spawn(move || {
                let data = l.lock();
                data.push(i);
                l.unlock();
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        let data = lock.lock();
        let mut sorted = data.clone();
        sorted.sort();
        assert_eq!(sorted, vec![0, 1, 2, 3, 4]);
        lock.unlock();
    }
}
