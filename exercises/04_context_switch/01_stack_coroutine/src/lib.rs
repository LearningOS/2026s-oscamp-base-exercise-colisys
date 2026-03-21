//! # 有栈协程与上下文切换（riscv64）
//!
//! 在本练习中，你将使用内联汇编实现最小的上下文切换，
//! 这是操作系统线程调度的核心机制。此 crate **仅限 riscv64**；
//! 在 riscv64 Linux 上运行 `cargo test`，或在 x86 上使用仓库的标准流程（`./check.sh` / `oscamp`）配合 QEMU。
//!
//! ## 核心概念
//! - **被调用者保存寄存器**：切换时保存和恢复它们，使被切换走的任务稍后能正确恢复执行。
//! - **栈指针 `sp`** 和 **返回地址 `ra`**：在新上下文中恢复它们；首次切换到任务时，`ret` 跳转到 `ra`（入口点）。
//! - 内联汇编：`core::arch::asm!`
//!
//! ## riscv64 ABI（本练习）
//! - 被调用者保存：`sp`、`ra`、`s0`–`s11`。`ret` 指令是 `jalr zero, 0(ra)`。
//! - 第一和第二个参数：`a0`（旧上下文）、`a1`（新上下文）。

#![cfg(target_arch = "riscv64")]
#![feature(naked_functions_rustic_abi)]

/// 一个任务的保存寄存器状态（riscv64）。布局必须与下面汇编中使用的偏移量匹配：
/// `sp` 在 0，`ra` 在 8，然后 `s0`–`s11` 在 16, 24, … 104。
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct TaskContext {
    pub sp: u64,
    pub ra: u64,
    pub s0: u64,
    pub s1: u64,
    pub s2: u64,
    pub s3: u64,
    pub s4: u64,
    pub s5: u64,
    pub s6: u64,
    pub s7: u64,
    pub s8: u64,
    pub s9: u64,
    pub s10: u64,
    pub s11: u64,
}

impl TaskContext {
    pub const fn empty() -> Self {
        Self {
            sp: 0,
            ra: 0,
            s0: 0,
            s1: 0,
            s2: 0,
            s3: 0,
            s4: 0,
            s5: 0,
            s6: 0,
            s7: 0,
            s8: 0,
            s9: 0,
            s10: 0,
            s11: 0,
        }
    }

    /// 初始化此上下文，使得切换到它时，执行从 `entry` 开始。
    ///
    /// - 设置 `ra = entry`，使得新上下文中第一次 `ret` 跳转到 `entry`。
    /// - 设置 `sp = stack_top`，16 字节对齐（RISC-V ABI 要求函数入口处栈 16 字节对齐）。
    /// - `s0`–`s11` 保持零；它们会在切换时被加载。
    pub fn init(&mut self, stack_top: usize, entry: usize) {
        println!("sp:{}", stack_top);
        //todo!("设置 ra = entry, sp = stack_top（16 字节对齐）");
        self.ra = entry as u64;
        // 低 4 位硬连线，像 arm 的 pc；log2(16) = 4
        self.sp = ((stack_top as u64) >> 4) << 4;
    }
}

