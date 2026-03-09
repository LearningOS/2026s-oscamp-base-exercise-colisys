//! # 异步通道
//!
//! 在本练习中，你将使用 `tokio::sync::mpsc` 异步通道实现生产者-消费者模式。
//!
//! ## 概念
//! - `tokio::sync::mpsc::channel` 创建有界异步通道
//! - 异步 `send` 和 `recv`
//! - 通道关闭机制（所有发送者被丢弃后接收者返回 None）

use tokio::sync::mpsc;

/// 异步生产者-消费者：
/// - 创建一个生产者任务，依次发送 items 中的每个元素
/// - 创建一个消费者任务，接收所有元素并收集到 Vec 中返回
///
/// 提示：设置通道容量为 items.len().max(1)
pub async fn producer_consumer(items: Vec<String>) -> Vec<String> {
    // TODO: 用 mpsc::channel 创建通道
    // TODO: 派生生产者任务：遍历 items，发送每一个
    // TODO: 派生消费者任务：循环 recv 直到通道关闭，收集结果
    // TODO: 等待消费者完成并返回结果
    todo!()
}

/// 扇入模式：多个生产者，一个消费者。
/// 创建 `n_producers` 个生产者，每个发送 `"producer {id}: message"`。
/// 消费者收集所有消息，排序后返回。
pub async fn fan_in(n_producers: usize) -> Vec<String> {
    // TODO: 创建 mpsc 通道
    // TODO: 派生 n_producers 个生产者任务
    //       每个发送 format!("producer {id}: message")
    // TODO: 丢弃原始发送者（重要！否则通道不会关闭）
    // TODO: 消费者循环接收，收集并排序
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_producer_consumer() {
        let items = vec!["hello".into(), "async".into(), "world".into()];
        let result = producer_consumer(items.clone()).await;
        assert_eq!(result, items);
    }

    #[tokio::test]
    async fn test_producer_consumer_empty() {
        let result = producer_consumer(vec![]).await;
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_fan_in() {
        let result = fan_in(3).await;
        assert_eq!(
            result,
            vec![
                "producer 0: message",
                "producer 1: message",
                "producer 2: message",
            ]
        );
    }

    #[tokio::test]
    async fn test_fan_in_single() {
        let result = fan_in(1).await;
        assert_eq!(result, vec!["producer 0: message"]);
    }
}
