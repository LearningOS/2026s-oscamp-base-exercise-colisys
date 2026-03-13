//! # 跨架构系统调用 ABI 描述与封装
//!
//! 通过填充结构体字段，描述 x86_64、aarch64 和 riscv64 在 Linux 上的系统调用 ABI。
//! 并通过条件编译在当前平台上实现真正的系统调用。
//!
//! ## 背景
//!
//! 不同的 CPU 架构使用不同的指令和寄存器来触发系统调用：
//!
//! | 架构     | 指令        | 系统调用号寄存器 | 返回值寄存器 | 参数寄存器                       |
//! |----------|-------------|----------------|------------|---------------------------------|
//! | x86_64   | `syscall`   | rax            | rax        | rdi, rsi, rdx, r10, r8, r9     |
//! | aarch64  | `svc #0`    | x8             | x0         | x0, x1, x2, x3, x4, x5        |
//! | riscv64  | `ecall`     | a7             | a0         | a0, a1, a2, a3, a4, a5         |
//!
//! ## 任务
//!
//! 1. 实现 `x86_64_abi()`、`aarch64_abi()`、`riscv64_abi()` —— 返回描述各架构 ABI 的结构体
//! 2. （条件编译）在当前平台上实现真正的 `syscall3` 内联汇编
//! 3. 在 `syscall3` 之上构建 `sys_write` / `sys_read` / `sys_close` / `sys_exit`
//!
//! ## 提示
//!
//! - Linux 系统调用号在不同架构上不同；x86_64 与 aarch64/riscv64 差异较大
//! - x86_64 的 `syscall` 指令会破坏 rcx 和 r11 寄存器
//! - aarch64 和 riscv64 共享统一的系统调用号表（来自 asm-generic）

#![cfg_attr(not(test), no_std)]

/// 描述特定架构的 Linux 系统调用 ABI
pub struct SyscallABI {
    /// 架构名称："x86_64"、"aarch64"、"riscv64"
    pub arch: &'static str,
    /// 触发系统调用的指令："syscall"、"svc #0"、"ecall"
    pub instruction: &'static str,
    /// 存放系统调用号的寄存器
    pub id_reg: &'static str,
    /// 存放返回值的寄存器
    pub ret_reg: &'static str,
    /// 参数寄存器（按顺序）
    pub arg_regs: &'static [&'static str],
    /// 系统调用指令额外破坏的寄存器
    pub clobbered: &'static [&'static str],
    /// write 系统调用号
    pub sys_write: usize,
    /// read 系统调用号
    pub sys_read: usize,
    /// close 系统调用号
    pub sys_close: usize,
    /// exit 系统调用号
    pub sys_exit: usize,
}

/// 返回 x86_64 Linux 系统调用 ABI 描述
pub fn x86_64_abi() -> SyscallABI {
    // TODO: 填充 x86_64 系统调用 ABI
    // 提示：x86_64 使用 "syscall" 指令，系统调用号在 rax 中
    // todo!();
    SyscallABI {
        arch: "x86_64",
        instruction: "syscall",
        id_reg: "rax",
        ret_reg: "rax",
        arg_regs: &["rdi", "rsi", "rdx", "r10", "r8", "r9"],
        clobbered: &["rcx", "r11"],
        sys_write: 1,
        sys_read: 0,
        sys_close: 3,
        sys_exit: 60,
    }
}

/// 返回 aarch64 Linux 系统调用 ABI 描述
pub fn aarch64_abi() -> SyscallABI {
    // TODO: 填充 aarch64 系统调用 ABI
    // 提示：aarch64 使用 "svc #0" 指令，系统调用号在 x8 中
    // todo!()
    SyscallABI {
        arch: "aarch64",
        instruction: "svc #0",
        id_reg: "x8",
        ret_reg: "x0",
        arg_regs: &["x0", "x1", "x2", "x3", "x4", "x5"],
        clobbered: &[],
        sys_write: 64,
        sys_read: 63,
        sys_close: 57,
        sys_exit: 93,
    }
}

/// 返回 riscv64 Linux 系统调用 ABI 描述
pub fn riscv64_abi() -> SyscallABI {
    // TODO: 填充 riscv64 系统调用 ABI
    // 提示：riscv64 使用 "ecall" 指令，系统调用号在 a7 中
    // todo!()
    SyscallABI {
        arch: "riscv64",
        instruction: "ecall",
        id_reg: "a7",
        ret_reg: "a0",
        arg_regs: &["a0", "a1", "a2", "a3", "a4", "a5"],
        clobbered: &[],
        sys_write: 64,
        sys_read: 63,
        sys_close: 57,
        sys_exit: 93,
    }
}

// ============================================================
// 真正的系统调用实现（条件编译，仅在匹配的平台上生效）
// ============================================================