/// 从 `old` 切换到 `new` 上下文：将当前被调用者保存寄存器保存到 `old`，从 `new` 加载，然后 `ret`（跳转到 `new.ra`）。
///
/// 在汇编中：将 `sp`、`ra`、`s0`–`s11` 存储到 `[a0]`（old），从 `[a1]`（new）加载，将 `a0`/`a1` 清零以免向新上下文泄漏指针，然后 `ret`。
///
/// 必须是 `#[unsafe(naked)]` 以防止编译器生成序言/尾声。
#[unsafe(naked)]
pub unsafe fn switch_context(old: &mut TaskContext, new: &TaskContext) {
    // todo!("将被调用者保存寄存器保存到 old，从 new 加载，然后 ret；使用 #[unsafe(naked)] + naked_asm!，参见模块文档了解 riscv64 ABI 和布局")

    //
    // Loads and Stores
    // (l)oad   (b)yte/(h)alfword/(w)ord
    // (s)tore  (b)yte/(h)alfword/(w)ord
    //
    // asm 写法：
    // asm! (
    //   "assembly code",
    //   [in/out/inout/lateout] (reg) {variable} => {variable},
    // )
    //
    // 因为这里是 64 位，所以用 sd 指令
    // sd rs2, offset(rs1); rs2 -> [rs1] + #offset;

    core::arch::naked_asm!(
        r#"sd sp, 0(a0);
        sd ra, 8(a0);
        sd s0, 16(a0);
        sd s1, 24(a0);
        sd s2, 32(a0);
        sd s3, 40(a0);
        sd s4, 48(a0);
        sd s5, 56(a0);
        sd s6, 64(a0);
        sd s7, 72(a0);
        sd s8, 80(a0);
        sd s9, 88(a0);
        sd s10, 96(a0);
        sd s11, 104(a0);

        # 这里用 ld 指令加载双字
        # ld rd, offset(rs1); rd <- [rs1] + #offset

        ld sp, 0(a1);
        ld ra, 8(a1);
        ld s0, 16(a1);
        ld s1, 24(a1);
        ld s2, 32(a1);
        ld s3, 40(a1);
        ld s4, 48(a1);
        ld s5, 56(a1);
        ld s6, 64(a1);
        ld s7, 72(a1);
        ld s8, 80(a1);
        ld s9, 88(a1);
        ld s10, 96(a1);
        ld s11, 104(a1);

        # 清除 a0, a1

        xor a0, a0, a0;
        xor a1, a1, a1;

        # 执行 ret，此时上下文已经换成 new 里面的了

        ret"#
    );
}

const STACK_SIZE: usize = 1024 * 64;

/// 为协程分配栈。返回 `(buffer, stack_top)`，其中 `stack_top` 是高地址（栈向下增长）。
/// buffer 必须在使用此栈的上下文生命周期内保持有效。
pub fn alloc_stack() -> (Vec<u8>, usize) {
    // todo!("分配栈缓冲区，返回 (buffer, stack_top)，其中 stack_top 16 字节对齐")
    let buf = vec![0u8; STACK_SIZE];

    let raw_pointer = buf.as_ptr() as usize;
    (buf, raw_pointer + STACK_SIZE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    extern "C" fn task_entry() {
        COUNTER.store(42, Ordering::SeqCst);
        loop {
            std::hint::spin_loop();
        }
    }

    #[test]
    fn test_alloc_stack() {
        let (buf, top) = alloc_stack();
        assert_eq!(top, buf.as_ptr() as usize + STACK_SIZE);
        assert!(top % 16 == 0);
    }

    #[test]
    fn test_context_init() {
        let (buf, top) = alloc_stack();
        let _ = buf;
        let mut ctx = TaskContext::empty();
        let entry = task_entry as *const () as usize;
        ctx.init(top, entry);
        assert_eq!(ctx.ra, entry as u64);
        assert!(ctx.sp != 0);
    }

    #[test]
    fn test_switch_to_task() {
        COUNTER.store(0, Ordering::SeqCst);

        static mut MAIN_CTX_PTR: *mut TaskContext = std::ptr::null_mut();
        static mut TASK_CTX_PTR: *mut TaskContext = std::ptr::null_mut();

        extern "C" fn cooperative_task() {
            COUNTER.store(99, Ordering::SeqCst);
            unsafe {
                switch_context(&mut *TASK_CTX_PTR, &*MAIN_CTX_PTR);
            }
        }

        let (_stack_buf, stack_top) = alloc_stack();
        let mut main_ctx = TaskContext::empty();
        let mut task_ctx = TaskContext::empty();
        task_ctx.init(stack_top, cooperative_task as *const () as usize);

        unsafe {
            MAIN_CTX_PTR = &mut main_ctx;
            TASK_CTX_PTR = &mut task_ctx;
            switch_context(&mut main_ctx, &task_ctx);
        }

        assert_eq!(COUNTER.load(Ordering::SeqCst), 99);
    }
}
