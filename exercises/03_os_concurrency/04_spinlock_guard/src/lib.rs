//! # RAII 自旋锁守卫
//!
//! 在本练习中，你将为自旋锁实现 RAII 守卫，使锁在离开作用域时自动释放。
//! 这是 Rust 所有权系统在系统编程中的经典应用。
//!
//! ## 关键点
//! - RAII（资源获取即初始化）模式
//! - `Deref` / `DerefMut` trait 用于透明访问
//! - `Drop` trait 用于自动释放
//! - 为什么手动 lock/unlock 是不安全的（忘记 unlock、panic 时未释放）

use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicBool, Ordering};

pub struct SpinLock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Sync for SpinLock<T> {}
unsafe impl<T: Send> Send for SpinLock<T> {}

/// 自旋锁守卫：持有锁的 RAII 句柄。
/// 当 SpinGuard 被丢弃时自动释放锁。
pub struct SpinGuard<'a, T> {
    lock: &'a SpinLock<T>,
}

impl<T> SpinLock<T> {
    pub fn new(data: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    /// 获取锁，返回 SpinGuard。
    ///
    /// TODO: 自旋等待获取锁（compare_exchange），成功后返回 SpinGuard。
    pub fn lock(&self) -> SpinGuard<'_, T> {
        // TODO: 自旋等待获取锁
        // TODO: 返回 SpinGuard { lock: self }
        todo!()
    }
}

// TODO: 为 SpinGuard 实现 Deref trait
// 返回 &T，通过 self.lock.data.get() 获取
impl<T> Deref for SpinGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        todo!()
    }
}

// TODO: 为 SpinGuard 实现 DerefMut trait
// 返回 &mut T
impl<T> DerefMut for SpinGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        todo!()
    }
}

// TODO: 为 SpinGuard 实现 Drop trait
// 将 lock.locked 设置为 false（使用 Release 顺序）
impl<T> Drop for SpinGuard<'_, T> {
    fn drop(&mut self) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_guard_auto_release() {
        let lock = SpinLock::new(0u32);
        {
            let mut guard = lock.lock();
            *guard = 42;
            // guard 在此处 drop，自动释放锁
        }
        // 应该能够再次获取锁
        let guard = lock.lock();
        assert_eq!(*guard, 42);
    }

    #[test]
    fn test_guard_deref() {
        let lock = SpinLock::new(String::from("hello"));
        let guard = lock.lock();
        assert_eq!(guard.len(), 5);
        assert_eq!(&*guard, "hello");
    }

    #[test]
    fn test_guard_deref_mut() {
        let lock = SpinLock::new(Vec::<i32>::new());
        {
            let mut guard = lock.lock();
            guard.push(1);
            guard.push(2);
            guard.push(3);
        }
        let guard = lock.lock();
        assert_eq!(&*guard, &[1, 2, 3]);
    }

    #[test]
    fn test_concurrent_with_guard() {
        let lock = Arc::new(SpinLock::new(0u64));
        let mut handles = vec![];

        for _ in 0..10 {
            let l = Arc::clone(&lock);
            handles.push(thread::spawn(move || {
                for _ in 0..1000 {
                    let mut guard = l.lock();
                    *guard += 1;
                    // guard 自动释放
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(*lock.lock(), 10000);
    }

    #[test]
    fn test_panic_safety() {
        let lock = Arc::new(SpinLock::new(0u32));
        let l = Arc::clone(&lock);

        let result = thread::spawn(move || {
            let mut guard = l.lock();
            *guard = 42;
            panic!("intentional panic");
        })
        .join();

        assert!(result.is_err());
        // 即使线程 panic，guard 的 Drop 也应该释放锁
        // 注意：此测试可能因 panic unwind 行为而有不同结果
    }
}
