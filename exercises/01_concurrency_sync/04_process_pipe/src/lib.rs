//! # 进程与管道
//!
//! 在本练习中，你将学习如何创建子进程并通过管道进行通信。
//!
//! ## 核心概念
//! - `std::process::Command` 创建子进程（对应 `fork()` + `execve()` 系统调用）
//! - `Stdio::piped()` 设置管道（对应 `pipe()` + `dup2()` 系统调用）
//! - 通过 stdin/stdout 与子进程通信
//! - 获取子进程退出状态（对应 `waitpid()` 系统调用）
//!
//! ## 操作系统概念映射
//! 本练习演示了底层操作系统原语在用户空间的抽象：
//! - **进程创建**：Rust 的 `Command::new()` 内部调用 `fork()` 创建子进程，
//!   然后调用 `execve()`（或等效函数）将子进程的内存映像替换为目标程序。
//! - **进程间通信（IPC）**：管道是内核管理的缓冲区，允许相关进程之间的单向数据流。
//!   `pipe()` 系统调用创建一个管道，返回两个文件描述符（读端、写端）。
//!   `dup2()` 复制文件描述符，实现标准输入/输出重定向。
//! - **资源管理**：文件描述符（包括管道端）在 Rust 的 `Stdio` 对象被丢弃时自动关闭，
//!   防止资源泄漏。
//!
//! ## 练习结构
//! 1. **基础命令执行**（`run_command`）—— 启动子进程并捕获其标准输出。
//! 2. **双向管道通信**（`pipe_through_cat`）—— 向子进程（`cat`）发送数据并读取其输出。
//! 3. **退出码获取**（`get_exit_code`）—— 获取子进程的终止状态。
//! 4. **进阶：错误处理版本**（`run_command_with_result`）—— 学习正确的错误传播。
//! 5. **进阶：复杂双向通信**（`pipe_through_grep`）—— 与过滤器程序交互，
//!    读取多行输入并产生过滤后的输出。
//!
//! 每个函数都有一个 `TODO` 注释，指示你需要编写代码的位置。
//! 运行 `cargo test` 来检查你的实现。

use std::io::{self, Read, Write};
use std::process::{self, Command, Stdio};

/// 执行给定的 shell 命令并返回其标准输出。
///
/// 例如：`run_command("echo", &["hello"])` 应返回 `"hello\n"`
///
/// # 底层系统调用
/// - `Command::new(program)` → `fork()` + `execve()` 系列
/// - `Stdio::piped()` → `pipe()` + `dup2()`（为 stdout 设置管道）
/// - `.output()` → `waitpid()`（等待子进程终止）
///
/// # 实现步骤
/// 1. 用给定的程序和参数创建一个 `Command`。
/// 2. 设置 `.stdout(Stdio::piped())` 以捕获子进程的标准输出。
/// 3. 调用 `.output()` 执行子进程并获取 `Output`。
/// 4. 将 `stdout` 字段（`Vec<u8>`）转换为 `String`。
pub fn run_command(program: &str, args: &[&str]) -> String {
    // TODO: 使用 Command::new 创建进程
    // TODO: 设置 stdout 为 Stdio::piped()
    // TODO: 用 .output() 执行并获取输出
    // TODO: 将 stdout 转换为 String 并返回
    let output_u8 = &Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .output()
        .unwrap()
        .stdout;
    String::from_utf8_lossy(output_u8).to_string()
}

