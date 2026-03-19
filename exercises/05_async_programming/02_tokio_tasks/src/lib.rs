//! # Tokio 异步任务
//!
//! 在本练习中，你将使用 `tokio::spawn` 创建并发异步任务。
//!
//! ## 概念
//! - `tokio::spawn` 创建异步任务
//! - `JoinHandle` 等待任务完成
//! - 异步任务之间的并发执行

use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

/// 并发计算 0..n 中每个数的平方，收集结果并按顺序返回。
///
/// 提示：为每个 i 创建 `tokio::spawn` 任务，收集 JoinHandle，按顺序 await 它们。
pub async fn concurrent_squares(n: usize) -> Vec<usize> {
    // TODO: 创建 n 个异步任务，每个计算 i * i
    // TODO: 收集所有 JoinHandle
    // TODO: 依次 await 获取结果
    // todo!();
    let mut result = vec![];
    for i in 0..n {
        result.push(tokio::spawn(async move { i * i }).await.unwrap())
    }
    result
}

/// 并发执行多个"耗时"任务（用 sleep 模拟），返回所有结果。
/// 每个任务休眠 `duration_ms` 毫秒然后返回其 `task_id`。
///
/// 关键：所有任务应并发执行，总时长应接近单个任务时长，而非所有任务时长之和。
pub async fn parallel_sleep_tasks(n: usize, duration_ms: u64) -> Vec<usize> {
    // TODO: 为 0..n 中的每个 id 创建异步任务
    // TODO: 每个任务休眠指定时长并返回自己的 id
    // TODO: 收集所有结果并排序
    // todo!();
    let mut result = vec![];
    for i in 0..n {
        result.push(
            tokio::spawn(async move {
                sleep(Duration::from_micros(duration_ms)).await;
                i
            })
            .await
            .unwrap(),
        );
    }
    result.sort();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Instant;

    #[tokio::test]
    async fn test_squares_basic() {
        let result = concurrent_squares(5).await;
        assert_eq!(result, vec![0, 1, 4, 9, 16]);
    }

    #[tokio::test]
    async fn test_squares_zero() {
        let result = concurrent_squares(0).await;
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_squares_one() {
        let result = concurrent_squares(1).await;
        assert_eq!(result, vec![0]);
    }

    #[tokio::test]
    async fn test_parallel_sleep() {
        let start = Instant::now();
        let result = parallel_sleep_tasks(5, 100).await;
        let elapsed = start.elapsed();

        assert_eq!(result, vec![0, 1, 2, 3, 4]);
        // 并发执行，总时间应远小于 5 * 100ms
        assert!(
            elapsed.as_millis() < 400,
            "任务应并发运行，耗时 {}ms",
            elapsed.as_millis()
        );
    }
}
