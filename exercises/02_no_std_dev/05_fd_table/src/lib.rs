//! # 文件描述符表
//!
//! 实现一个简单的文件描述符（fd）表——操作系统内核中管理打开文件的核心数据结构。
//!
//! ## 背景
//!
//! 在 Linux 内核中，每个进程都有一个 fd 表，将整数 fd 映射到内核文件对象。
//! 用户程序通过 fd 执行 read/write/close，内核通过 fd 表查找对应的文件对象。
//!
//! ```text
//! fd 表:
//!   0 -> Stdin
//!   1 -> Stdout
//!   2 -> Stderr
//!   3 -> File("/etc/passwd")
//!   4 -> (空)
//!   5 -> Socket(...)
//! ```
//!
//! ## 任务
//!
//! 在 `FdTable` 上实现以下方法：
//!
//! - `new()` —— 创建空的 fd 表
//! - `alloc(file)` -> `usize` —— 分配新的 fd，返回 fd 号
//!   - 优先复用最小的已关闭 fd 号
//!   - 如果没有空闲槽位，扩展表
//! - `get(fd)` -> `Option<Arc<dyn File>>` —— 获取 fd 对应的文件对象
//! - `close(fd)` -> `bool` —— 关闭 fd，返回是否成功（fd 不存在则返回 false）
//! - `count()` -> `usize` —— 返回当前已分配的 fd 数量（不包括已关闭的）
//!
//! ## 核心概念
//!
//! - Trait 对象：`Arc<dyn File>`
//! - `Vec<Option<T>>` 作为稀疏表
//! - fd 号复用策略（找最小空闲槽位）
//! - `Arc` 引用计数与资源释放

use std::{ops::BitOrAssign, sync::Arc};

/// 文件抽象 trait —— 内核中所有"文件"（普通文件、管道、套接字）都实现此 trait
pub trait File: Send + Sync {
    fn read(&self, buf: &mut [u8]) -> isize;
    fn write(&self, buf: &[u8]) -> isize;
}

/// 文件描述符表
pub struct FdTable {
    // TODO: 设计内部结构
    // 提示：使用 Vec<Option<Arc<dyn File>>>
    //       索引是 fd 号，None 表示 fd 已关闭或未分配
    table: Vec<Option<Arc<dyn File>>>,
}

impl FdTable {
    /// 创建空的 fd 表
    pub fn new() -> Self {
        // TODO
        FdTable { table: vec![] }
    }

    /// 分配新的 fd，返回 fd 号。
    ///
    /// 优先复用最小的已关闭 fd 号；如果没有空闲槽位，则追加到末尾。
    pub fn alloc(&mut self, file: Arc<dyn File>) -> usize {
        // TODO
        let mut fd: usize = 0;
        for ele in self.table.iter_mut() {
            if ele.is_none() {
                println!("alloc: found reusable fd {}", fd);
                *ele = Some(file);
                return fd;
            } else {
                println!("alloc: fd {} is some, skip", fd);
            }
            fd += 1;
        }
        println!("alloc: alloc new fd @ {}", fd);
        self.table.push(Some(file));
        fd
    }

    /// 获取 fd 对应的文件对象。如果 fd 不存在或已关闭，返回 None。
    pub fn get(&self, fd: usize) -> Option<Arc<dyn File>> {
        // TODO
        if let Some(_fd) = self.table.get(fd) {
            if let Some(_f) = _fd {
                return Some(_f.clone());
            }
        }

        None
    }

    /// 关闭 fd。成功返回 true，如果 fd 不存在或已关闭返回 false。
    pub fn close(&mut self, fd: usize) -> bool {
        // TODO
        if self.get(fd).is_some() {
            if let Some(_f) = self.table.get_mut(fd) {
                *_f = None;
                println!("close: set *_f @ {} to None, {:?}", fd, _f.is_none());
            }
            true
        } else {
            println!("close: not found fd {}", fd);
            false
        }
    }

    /// 返回当前已分配的 fd 数量（不包括已关闭的）
    pub fn count(&self) -> usize {
        // TODO
        self.table.iter().filter(|x| x.is_some()).count()
    }
}

impl Default for FdTable {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// 测试用的 File 实现
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct MockFile {
        id: usize,
        write_log: Mutex<Vec<Vec<u8>>>,
    }

    impl MockFile {
        fn new(id: usize) -> Arc<Self> {
            Arc::new(Self {
                id,
                write_log: Mutex::new(vec![]),
            })
        }
    }

    impl File for MockFile {
        fn read(&self, buf: &mut [u8]) -> isize {
            buf[0] = self.id as u8;
            1
        }
        fn write(&self, buf: &[u8]) -> isize {
            self.write_log.lock().unwrap().push(buf.to_vec());
            buf.len() as isize
        }
    }

    #[test]
    fn test_alloc_basic() {
        let mut table = FdTable::new();
        let fd = table.alloc(MockFile::new(0));
        assert_eq!(fd, 0, "first fd should be 0");
        let fd2 = table.alloc(MockFile::new(1));
        assert_eq!(fd2, 1, "second fd should be 1");
    }

    #[test]
    fn test_get() {
        let mut table = FdTable::new();
        let file = MockFile::new(42);
        let fd = table.alloc(file);
        assert_eq!(fd, 0);
        let got = table.get(fd);
        assert!(got.is_some(), "get should return Some");
        let mut buf = [0u8; 1];
        got.unwrap().read(&mut buf);
        assert_eq!(buf[0], 42);
    }

    #[test]
    fn test_get_invalid() {
        let table = FdTable::new();
        assert!(table.get(0).is_none());
        assert!(table.get(999).is_none());
    }

    #[test]
    fn test_close_and_reuse() {
        let mut table = FdTable::new();
        let fd0 = table.alloc(MockFile::new(0)); // fd=0
        let fd1 = table.alloc(MockFile::new(1)); // fd=1
        let fd2 = table.alloc(MockFile::new(2)); // fd=2

        assert!(table.close(fd1), "closing fd=1 should succeed");
        assert!(
            table.get(fd1).is_none(),
            "get should return None after close"
        );

        // 下次分配应该复用 fd=1（最小的空闲 fd）
        let fd_new = table.alloc(MockFile::new(99));
        assert_eq!(fd_new, fd1, "should reuse the smallest closed fd");

        let _ = (fd0, fd2);
    }

    #[test]
    fn test_close_invalid() {
        let mut table = FdTable::new();
        assert!(
            !table.close(0),
            "closing non-existent fd should return false"
        );
    }

    #[test]
    fn test_count() {
        let mut table = FdTable::new();
        assert_eq!(table.count(), 0);
        let fd0 = table.alloc(MockFile::new(0));
        let fd1 = table.alloc(MockFile::new(1));
        assert_eq!(table.count(), 2);
        assert!(table.close(fd0));
        assert_eq!(table.count(), 1);
        assert!(table.close(fd1));
        assert_eq!(table.count(), 0);
    }

    #[test]
    fn test_write_through_fd() {
        let mut table = FdTable::new();
        let file = MockFile::new(0);
        let fd = table.alloc(file);
        let f = table.get(fd).unwrap();
        let n = f.write(b"hello");
        assert_eq!(n, 5);
    }
}