/// 通过管道向子进程（cat）的 stdin 写入数据，并读取其 stdout 输出。
///
/// 这演示了父子进程之间的双向管道通信。
///
/// # 底层系统调用
/// - `Command::new("cat")` → `fork()` + `execve("cat")`
/// - `Stdio::piped()`（两次）→ `pipe()` 创建两个管道（stdin 和 stdout）+ `dup2()` 重定向
/// - `ChildStdin::write_all()` → `write()` 写入管道写端
/// - `drop(stdin)` → `close()` 关闭写端，向子进程发送 EOF
/// - `ChildStdout::read_to_string()` → `read()` 从管道读端读取
///
/// # 所有权与资源管理
/// Rust 的所有权系统确保管道在正确的时机被关闭：
/// 1. `ChildStdin` 句柄由父进程拥有；向其写入数据会将数据传输给子进程。
/// 2. 写入完成后，我们显式 `drop(stdin)`（或让其离开作用域）以关闭写端。
/// 3. 关闭写端向 `cat` 发送 EOF 信号，使其处理完所有输入后退出。
/// 4. 然后读取 `ChildStdout` 句柄直到结束；丢弃它会关闭读端。
///
/// 如果不丢弃 `stdin`，子进程将永远等待更多输入（管道永远不会关闭）。
///
/// # 实现步骤
/// 1. 为 `"cat"` 创建一个 `Command`，设置 `.stdin(Stdio::piped())` 和 `.stdout(Stdio::piped())`。
/// 2. 调用 `.spawn()` 启动命令，获得带有 `stdin` 和 `stdout` 句柄的 `Child`。
/// 3. 将 `input` 字节写入子进程的 stdin（`child.stdin.take().unwrap().write_all(...)`）。
/// 4. 丢弃 stdin 句柄（显式 `drop` 或让其离开作用域）以关闭管道。
/// 5. 读取子进程的 stdout（`child.stdout.take().unwrap().read_to_string(...)`）。
/// 6. 调用 `.wait()` 等待子进程退出（或依赖 drop 时自动等待）。
pub fn pipe_through_cat(input: &str) -> String {
    // TODO: 创建 "cat" 命令，设置 stdin 和 stdout 为 piped
    // TODO: 启动进程
    // TODO: 将 input 写入子进程 stdin
    // TODO: 丢弃 stdin 以关闭管道（否则 cat 不会退出）
    // TODO: 从子进程 stdout 读取输出
    let mut process = Command::new("cat")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    {
        let mut stdin = process.stdin.take().unwrap();
        stdin.write_all(input.as_bytes()).unwrap();
        drop(stdin);
    }

    process.wait().unwrap();
    let mut s = String::new();
    process
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut s)
        .unwrap();
    s
}

/// 获取子进程退出码。
/// 执行命令 `sh -c {command}` 并返回退出码。
///
/// # 底层系统调用
/// - `Command::new("sh")` → `fork()` + `execve("/bin/sh")`
/// - `.args(["-c", command])` 传递 shell 命令行
/// - `.status()` → `waitpid()`（等待子进程并获取退出状态）
/// - `ExitStatus::code()` 提取低位退出码（0-255）
///
/// # 实现步骤
/// 1. 为 `"sh"` 创建一个 `Command`，参数为 `["-c", command]`。
/// 2. 调用 `.status()` 执行 shell 并获取 `ExitStatus`。
/// 3. 使用 `.code()` 获取 `Option<i32>` 类型的退出码。
/// 4. 如果子进程正常终止，返回退出码；否则返回默认值。
pub fn get_exit_code(command: &str) -> i32 {
    // TODO: 使用 Command::new("sh").args(["-c", command])
    // TODO: 执行并获取状态
    // TODO: 返回退出码
    let child = Command::new("sh").args(["-c", command]).spawn().unwrap();
    let result = child.wait_with_output().unwrap();
    result.status.code().unwrap()
}

/// 执行给定的 shell 命令并以 `Result` 形式返回其标准输出。
///
/// 此版本正确传播进程创建、执行或 I/O 过程中可能发生的错误
///（例如命令未找到、权限被拒绝、管道破裂）。
///
/// # 底层系统调用
/// 与 `run_command` 相同，但错误从操作系统捕获并以 `Err` 形式返回。
///
/// # 错误处理
/// - `Command::new()` 只构造构建器；错误发生在 `.output()` 时。
/// - `.output()` 返回 `Result<Output, std::io::Error>`。
/// - `String::from_utf8()` 可能失败，如果子进程输出不是有效的 UTF-8。
///   这种情况下我们返回一个类型为 `InvalidData` 的 `io::Error`。
///
/// # 实现步骤
/// 1. 用给定的程序和参数创建一个 `Command`。
/// 2. 设置 `.stdout(Stdio::piped())`。
/// 3. 调用 `.output()` 并传播任何 `io::Error`。
/// 4. 用 `String::from_utf8` 将 `stdout` 转换为 `String`；如果失败，映射为 `io::Error`。
pub fn run_command_with_result(program: &str, args: &[&str]) -> io::Result<String> {
    // TODO: 使用 Command::new 创建进程
    // TODO: 设置 stdout 为 Stdio::piped()
    // TODO: 用 .output() 执行并处理 Result
    // TODO: 用 from_utf8 将 stdout 转换为 String，将错误映射为 io::Error
    let output = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .output()?;
    Ok(String::from_utf8(output.stdout).unwrap())
}

