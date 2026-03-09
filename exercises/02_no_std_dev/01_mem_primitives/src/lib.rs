//! # no_std 内存原语
//!
//! 在 `#![no_std]` 环境中，没有标准库——只有 `core`。
//! 这些内存操作函数是操作系统内核中最基础的构建块。
//! 在裸机环境中，libc 中的 memcpy/memset 等函数必须由我们自己实现。
//!
//! ## 任务
//!
//! 实现以下五个函数：
//! - 只使用 `core` crate，不使用 `std`
//! - 不要调用 `core::ptr::copy`、`core::ptr::copy_nonoverlapping` 等（自己编写循环）
//! - 正确处理边界情况（n=0、重叠内存区域等）
//! - 通过所有测试

// 在生产环境强制使用 no_std；在测试中允许使用 std（cargo 测试框架需要）
#![cfg_attr(not(test), no_std)]
#![allow(unused_variables)]

/// 从 `src` 复制 `n` 个字节到 `dst`。
///
/// - `dst` 和 `src` 不能重叠（重叠区域请使用 `my_memmove`）
/// - 返回 `dst`
///
/// # 安全性
/// `dst` 和 `src` 必须各自指向至少 `n` 字节的有效内存。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn my_memcpy(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    // TODO: 实现 memcpy
    // 提示：逐字节从 src 读取并写入 dst
    todo!()
}

/// 将从 `dst` 开始的 `n` 个字节设置为值 `c`。
///
/// 返回 `dst`。
///
/// # 安全性
/// `dst` 必须指向至少 `n` 字节的有效可写内存。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn my_memset(dst: *mut u8, c: u8, n: usize) -> *mut u8 {
    // TODO: 实现 memset
    todo!()
}

/// 从 `src` 复制 `n` 个字节到 `dst`，正确处理重叠内存。
///
/// 返回 `dst`。
///
/// # 安全性
/// `dst` 和 `src` 必须各自指向至少 `n` 字节的有效内存。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn my_memmove(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    // TODO: 实现 memmove
    // 提示：当 dst > src 且区域重叠时，从后向前复制（从末尾到开头）
    todo!()
}

/// 返回以 null 结尾的字节字符串的长度，不包括末尾的 null。
///
/// # 安全性
/// `s` 必须指向一个有效的以 null 结尾的字节字符串。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn my_strlen(s: *const u8) -> usize {
    // TODO: 实现 strlen
    todo!()
}

/// 比较两个以 null 结尾的字节字符串。
///
/// 返回值：
/// - `0`  ：字符串相等
/// - `< 0`：`s1` 按字典序小于 `s2`
/// - `> 0`：`s1` 按字典序大于 `s2`
///
/// # 安全性
/// `s1` 和 `s2` 必须各自指向一个有效的以 null 结尾的字节字符串。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn my_strcmp(s1: *const u8, s2: *const u8) -> i32 {
    // TODO: 实现 strcmp
    todo!()
}

// ============================================================
// 测试（std 在 #[cfg(test)] 下可用）
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memcpy_basic() {
        let src = [1u8, 2, 3, 4, 5];
        let mut dst = [0u8; 5];
        unsafe { my_memcpy(dst.as_mut_ptr(), src.as_ptr(), 5) };
        assert_eq!(dst, src);
    }

    #[test]
    fn test_memcpy_zero_len() {
        let src = [0xFFu8; 4];
        let mut dst = [0u8; 4];
        unsafe { my_memcpy(dst.as_mut_ptr(), src.as_ptr(), 0) };
        assert_eq!(dst, [0u8; 4]);
    }

    #[test]
    fn test_memset_basic() {
        let mut buf = [0u8; 8];
        unsafe { my_memset(buf.as_mut_ptr(), 0xAB, 8) };
        assert!(buf.iter().all(|&b| b == 0xAB));
    }

    #[test]
    fn test_memset_partial() {
        let mut buf = [0u8; 8];
        unsafe { my_memset(buf.as_mut_ptr(), 0xFF, 4) };
        assert_eq!(&buf[..4], &[0xFF; 4]);
        assert_eq!(&buf[4..], &[0x00; 4]);
    }

    #[test]
    fn test_memmove_no_overlap() {
        let src = [1u8, 2, 3, 4];
        let mut dst = [0u8; 4];
        unsafe { my_memmove(dst.as_mut_ptr(), src.as_ptr(), 4) };
        assert_eq!(dst, src);
    }

    #[test]
    fn test_memmove_overlap_forward() {
        // 将 buf[0..4] 复制到 buf[1..5]，向右移动 1 位
        let mut buf = [1u8, 2, 3, 4, 5];
        unsafe { my_memmove(buf.as_mut_ptr().add(1), buf.as_ptr(), 4) };
        assert_eq!(buf, [1, 1, 2, 3, 4]);
    }

    #[test]
    fn test_strlen_basic() {
        let s = b"hello\0";
        assert_eq!(unsafe { my_strlen(s.as_ptr()) }, 5);
    }

    #[test]
    fn test_strlen_empty() {
        let s = b"\0";
        assert_eq!(unsafe { my_strlen(s.as_ptr()) }, 0);
    }

    #[test]
    fn test_strcmp_equal() {
        let a = b"hello\0";
        let b = b"hello\0";
        assert_eq!(unsafe { my_strcmp(a.as_ptr(), b.as_ptr()) }, 0);
    }

    #[test]
    fn test_strcmp_less() {
        let a = b"abc\0";
        let b = b"abd\0";
        assert!(unsafe { my_strcmp(a.as_ptr(), b.as_ptr()) } < 0);
    }

    #[test]
    fn test_strcmp_greater() {
        let a = b"abd\0";
        let b = b"abc\0";
        assert!(unsafe { my_strcmp(a.as_ptr(), b.as_ptr()) } > 0);
    }
}
