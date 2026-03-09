//! # 线程创建
//!
//! 在本练习中，你将学习如何创建线程以及在线程间传递数据。
//!
//! ## 核心概念
//! - `std::thread::spawn` 用于创建新线程
//! - `move` 闭包捕获变量的所有权
//! - `JoinHandle::join()` 等待线程完成并获取返回值
//!
//! ## 高级线程操作
//! - **线程休眠**：`thread::sleep` 暂停当前线程。
//! - **线程本地存储**：`thread_local!` 宏定义每个线程独有的静态变量。
//! - **线程命名**：`Builder::name` 为线程分配名称，便于调试。
//! - **线程优先级**：通过 `thread::Builder` 设置（平台相关）。
//! - **线程池**：如 `rayon` 等库管理线程复用。
//! - **线程通信**：使用 `std::sync::mpsc`（多生产者单消费者）或第三方库（如 `crossbeam`）。
//! - **共享状态**：`Arc<Mutex<T>>` 或 `Arc<RwLock<T>>` 安全地在线程间共享可变数据。
//! - **同步原语**：`Barrier` 同步多个线程，`Condvar` 实现条件变量。
//! - **线程阻塞/唤醒**：`thread::park` 阻塞线程，`unpark` 唤醒线程，用于自定义调度。
//! - **获取当前线程句柄**：`thread::current()`。
//! - **作用域线程**：`crossbeam::scope` 或标准库的 `thread::scope`（Rust 1.63+）允许线程借用栈数据而无需 `move`。
//!
//! Rust 通过所有权系统以及 `Send` 和 `Sync` trait 在静态层面防止数据竞争。
//! 实现了 `Send` 的类型可以在线程间转移。
//! 实现了 `Sync` 的类型可以被多个线程同时引用。
//! 大多数 Rust 标准类型都是 `Send + Sync`；例外包括 `Rc<T>`（非原子引用计数）和裸指针。
//!
//! ## 练习结构
//! 1. **基础练习**（`double_in_thread`、`parallel_sum`）——介绍线程创建的基本概念。
//! 2. **进阶练习**（`named_sleeper`、`increment_thread_local`、`scoped_slice_sum`、`handle_panic`）——探索更多线程操作。
//! 每个函数都有一个 `TODO` 注释，指示你需要编写代码的位置。
//! 运行 `cargo test` 来检查你的实现。

#[allow(unused_imports)]
use std::cell::RefCell;
#[allow(unused_imports)]
use std::thread;
#[allow(unused_imports)]
use std::time::Duration;

// ============================================================================
// 示例代码：高级线程模式
// ============================================================================
// 以下示例说明了在 Rust 并发编程中有用的其他线程相关概念。

/// 示例：处理线程 panic。
///
/// `join()` 返回一个 `Result`。如果线程 panic，`Result` 为 `Err`。
/// 这演示了如何捕获和处理来自派生线程的 panic。
///
/// ```rust
/// use std::thread;
///
/// fn panic_handling_example() {
///     let handle = thread::spawn(|| {
///         // 模拟 panic
///         panic!("Thread panicked!");
///     });
///
///     match handle.join() {
///         Ok(_) => println!("Thread completed successfully."),
///         Err(e) => println!("Thread panicked: {:?}", e),
///     }
/// }
/// ```
///
/// 相比之下，下面的练习为了简单起见使用 `unwrap()`，假设线程不会 panic。

/// 示例：命名线程和自定义栈大小。
///
/// 使用 `thread::Builder` 可以为线程分配名称（有助于调试）并设置其栈大小。
///
/// ```rust
/// use std::thread;
///
/// fn named_thread_example() {
///     let builder = thread::Builder::new()
///         .name("my-worker".into())
///         .stack_size(32 * 1024); // 32 KiB
///
///     let handle = builder.spawn(|| {
///         println!("Hello from thread: {:?}", thread::current().name());
///         42
///     }).unwrap();
///
///     let result = handle.join().unwrap();
///     println!("Thread returned: {}", result);
/// }
/// ```

/// 示例：作用域线程（Rust 1.63+）。
///
/// 作用域线程允许借用栈数据而无需转移所有权。
/// 线程保证在作用域结束前完成，因此引用保持有效。
///
/// ```rust
/// use std::thread;
///
/// fn scoped_thread_example() {
///     let a = vec![1, 2, 3];
///     let b = vec![4, 5, 6];
///
///     let (sum_a, sum_b) = thread::scope(|s| {
///         let h1 = s.spawn(|| a.iter().sum::<i32>());
///         let h2 = s.spawn(|| b.iter().sum::<i32>());
///         (h1.join().unwrap(), h2.join().unwrap())
///     });
///
///     // `a` 和 `b` 在这里仍然可以访问。
///     println!("sum_a = {}, sum_b = {}", sum_a, sum_b);
/// }
/// ```

