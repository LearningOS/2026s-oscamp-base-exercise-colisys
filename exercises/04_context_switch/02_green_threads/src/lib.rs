//! # 绿色线程调度器（riscv64）
//!
//! 在本练习中，你将在上下文切换的基础上构建一个简单的协作式（绿色）线程调度器。
//! 此 crate **仅限 riscv64**；使用仓库的标准流程（`./check.sh` / `oscamp`）或在 riscv64 上原生运行。
//!
//! ## 核心概念
//! - 协作式 vs 抢占式调度
//! - 线程状态：`Ready`、`Running`、`Finished`
//! - `yield_now()`：当前线程主动让出 CPU
//! - 调度器循环：选择下一个就绪线程并切换到它
//!
//! ## 设计
//! 每个绿色线程有自己的栈和 `TaskContext`。线程调用 `yield_now()` 来让出。
//! 调度器在就绪线程间轮转。用户入口被 `thread_wrapper` 包装，后者
//! 调用入口然后将线程标记为 `Finished` 并切换回去。

#![cfg(target_arch = "riscv64")]

use core::arch::naked_asm;

/// 每线程栈大小。稍大以避免在 QEMU / 测试框架下溢出。
const STACK_SIZE: usize = 1024 * 128;

/// 任务上下文（riscv64）；布局必须与 `01_stack_coroutine::TaskContext` 及下面汇编匹配。
#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct TaskContext {
    sp: u64,
    ra: u64,
    s0: u64,
    s1: u64,
    s2: u64,
    s3: u64,
    s4: u64,
    s5: u64,
    s6: u64,
    s7: u64,
    s8: u64,
    s9: u64,
    s10: u64,
    s11: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThreadState {
    Ready,
    Running,
    Finished,
}

struct GreenThread {
    ctx: TaskContext,
    state: ThreadState,
    _stack: Option<Vec<u8>>,
    /// 用户入口；在线程首次被调度时取出一次并传给 `thread_wrapper`。
    entry: Option<extern "C" fn()>,
}

/// 由调度器在切换到新线程前设置；`thread_wrapper` 读取并调用一次。
static mut CURRENT_THREAD_ENTRY: Option<extern "C" fn()> = None;

/// 作为每个绿色线程初始 `ra` 运行的包装函数：调用用户入口（从 `CURRENT_THREAD_ENTRY`），然后标记 Finished 并切换回去。
extern "C" fn thread_wrapper() {
    let entry = unsafe { core::ptr::read(&raw const CURRENT_THREAD_ENTRY) };
    if let Some(f) = entry {
        unsafe { CURRENT_THREAD_ENTRY = None };
        f();
    }
    thread_finished();
}

/// 将当前被调用者保存寄存器保存到 `old`，从 `new` 加载，然后 `ret` 到 `new.ra`。
/// 在 `ret` 前将 `a0`/`a1` 清零，以免向新上下文泄漏指针。
///
/// 必须是 `#[unsafe(naked)]` 以防止编译器生成序言/尾声。
#[unsafe(naked)]
unsafe extern "C" fn switch_context(_old: &mut TaskContext, _new: &TaskContext) {
    naked_asm!(
        "sd sp, 0(a0)",
        "sd ra, 8(a0)",
        "sd s0, 16(a0)",
        "sd s1, 24(a0)",
        "sd s2, 32(a0)",
        "sd s3, 40(a0)",
        "sd s4, 48(a0)",
        "sd s5, 56(a0)",
        "sd s6, 64(a0)",
        "sd s7, 72(a0)",
        "sd s8, 80(a0)",
        "sd s9, 88(a0)",
        "sd s10, 96(a0)",
        "sd s11, 104(a0)",
        "ld sp, 0(a1)",
        "ld ra, 8(a1)",
        "ld s0, 16(a1)",
        "ld s1, 24(a1)",
        "ld s2, 32(a1)",
        "ld s3, 40(a1)",
        "ld s4, 48(a1)",
        "ld s5, 56(a1)",
        "ld s6, 64(a1)",
        "ld s7, 72(a1)",
        "ld s8, 80(a1)",
        "ld s9, 88(a1)",
        "ld s10, 96(a1)",
        "ld s11, 104(a1)",
        "li a0, 0",
        "li a1, 0",
        "ret",
    );
}

pub struct Scheduler {
    threads: Vec<GreenThread>,
    current: usize,
}

impl Scheduler {
    pub fn new() -> Self {
        let main_thread = GreenThread {
            ctx: TaskContext::default(),
            state: ThreadState::Running,
            _stack: None,
            entry: None,
        };

        Self {
            threads: vec![main_thread],
            current: 0,
        }
    }

