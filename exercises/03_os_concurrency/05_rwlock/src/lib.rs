//! # 读写锁（写者优先）
//!
//! 在本练习中，你将使用原子操作从零实现一个**写者优先**的读写锁。
//! 多个读者可以同时持有锁；写者独占持有。
//!
//! **注意：** Rust 标准库已提供 [`std::sync::RwLock`]。本练习实现一个简化版本，
//! 用于学习协议和策略，不使用标准库的实现。
//!
//! ## 读写锁的常见策略
//! 不同的实现在读者和写者同时等待时可以给予不同的**优先级**：
//!
//! - **读者优先**：当写者在等待时，新读者仍可进入，因此如果读者持续到来，写者可能饥饿。
//! - **写者优先（本实现）**：一旦有写者在等待，就不再接受新读者，直到该写者运行。
//! - **读写公平**：按公平顺序（如 FIFO 或轮转）服务请求，读者和写者都不会系统性饥饿。
//!
//! ## 核心概念
//! - **读者**：共享访问；多个线程可同时持有读锁。
//! - **写者**：独占访问；写者持有锁时，不能有其他写者或读者。
//! - **写者优先（本实现）**：当至少有一个写者在等待时，新读者会被阻塞直到写者运行。
//!
//! ## 状态（单一原子变量）
//! 我们使用一个 `AtomicU32`：低位 = 读者计数，两个标志位 = 写者持有 / 写者等待。
//! 所有逻辑用 compare_exchange 和 load/store 实现；不使用 `std::sync::RwLock`。

use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicU32, Ordering};

/// 最大并发读者数（适应状态位）
/// xx111111 11111111 11111111 11111111
/// 0 ~ 30位
const READER_MASK: u32 = (1 << 30) - 1;
/// 写者持有锁时设置的位。
/// x1xxxxxx xxxxxxxx xxxxxxxx xxxxxxxx
/// 第 31 位
const WRITER_HOLDING: u32 = 1 << 30;
/// 至少有一个写者在等待时设置的位（写者优先：阻塞新读者）。
/// 1xxxxxxx xxxxxxxx xxxxxxxx xxxxxxxx
/// 第 32 位
const WRITER_WAITING: u32 = 1 << 31;

/// 写者优先读写锁。从零实现；不使用 `std::sync::RwLock`。
pub struct RwLock<T> {
    state: AtomicU32,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for RwLock<T> {}
unsafe impl<T: Send + Sync> Sync for RwLock<T> {}

impl<T> RwLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            state: AtomicU32::new(0),
            data: UnsafeCell::new(data),
        }
    }

    /// 获取读锁。阻塞（自旋）直到没有写者持有且没有写者在等待（写者优先）。
    ///
    /// TODO: 实现读锁获取
    /// 1. 在循环中，加载 state（Acquire）。
    /// 2. 如果设置了 WRITER_HOLDING 或 WRITER_WAITING，spin_loop 并继续（写者优先：写者等待时阻止新读者）。
    /// 3. 如果读者计数（state & READER_MASK）已经是 READER_MASK，自旋并继续。
    /// 4. 尝试 compare_exchange(s, s + 1, AcqRel, Acquire)；成功则返回 RwLockReadGuard { lock: self }。
    pub fn read(&self) -> RwLockReadGuard<'_, T> {
        // TODO
        // todo!();
        loop {
            core::hint::spin_loop();
            let state = self.state.load(Ordering::Acquire);
            if state & (WRITER_HOLDING | WRITER_WAITING) != 0 || state & READER_MASK == READER_MASK
            {
                continue;
            }
            if let Ok(_) =
                self.state
                    .compare_exchange(state, state + 1, Ordering::AcqRel, Ordering::Acquire)
            {
                return RwLockReadGuard { lock: self };
            }
        }
    }

    /// 获取写锁。阻塞直到没有读者且没有其他写者。
    ///
    /// TODO: 实现写锁获取（写者优先）
    /// 1. 首先设置 WRITER_WAITING：fetch_or(WRITER_WAITING, Release) 让新读者阻塞。
    /// 2. 在循环中：加载 state；如果有读者（READER_MASK）或 WRITER_HOLDING，spin_loop 并继续。
    /// 3. 尝试 compare_exchange(WRITER_WAITING, WRITER_HOLDING, ...) 获取锁；或者如果写者刚释放，compare_exchange(0, WRITER_HOLDING, ...)。
    /// 4. 成功则返回 RwLockWriteGuard { lock: self }。
    pub fn write(&self) -> RwLockWriteGuard<'_, T> {
        // TODO
        self.state.fetch_or(WRITER_WAITING, Ordering::Release);
        loop {
            core::hint::spin_loop();
            let state = self.state.load(Ordering::Acquire);
            if state & (READER_MASK | WRITER_HOLDING) != 0 {
                continue;
            }

            if let Ok(_) =
                self.state
                    .compare_exchange(0, WRITER_HOLDING, Ordering::AcqRel, Ordering::Acquire)
            {
                return RwLockWriteGuard { lock: self };
            }

            if let Ok(_) = self.state.compare_exchange(
                WRITER_WAITING,
                WRITER_HOLDING,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                return RwLockWriteGuard { lock: self };
            }
        }
    }
}