/// 示例：线程本地存储。
///
/// 每个线程获得 `thread_local!` 变量的独立副本。
///
/// ```rust
/// use std::cell::RefCell;
/// use std::thread;
///
/// thread_local! {
///     static THREAD_ID: RefCell<usize> = RefCell::new(0);
/// }
///
/// fn thread_local_example() {
///     THREAD_ID.with(|id| {
///         *id.borrow_mut() = 1;
///     });
///
///     let handle = thread::spawn(|| {
///         THREAD_ID.with(|id| {
///             *id.borrow_mut() = 2;
///         });
///         THREAD_ID.with(|id| println!("Thread local value: {}", *id.borrow()));
///     });
///
///     handle.join().unwrap();
///
///     THREAD_ID.with(|id| println!("Main thread value: {}", *id.borrow()));
/// }
/// ```

// ============================================================================
// 练习函数
// ============================================================================

/// 在新线程中将向量的每个元素乘以 2，返回结果向量。
///
/// 提示：使用 `thread::spawn` 和 `move` 闭包。
#[allow(unused_variables)]
pub fn double_in_thread(numbers: Vec<i32>) -> Vec<i32> {
    // TODO: 创建一个新线程，将 numbers 中的每个元素乘以 2
    // 使用 thread::spawn 和 move 闭包
    // 使用 join().unwrap() 获取结果
    thread::spawn(move || numbers.into_iter().map(|x| x * 2).collect())
        .join()
        .unwrap()
}

/// 并行计算两个向量的和，返回两个和的元组。
///
/// 提示：为每个向量创建两个线程。
#[allow(unused_variables)]
pub fn parallel_sum(a: Vec<i32>, b: Vec<i32>) -> (i32, i32) {
    // TODO: 创建两个线程分别对 a 和 b 求和
    // 等待两个线程获取结果
    // thread::spawn(move || {
    //     a.into_iter()
    //         .zip(b)
    //         .reduce(|(mut acc_a, mut acc_b), (a, b)| {
    //             acc_a += a;
    //             acc_b += b;
    //             (acc_a, acc_b)
    //         })
    //         .unwrap()
    // })
    // .join()
    // .unwrap()
    let th1 = thread::spawn(move || a.iter().sum::<i32>());
    let th2 = thread::spawn(move || b.iter().sum::<i32>());
    (th1.join().unwrap(), th2.join().unwrap())
}

// ============================================================================
// 进阶练习函数
// ============================================================================

/// 创建一个命名线程，休眠指定的毫秒数后返回输入值。
///
/// 线程应命名为 `"sleeper"`。使用 `thread::Builder` 设置名称。
/// 在线程内部，先调用 `thread::sleep(Duration::from_millis(ms))`，然后返回 `value`。
///
/// 提示：`thread::sleep` 使当前线程阻塞；它不会影响其他线程。
#[allow(unused_variables)]
pub fn named_sleeper(value: i32, ms: u64) -> i32 {
    // TODO: 创建一个名为 "sleeper" 的线程构建器
    // TODO: 派生一个线程，休眠 `ms` 毫秒后返回 `value`
    // TODO: 等待线程完成并返回值
    thread::Builder::new()
        .name("sleeper".to_string())
        .spawn(move || {
            thread::sleep(Duration::from_micros(ms));
            value
        })
        .unwrap()
        .join()
        .unwrap()
}

thread_local! {
    static THREAD_COUNT: RefCell<usize> = RefCell::new(0);
}

/// 使用线程本地存储来统计每个线程调用 `increment` 的次数。
///
/// 定义一个 `thread_local!` 静态变量 `THREAD_COUNT`，类型为 `RefCell<usize>`，初始化为 0。
/// 每次调用 `increment` 应该将线程本地计数加 1 并返回新值。
///
/// 提示：使用 `THREAD_COUNT.with(|cell| { ... })` 来访问线程本地变量。
pub fn increment_thread_local() -> usize {
    // TODO: 使用 THREAD_COUNT.with 递增并返回新的计数值
    THREAD_COUNT.with_borrow_mut(|v| {
        *v += 1;
        *v
    })
}

