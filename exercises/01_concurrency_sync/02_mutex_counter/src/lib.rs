//! # Mutex 共享状态
//!
//! 在本练习中，你将使用 `Arc<Mutex<T>>` 在多个线程之间安全地共享和修改数据。
//!
//! ## 核心概念
//! - `Mutex<T>` 互斥锁保护共享数据
//! - `Arc<T>` 原子引用计数支持跨线程共享
//! - `lock()` 获取锁并访问数据

use std::sync::{Arc, Mutex};
use std::thread;

/// 使用 `n_threads` 个线程并发递增计数器。
/// 每个线程递增计数器 `count_per_thread` 次。
/// 返回最终计数器值。
///
/// 提示：使用 `Arc<Mutex<usize>>` 作为共享计数器。
pub fn concurrent_counter(n_threads: usize, count_per_thread: usize) -> usize {
    // TODO: 创建初始值为 0 的 Arc<Mutex<usize>>
    // TODO: 派生 n_threads 个线程
    // TODO: 在每个线程中 lock() 并递增 count_per_thread 次
    // TODO: 等待所有线程结束，返回最终值

    let counter = Arc::new(Mutex::new(0));
    thread::scope(|scope| {
        for _ in 0..n_threads {
            scope
                .spawn(|| {
                    let mut ptr = counter.lock().unwrap();
                    *ptr += count_per_thread;
                })
                .join()
                .unwrap();
        }
    });
    let d = counter.lock().unwrap();
    *d
}

/// 使用多个线程并发地向共享向量添加元素。
/// 每个线程将自己的 id（0..n_threads）推入向量。
/// 返回排序后的向量。
///
/// 提示：使用 `Arc<Mutex<Vec<usize>>>`。
pub fn concurrent_collect(n_threads: usize) -> Vec<usize> {
    // TODO: 创建 Arc<Mutex<Vec<usize>>>
    // TODO: 每个线程推入自己的 id
    // TODO: 等待所有线程结束后，排序结果并返回

    let counter = Arc::new(Mutex::new(vec![]));
    thread::scope(|scope| {
        for id in 0..n_threads {
            let counter = counter.clone();
            scope
                .spawn(move || {
                    let mut ptr = counter.lock().unwrap();
                    ptr.push(id);
                })
                .join()
                .unwrap();
        }
    });
    let d = counter.lock().unwrap();
    let mut arr = d.clone();
    arr.sort();
    arr
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter_single_thread() {
        assert_eq!(concurrent_counter(1, 100), 100);
    }

    #[test]
    fn test_counter_multi_thread() {
        assert_eq!(concurrent_counter(10, 100), 1000);
    }

    #[test]
    fn test_counter_zero() {
        assert_eq!(concurrent_counter(5, 0), 0);
    }

    #[test]
    fn test_collect() {
        let result = concurrent_collect(5);
        assert_eq!(result, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_collect_single() {
        assert_eq!(concurrent_collect(1), vec![0]);
    }
}