/// 发起一个最多 3 个参数的 Linux 系统调用。
///
/// # 安全性
/// 调用者必须确保系统调用号和参数有效。
#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
pub unsafe fn syscall3(id: usize, arg0: usize, arg1: usize, arg2: usize) -> isize {
    // TODO: 使用 core::arch::asm! 实现 x86_64 系统调用
    // 提示：
    //   - "syscall" 指令
    //   - inlateout("rax") id => ret
    //   - in("rdi") arg0, in("rsi") arg1, in("rdx") arg2
    //   - out("rcx") _, out("r11") _
    // todo!()
    let mut ret: isize = 0;
    core::arch::asm!(
        "syscall",
        inlateout("rax") id => ret,
        in("rdi") arg0, in("rsi") arg1, in("rdx") arg2,
        out("rcx") _, out("r11") _
    );
    ret
}

#[cfg(all(target_arch = "aarch64", target_os = "linux"))]
pub unsafe fn syscall3(id: usize, arg0: usize, arg1: usize, arg2: usize) -> isize {
    // TODO: 使用 core::arch::asm! 实现 aarch64 系统调用
    // 提示：
    //   - "svc #0" 指令
    //   - in("x8") id
    //   - inlateout("x0") arg0 => ret
    //   - in("x1") arg1, in("x2") arg2
    // todo!()
    let mut ret: isize = 0;
    core::arch::asm!(
        "svc #0",
        in("x8") id,
        inlateout("x0") arg0 => ret,
        in("x1") arg1, in("x2") arg2
    );
    ret
}

// 非 Linux 平台：提供桩函数以便代码编译通过
#[cfg(not(target_os = "linux"))]
pub unsafe fn syscall3(_id: usize, _arg0: usize, _arg1: usize, _arg2: usize) -> isize {
    panic!("syscall3 is only available on Linux")
}

// 平台相关的 write 系统调用号
#[cfg(target_arch = "x86_64")]
const NATIVE_SYS_WRITE: usize = 1;
#[cfg(target_arch = "x86_64")]
const NATIVE_SYS_READ: usize = 0;
#[cfg(target_arch = "x86_64")]
const NATIVE_SYS_CLOSE: usize = 3;
#[cfg(target_arch = "x86_64")]
const NATIVE_SYS_EXIT: usize = 60;

#[cfg(target_arch = "aarch64")]
const NATIVE_SYS_WRITE: usize = 64;
#[cfg(target_arch = "aarch64")]
const NATIVE_SYS_READ: usize = 63;
#[cfg(target_arch = "aarch64")]
const NATIVE_SYS_CLOSE: usize = 57;
#[cfg(target_arch = "aarch64")]
const NATIVE_SYS_EXIT: usize = 93;

// 其他架构的回退（实际不使用，仅供编译通过）
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
const NATIVE_SYS_WRITE: usize = 0;
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
const NATIVE_SYS_READ: usize = 0;
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
const NATIVE_SYS_CLOSE: usize = 0;
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
const NATIVE_SYS_EXIT: usize = 0;

/// 将 `buf` 中的数据写入文件描述符 `fd`。
pub fn sys_write(fd: usize, buf: &[u8]) -> isize {
    // TODO: 调用 syscall3 实现 write
    // todo!()
    let buf_ptr = buf.as_ptr() as usize;
    unsafe { syscall3(NATIVE_SYS_WRITE, fd, buf_ptr, buf.len()) }
}

/// 从文件描述符 `fd` 读取数据到 `buf`。
pub fn sys_read(fd: usize, buf: &mut [u8]) -> isize {
    // TODO: 调用 syscall3 实现 read
    // todo!()
    let buf_ptr = buf.as_ptr() as usize;
    unsafe { syscall3(NATIVE_SYS_READ, fd, buf_ptr, buf.len()) }
}

/// 关闭文件描述符 `fd`。
pub fn sys_close(fd: usize) -> isize {
    // TODO: 调用 syscall3 实现 close
    // todo!()
    unsafe { syscall3(NATIVE_SYS_CLOSE, fd, 0, 0) }
}

/// 终止当前进程。
pub fn sys_exit(code: i32) -> ! {
    // TODO: 调用 syscall3 实现 exit
    // todo!()
    unsafe { syscall3(NATIVE_SYS_EXIT, code as usize, 0, 0) };
    loop {}
}