/// 使用**作用域线程**派生两个线程，计算两个切片的和，无需转移所有权。
///
/// 使用 `thread::scope` 允许线程借用切片 `&[i32]`。
/// 每个线程计算其切片的和，函数返回 `(sum_a, sum_b)`。
///
/// 提示：切片是引用，所以不能将它们 move 到闭包中。
/// `thread::scope` 保证所有派生的线程在作用域结束前完成，
/// 使得借用是安全的。
#[allow(unused_variables)]
pub fn scoped_slice_sum(a: &[i32], b: &[i32]) -> (i32, i32) {
    // TODO: 使用 thread::scope 派生两个线程
    // TODO: 每个线程计算其切片的和
    // TODO: 等待两个线程并返回结果
    thread::scope(|s| {
        let th1 = s.spawn(|| a.iter().sum::<i32>());
        let th2 = s.spawn(|| b.iter().sum::<i32>());
        (th1.join().unwrap(), th2.join().unwrap())
    })
}

/// 处理派生线程中可能发生的 panic。
///
/// 派生一个可能 panic 的线程：如果 `should_panic` 为 `true`，线程调用 `panic!("oops")`；
/// 否则返回 `value`。
/// 如果线程成功完成，函数应返回 `Ok(value)`；
/// 如果线程 panic，函数应返回 `Err(())`。
///
/// 提示：`join()` 返回 `Result<Result<i32, Box<dyn Any + Send>>, _>`。
/// 你需要匹配外层的 `Result`（线程 panic）和内层的 `Result`（如果线程返回一个 `Result`）。
/// 在本练习中，内部类型只是 `i32`，不是 `Result`。
#[allow(unused_variables)]
pub fn handle_panic(value: i32, should_panic: bool) -> Result<i32, ()> {
    // TODO: 派生一个线程，要么 panic 要么返回 value
    // TODO: 等待线程并正确映射结果
    match thread::spawn(move || if should_panic { panic!("oops") } else { value }).join() {
        Ok(v) => Ok(v),
        Err(_) => Err(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_double_basic() {
        let nums = vec![1, 2, 3, 4, 5];
        assert_eq!(double_in_thread(nums), vec![2, 4, 6, 8, 10]);
    }

    #[test]
    fn test_double_empty() {
        assert_eq!(double_in_thread(vec![]), vec![]);
    }

    #[test]
    fn test_double_negative() {
        assert_eq!(double_in_thread(vec![-1, 0, 1]), vec![-2, 0, 2]);
    }

    #[test]
    fn test_parallel_sum() {
        let a = vec![1, 2, 3];
        let b = vec![10, 20, 30];
        assert_eq!(parallel_sum(a, b), (6, 60));
    }

    #[test]
    fn test_parallel_sum_empty() {
        assert_eq!(parallel_sum(vec![], vec![]), (0, 0));
    }

    // 进阶练习测试
    #[test]
    fn test_named_sleeper() {
        // 线程应该休眠一小段时间；我们只是验证它返回正确的值。
        let result = named_sleeper(42, 10); // 休眠 10 毫秒
        assert_eq!(result, 42);
    }

    #[test]
    fn test_thread_local() {
        // 每个线程都有自己的计数器，所以派生两个线程并在每个线程中调用 increment
        // 应该让每个线程拥有自己独立的计数序列。
        use std::sync::Arc;
        use std::sync::Mutex;

        let counters = Arc::new(Mutex::new(Vec::new()));
        let mut handles = Vec::new();
        for _ in 0..2 {
            let counters = Arc::clone(&counters);
            handles.push(thread::spawn(move || {
                let v1 = increment_thread_local();
                let v2 = increment_thread_local();
                counters.lock().unwrap().push((v1, v2));
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        let results = counters.lock().unwrap();
        // 每个线程应该独立计数得到 (1, 2)。
        assert_eq!(results.len(), 2);
        assert!(results.contains(&(1, 2)));
    }

    #[test]
    fn test_scoped_slice_sum() {
        let a = [1, 2, 3];
        let b = [10, 20, 30];
        let (sum_a, sum_b) = scoped_slice_sum(&a, &b);
        assert_eq!(sum_a, 6);
        assert_eq!(sum_b, 60);
        // 确保切片仍然可以访问（它们是被借用，而不是被移动）。
        assert_eq!(a.len(), 3);
        assert_eq!(b.len(), 3);
    }

    #[test]
    fn test_handle_panic_ok() {
        let result = handle_panic(100, false);
        assert_eq!(result, Ok(100));
    }

    #[test]
    fn test_handle_panic_error() {
        let result = handle_panic(100, true);
        assert_eq!(result, Err(()));
    }
}