/// 通过双向管道与 `grep` 交互，过滤包含特定模式的行。
///
/// 这演示了复杂的父子通信：父进程发送多行输入，
/// 子进程（`grep`）根据模式过滤它们，父进程读回匹配的行。
///
/// # 底层系统调用
/// - `Command::new("grep")` → `fork()` + `execve("grep")`
/// - 两个管道（stdin 和 stdout）与 `pipe_through_cat` 相同
/// - 逐行写入和读取以模拟交互式过滤
///
/// # 实现步骤
/// 1. 为 `"grep"` 创建一个 `Command`，参数为 `pattern`，两端都设置管道。
/// 2. 调用 `.spawn()` 启动命令，获得带有 `stdin` 和 `stdout` 句柄的 `Child`。
/// 3. 将 `input` 的每一行（以 `'\n'` 分隔）写入子进程的 stdin。
/// 4. 关闭写端（丢弃 stdin）以发送 EOF 信号。
/// 5. 逐行读取子进程的 stdout，收集匹配的行。
/// 6. 等待子进程退出（可选；`grep` 会在 EOF 后退出）。
/// 7. 将匹配的行拼接为单个 `String` 返回。
///
pub fn pipe_through_grep(pattern: &str, input: &str) -> String {
    // TODO: 创建 "grep" 命令并传入 pattern，设置 stdin 和 stdout 为 piped
    // TODO: 启动进程
    // TODO: 将输入行写入子进程 stdin
    // TODO: 丢弃 stdin 以关闭管道
    // TODO: 逐行读取子进程 stdout 输出
    // TODO: 收集并返回匹配的行
    let mut child = Command::new("grep")
        .args([pattern])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    {
        let mut stdin = child.stdin.take().unwrap();
        stdin.write_all(input.as_bytes()).unwrap();
        drop(stdin);
    };

    {
        let mut stdout = child.stdout.take().unwrap();
        let mut buf = vec![];
        stdout.read_to_end(&mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_echo() {
        let output = run_command("echo", &["hello"]);
        assert_eq!(output.trim(), "hello");
    }

    #[test]
    fn test_run_with_args() {
        let output = run_command("echo", &["-n", "no newline"]);
        assert_eq!(output, "no newline");
    }

    #[test]
    fn test_pipe_cat() {
        let output = pipe_through_cat("hello pipe!");
        assert_eq!(output, "hello pipe!");
    }

    #[test]
    fn test_pipe_multiline() {
        let input = "line1\nline2\nline3";
        assert_eq!(pipe_through_cat(input), input);
    }

    #[test]
    fn test_exit_code_success() {
        assert_eq!(get_exit_code("true"), 0);
    }

    #[test]
    fn test_exit_code_failure() {
        assert_eq!(get_exit_code("false"), 1);
    }

    #[test]
    fn test_run_command_with_result_success() {
        let result = run_command_with_result("echo", &["hello"]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "hello");
    }

    #[test]
    fn test_run_command_with_result_nonexistent() {
        let result = run_command_with_result("nonexistent_command_xyz", &[]);
        // 应该返回错误，因为命令不存在
        assert!(result.is_err());
    }

    #[test]
    fn test_pipe_through_grep_basic() {
        let input = "apple\nbanana\ncherry\n";
        let output = pipe_through_grep("a", input);
        // grep 输出匹配的行并带换行
        assert_eq!(output, "apple\nbanana\n");
    }

    #[test]
    fn test_pipe_through_grep_no_match() {
        let input = "apple\nbanana\ncherry\n";
        let output = pipe_through_grep("z", input);
        // 没有匹配行 -> 空字符串
        assert_eq!(output, "");
    }

    #[test]
    fn test_pipe_through_grep_multiline() {
        let input = "first line\nsecond line\nthird line\n";
        let output = pipe_through_grep("second", input);
        assert_eq!(output, "second line\n");
    }
}