// ============================================================
// 测试
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;

    // ---- ABI 知识测试（可在任何平台运行）----

    #[test]
    fn test_x86_64_instruction() {
        let abi = x86_64_abi();
        assert_eq!(abi.arch, "x86_64");
        assert_eq!(abi.instruction, "syscall");
    }

    #[test]
    fn test_x86_64_registers() {
        let abi = x86_64_abi();
        assert_eq!(abi.id_reg, "rax");
        assert_eq!(abi.ret_reg, "rax");
        assert_eq!(
            abi.arg_regs,
            &["rdi", "rsi", "rdx", "r10", "r8", "r9"],
            "x86_64 argument register order is incorrect"
        );
    }

    #[test]
    fn test_x86_64_clobbered() {
        let abi = x86_64_abi();
        assert!(
            abi.clobbered.contains(&"rcx") && abi.clobbered.contains(&"r11"),
            "x86_64 syscall clobbers rcx and r11"
        );
    }

    #[test]
    fn test_x86_64_syscall_numbers() {
        let abi = x86_64_abi();
        assert_eq!(abi.sys_write, 1);
        assert_eq!(abi.sys_read, 0);
        assert_eq!(abi.sys_close, 3);
        assert_eq!(abi.sys_exit, 60);
    }

    #[test]
    fn test_aarch64_instruction() {
        let abi = aarch64_abi();
        assert_eq!(abi.arch, "aarch64");
        assert_eq!(abi.instruction, "svc #0");
    }

    #[test]
    fn test_aarch64_registers() {
        let abi = aarch64_abi();
        assert_eq!(abi.id_reg, "x8");
        assert_eq!(abi.ret_reg, "x0");
        assert_eq!(
            abi.arg_regs,
            &["x0", "x1", "x2", "x3", "x4", "x5"],
            "aarch64 argument register order is incorrect"
        );
    }

    #[test]
    fn test_aarch64_clobbered() {
        let abi = aarch64_abi();
        assert!(
            abi.clobbered.is_empty(),
            "aarch64 svc does not clobber additional registers"
        );
    }

    #[test]
    fn test_aarch64_syscall_numbers() {
        let abi = aarch64_abi();
        assert_eq!(abi.sys_write, 64);
        assert_eq!(abi.sys_read, 63);
        assert_eq!(abi.sys_close, 57);
        assert_eq!(abi.sys_exit, 93);
    }

    #[test]
    fn test_riscv64_instruction() {
        let abi = riscv64_abi();
        assert_eq!(abi.arch, "riscv64");
        assert_eq!(abi.instruction, "ecall");
    }

    #[test]
    fn test_riscv64_registers() {
        let abi = riscv64_abi();
        assert_eq!(abi.id_reg, "a7");
        assert_eq!(abi.ret_reg, "a0");
        assert_eq!(
            abi.arg_regs,
            &["a0", "a1", "a2", "a3", "a4", "a5"],
            "riscv64 argument register order is incorrect"
        );
    }

    #[test]
    fn test_riscv64_clobbered() {
        let abi = riscv64_abi();
        assert!(
            abi.clobbered.is_empty(),
            "riscv64 ecall does not clobber additional registers"
        );
    }

    #[test]
    fn test_riscv64_syscall_numbers() {
        let abi = riscv64_abi();
        assert_eq!(abi.sys_write, 64);
        assert_eq!(abi.sys_read, 63);
        assert_eq!(abi.sys_close, 57);
        assert_eq!(abi.sys_exit, 93);
    }

    #[test]
    fn test_aarch64_riscv64_share_numbers() {
        let aarch64 = aarch64_abi();
        let riscv64 = riscv64_abi();
        assert_eq!(
            aarch64.sys_write, riscv64.sys_write,
            "aarch64 and riscv64 share asm-generic syscall numbers"
        );
        assert_eq!(aarch64.sys_read, riscv64.sys_read);
        assert_eq!(aarch64.sys_close, riscv64.sys_close);
        assert_eq!(aarch64.sys_exit, riscv64.sys_exit);
    }

    // ---- 真正的系统调用测试（仅在 Linux 上运行）----

    #[cfg(target_os = "linux")]
    mod linux_tests {
        use super::*;

        #[test]
        fn test_sys_write_stdout() {
            let msg = b"[syscall_wrapper] sys_write test\n";
            let ret = sys_write(1, msg);
            assert_eq!(
                ret,
                msg.len() as isize,
                "sys_write should return bytes written"
            );
        }

        #[test]
        fn test_sys_write_stderr() {
            let msg = b"[syscall_wrapper] stderr test\n";
            let ret = sys_write(2, msg);
            assert_eq!(ret, msg.len() as isize);
        }

        #[test]
        fn test_sys_write_invalid_fd() {
            let ret = sys_write(999, b"hello");
            assert!(ret < 0, "invalid fd should return negative, got {ret}");
        }

        #[test]
        fn test_sys_close_invalid_fd() {
            let ret = sys_close(999);
            assert!(ret < 0, "closing invalid fd should return negative");
        }
    }
}