    /// 注册一个新绿色线程，首次调度时将运行 `entry`。
    ///
    /// 1. 分配 `STACK_SIZE` 字节的栈；计算 `stack_top`（高地址）。
    /// 2. 设置上下文：`ra = thread_wrapper`，使得首次切换跳转到包装函数；
    ///    `sp` 必须 16 字节对齐（例如 `(stack_top - 16) & !15` 以留出余量）。
    /// 3. 推入一个 `GreenThread`，包含此上下文、状态 `Ready`，以及存储的 `entry` 供包装函数调用。
    pub fn spawn(&mut self, entry: extern "C" fn()) {
        todo!("分配栈，初始化 ctx（ra=thread_wrapper，sp 对齐），推入 GreenThread(Ready, entry)")
    }

    /// 运行调度器直到所有线程（除主线程外）都 `Finished`。
    ///
    /// 1. 将全局 `SCHEDULER` 指针设为 `self`，以便 `yield_now` 和 `thread_finished` 可以回调。
    /// 2. 循环：如果 `threads[1..]` 全部 `Finished` 则退出；否则调用 `schedule_next()`（可能切换走后再返回）。
    /// 3. 完成后清除 `SCHEDULER`。
    pub fn run(&mut self) {
        todo!("将 SCHEDULER 设为 self，循环直到 threads[1..] 全部 Finished，调用 schedule_next，然后清除 SCHEDULER")
    }

    /// 找到下一个就绪线程（从 `current + 1` 开始轮转），将当前标记为 `Ready`（如果未 `Finished`），将下一个标记为 `Running`，如果下一个线程有入口则设置 `CURRENT_THREAD_ENTRY`，然后切换到它。
    fn schedule_next(&mut self) {
        todo!("轮转查找下一个 Ready，将当前设为 Ready（如未 Finished），下一个设为 Running，设置 CURRENT_THREAD_ENTRY，然后 switch_context")
    }
}

impl TaskContext {
    fn as_mut_ptr(&mut self) -> *mut TaskContext {
        self as *mut TaskContext
    }
    fn as_ptr(&self) -> *const TaskContext {
        self as *const TaskContext
    }
}

static mut SCHEDULER: *mut Scheduler = std::ptr::null_mut();

/// 当前线程主动让出；调度器将选择下一个就绪线程。
pub fn yield_now() {
    unsafe {
        if !SCHEDULER.is_null() {
            (*SCHEDULER).schedule_next();
        }
    }
}

/// 将当前线程标记为 `Finished` 并切换到下一个（由 `thread_wrapper` 在用户入口返回后调用）。
fn thread_finished() {
    unsafe {
        if !SCHEDULER.is_null() {
            let sched = &mut *SCHEDULER;
            sched.threads[sched.current].state = ThreadState::Finished;
            sched.schedule_next();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Mutex;

    /// 测试必须串行运行：调度器使用全局状态（SCHEDULER, CURRENT_THREAD_ENTRY）。
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    static EXEC_ORDER: AtomicU32 = AtomicU32::new(0);

    extern "C" fn task_a() {
        EXEC_ORDER.fetch_add(1, Ordering::SeqCst);
        yield_now();
        EXEC_ORDER.fetch_add(10, Ordering::SeqCst);
        yield_now();
        EXEC_ORDER.fetch_add(100, Ordering::SeqCst);
    }

    extern "C" fn task_b() {
        EXEC_ORDER.fetch_add(1, Ordering::SeqCst);
        yield_now();
        EXEC_ORDER.fetch_add(10, Ordering::SeqCst);
    }

    #[test]
    fn test_scheduler_runs_all() {
        let _guard = TEST_LOCK.lock().unwrap();
        EXEC_ORDER.store(0, Ordering::SeqCst);

        let mut sched = Scheduler::new();
        sched.spawn(task_a);
        sched.spawn(task_b);
        sched.run();

        let got = EXEC_ORDER.load(Ordering::SeqCst);
        if got != 122 {
            panic!(
                "EXEC_ORDER: 期望 122，得到 {}（使用 --nocapture 运行以查看 stderr）",
                got
            );
        }
    }

    static SIMPLE_FLAG: AtomicU32 = AtomicU32::new(0);

    extern "C" fn simple_task() {
        SIMPLE_FLAG.store(42, Ordering::SeqCst);
    }

    #[test]
    fn test_single_thread() {
        let _guard = TEST_LOCK.lock().unwrap();
        SIMPLE_FLAG.store(0, Ordering::SeqCst);

        let mut sched = Scheduler::new();
        sched.spawn(simple_task);
        sched.run();

        assert_eq!(SIMPLE_FLAG.load(Ordering::SeqCst), 42);
    }
}
