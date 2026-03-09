//! # 内存顺序与同步
//!
//! 在本练习中，你将使用正确的内存顺序实现线程同步原语。
//!
//! ## 核心概念
//! - `Ordering::Relaxed`：无同步保证
//! - `Ordering::Acquire`：读操作，防止后续的读/写被重排到此操作之前
//! - `Ordering::Release`：写操作，防止之前的读/写被重排到此操作之后
//! - `Ordering::AcqRel`：同时具有 Acquire 和 Release 语义
//! - `Ordering::SeqCst`：顺序一致（全局顺序）
//!
//! ## Release-Acquire 配对
//! 当线程 A 使用 Release 写入，线程 B 使用 Acquire 读取同一位置时，
//! 线程 B 将看到线程 A 在 Release 之前执行的所有写入。

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

/// 使用 Release-Acquire 语义在两个线程之间安全地传递数据。
///
/// `produce` 先写入数据，然后用 Release 设置标志；
/// `consume` 用 Acquire 读取标志，确保能看到数据。
pub struct FlagChannel {
    data: AtomicU32,
    ready: AtomicBool,
}

impl FlagChannel {
    pub const fn new() -> Self {
        Self {
            data: AtomicU32::new(0),
            ready: AtomicBool::new(false),
        }
    }

    /// 生产者：先存储数据，然后设置 ready 标志。
    ///
    /// TODO：选择正确的 Ordering
    /// - 写入数据应该使用什么 Ordering？
    /// - 写入 ready 应该使用什么 Ordering？（确保数据写入对消费者可见）
    pub fn produce(&self, value: u32) {
        // TODO: 存储数据（选择合适的 Ordering）
        // TODO: 设置 ready = true（选择合适的 Ordering，确保数据写入在此之前完成）
        todo!()
    }

    /// 消费者：自旋等待 ready 标志，然后读取数据。
    ///
    /// TODO：选择正确的 Ordering
    /// - 读取 ready 应该使用什么 Ordering？（确保能看到 produce 中的数据写入）
    /// - 读取数据应该使用什么 Ordering？
    pub fn consume(&self) -> u32 {
        // TODO: 自旋等待 ready 变为 true（选择合适的 Ordering）
        // TODO: 读取数据（选择合适的 Ordering）
        todo!()
    }

    /// 重置通道状态
    pub fn reset(&self) {
        self.ready.store(false, Ordering::Relaxed);
        self.data.store(0, Ordering::Relaxed);
    }
}

/// 使用 SeqCst 的简单一次性初始化器。
/// 保证 `init` 只执行一次，且所有线程都能看到初始化后的值。
pub struct OnceCell {
    initialized: AtomicBool,
    value: AtomicU32,
}

impl OnceCell {
    pub const fn new() -> Self {
        Self {
            initialized: AtomicBool::new(false),
            value: AtomicU32::new(0),
        }
    }

    /// 尝试初始化。如果尚未初始化，存储值并返回 true。
    /// 如果已经初始化，返回 false。
    ///
    /// 提示：使用 `compare_exchange` 确保只有一个线程成功。
    pub fn init(&self, val: u32) -> bool {
        // TODO: 使用 compare_exchange 确保只初始化一次
        // 成功时存储值
        todo!()
    }

    /// 获取值。如果已初始化返回 Some，否则返回 None。
    pub fn get(&self) -> Option<u32> {
        // TODO: 检查 initialized 标志，然后读取值
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_flag_channel() {
        let ch = Arc::new(FlagChannel::new());
        let ch2 = Arc::clone(&ch);

        let producer = thread::spawn(move || {
            ch2.produce(42);
        });

        let consumer = thread::spawn(move || ch.consume());

        producer.join().unwrap();
        let val = consumer.join().unwrap();
        assert_eq!(val, 42);
    }

    #[test]
    fn test_flag_channel_large_value() {
        let ch = Arc::new(FlagChannel::new());
        let ch2 = Arc::clone(&ch);

        let producer = thread::spawn(move || {
            ch2.produce(0xDEAD_BEEF);
        });

        let val = ch.consume();
        producer.join().unwrap();
        assert_eq!(val, 0xDEAD_BEEF);
    }

    #[test]
    fn test_once_cell_init_once() {
        let cell = OnceCell::new();
        assert!(cell.init(42));
        assert!(!cell.init(100));
        assert_eq!(cell.get(), Some(42));
    }

    #[test]
    fn test_once_cell_not_initialized() {
        let cell = OnceCell::new();
        assert_eq!(cell.get(), None);
    }

    #[test]
    fn test_once_cell_concurrent() {
        let cell = Arc::new(OnceCell::new());
        let mut handles = vec![];

        for i in 0..10 {
            let c = Arc::clone(&cell);
            handles.push(thread::spawn(move || c.init(i)));
        }

        let results: Vec<bool> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        // 恰好一个线程成功初始化
        assert_eq!(results.iter().filter(|&&r| r).count(), 1);
        assert!(cell.get().is_some());
    }
}