/// 读锁守卫；drop 时释放读锁。
pub struct RwLockReadGuard<'a, T> {
    lock: &'a RwLock<T>,
}

// TODO: 为 RwLockReadGuard 实现 Deref
// 返回数据的共享引用：unsafe { &*self.lock.data.get() }
impl<T> Deref for RwLockReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

// TODO: 为 RwLockReadGuard 实现 Drop
// 递减读者计数：self.lock.state.fetch_sub(1, Ordering::Release)
impl<T> Drop for RwLockReadGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.state.fetch_sub(1, Ordering::Release);
    }
}

/// 写锁守卫；drop 时释放写锁。
pub struct RwLockWriteGuard<'a, T> {
    lock: &'a RwLock<T>,
}

// TODO: 为 RwLockWriteGuard 实现 Deref
// 返回共享引用：unsafe { &*self.lock.data.get() }
impl<T> Deref for RwLockWriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

// TODO: 为 RwLockWriteGuard 实现 DerefMut
// 返回可变引用：unsafe { &mut *self.lock.data.get() }
impl<T> DerefMut for RwLockWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

// TODO: 为 RwLockWriteGuard 实现 Drop
// 清除写者位使锁空闲：self.lock.state.fetch_and(!(WRITER_HOLDING | WRITER_WAITING), Ordering::Release)
impl<T> Drop for RwLockWriteGuard<'_, T> {
    fn drop(&mut self) {
        self.lock
            .state
            .fetch_and(!(WRITER_WAITING | WRITER_HOLDING), Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_multiple_readers() {
        let lock = Arc::new(RwLock::new(0u32));
        let mut handles = vec![];
        for _ in 0..10 {
            let l = Arc::clone(&lock);
            handles.push(thread::spawn(move || {
                let g = l.read();
                assert_eq!(*g, 0);
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
    }

    #[test]
    fn test_writer_excludes_readers() {
        let lock = Arc::new(RwLock::new(0u32));
        let lock_w = Arc::clone(&lock);
        let writer = thread::spawn(move || {
            let mut g = lock_w.write();
            *g = 42;
        });
        writer.join().unwrap();
        let g = lock.read();
        assert_eq!(*g, 42);
    }

    #[test]
    fn test_concurrent_reads_after_write() {
        let lock = Arc::new(RwLock::new(Vec::<i32>::new()));
        {
            let mut g = lock.write();
            g.push(1);
            g.push(2);
        }
        let mut handles = vec![];
        for _ in 0..5 {
            let l = Arc::clone(&lock);
            handles.push(thread::spawn(move || {
                let g = l.read();
                assert_eq!(g.len(), 2);
                assert_eq!(&*g, &[1, 2]);
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
    }

    #[test]
    fn test_concurrent_writes_serialized() {
        let lock = Arc::new(RwLock::new(0u64));
        let mut handles = vec![];
        for _ in 0..10 {
            let l = Arc::clone(&lock);
            handles.push(thread::spawn(move || {
                for _ in 0..100 {
                    let mut g = l.write();
                    *g += 1;
                }
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        assert_eq!(*lock.read(), 1000);
    }
}
