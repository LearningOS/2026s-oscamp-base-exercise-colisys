//! # 页表项标志位
//!
//! 在本练习中，你将学习 RISC-V SV39 页表项（PTE）的结构，
//! 并通过位操作构造/解析页表项。
//!
//! ## 概念
//! - RISC-V SV39 页表项 64 位布局
//! - 构造/提取字段的位操作
//! - PTE 标志位的含义（V/R/W/X/U/G/A/D）
//!
//! ## SV39 PTE 布局（64 位）
//! ```text
//! 63    54 53        10 9  8 7 6 5 4 3 2 1 0
//! ┌───────┬────────────┬────┬─┬─┬─┬─┬─┬─┬─┬─┐
//! │ 保留  │  PPN[2:0]  │ RSW│D│A│G│U│X│W│R│V│
//! │ 10位  │  44 位     │ 2位│ │ │ │ │ │ │ │ │
//! └───────┴────────────┴────┴─┴─┴─┴─┴─┴─┴─┴─┘
//! ```
//! - V（Valid）：有效位，指示页表项是否有效。
//!
//! - R/W/X（Read/Write/Execute）：读、写、执行权限位。
//!
//! - U（User）：用户可访问位，允许用户态访问。
//!
//! - G（Global）：全局映射位（通常用于内核空间，避免 TLB 刷新）。
//!
//! - A（Accessed）：访问位，页面被访问时由硬件设置。
//!
//! - D（Dirty）：脏位，页面被写入时由硬件设置。
//!
//! - RSW（Reserved for Supervisor Software）：保留给操作系统软件使用的两位。
//!
//! - PPN（Physical Page Number）：物理页号，占用 44 位（bits [53:10]），指定物理页帧的基地址。
//! - PPN[2:0]（Physical Page Number）：在 RISC-V SV39 分页机制中，物理页号（PPN）分为三部分，分别称为 PPN[2]、PPN[1] 和 PPN[0]。这种划分是为了支持多级页表的索引。
//! - Rsvd（Reserved）：保留位，通常设置为 0。

/// PTE 标志位常量
pub const PTE_V: u64 = 1 << 0; // 有效
pub const PTE_R: u64 = 1 << 1; // 可读
pub const PTE_W: u64 = 1 << 2; // 可写
pub const PTE_X: u64 = 1 << 3; // 可执行
pub const PTE_U: u64 = 1 << 4; // 用户可访问
pub const PTE_G: u64 = 1 << 5; // 全局
pub const PTE_A: u64 = 1 << 6; // 已访问
pub const PTE_D: u64 = 1 << 7; // 脏

/// PTE 中 PPN 字段的偏移和掩码
const PPN_SHIFT: u32 = 10;
const PPN_MASK: u64 = (1u64 << 44) - 1; // 44 位 PPN

/// 从物理页号（PPN）和标志位构造页表项。
///
/// PPN 占据 bits [53:10]，标志位占据 bits [7:0]。
///
/// 示例：ppn=0x12345, flags=PTE_V|PTE_R|PTE_W
/// 结果应为：(0x12345 << 10) | 0b111 = 0x48D14007
///
/// 提示：将 PPN 左移 PPN_SHIFT 位，然后与标志位 OR。
pub fn make_pte(ppn: u64, flags: u64) -> u64 {
    // TODO: 使用 ppn 和 flags 构造页表项
    // todo!();
    (ppn << PPN_SHIFT) | (flags & ((1 << 8) - 1))
}

/// 从页表项提取物理页号（PPN）。
///
/// 提示：右移 PPN_SHIFT 位，然后与 PPN_MASK 做 AND。
pub fn extract_ppn(pte: u64) -> u64 {
    // TODO: 从 pte 提取 PPN
    // todo!();
    (pte >> PPN_SHIFT) & PPN_MASK
}

/// 从页表项提取标志位（低 8 位）。
pub fn extract_flags(pte: u64) -> u64 {
    // TODO: 提取低 8 位标志
    pte & ((1 << 8) - 1)
}

/// 检查页表项是否有效（V 位是否设置）。
pub fn is_valid(pte: u64) -> bool {
    // TODO: 检查 PTE_V
    (pte & PTE_V) > 0
}

/// 判断页表项是否为叶子 PTE。
///
/// 在 SV39 中，如果 R、W、X 位中任意一位被设置，则 PTE 是叶子，
/// 指向最终的物理页。否则它指向下一级页表。
pub fn is_leaf(pte: u64) -> bool {
    // TODO: 检查 R/W/X 位是否有任意位被设置
    // todo!()
    (pte & (PTE_R | PTE_W | PTE_X)) > 0
}

/// 根据给定的权限检查页表项是否允许请求的访问。
///
/// - `read`：需要读权限
/// - `write`：需要写权限
/// - `execute`：需要执行权限
///
/// 当且仅当：PTE 有效，且每个请求的权限都满足时返回 true。
pub fn check_permission(pte: u64, read: bool, write: bool, execute: bool) -> bool {
    // TODO: 首先检查是否有效，然后检查每个请求的权限
    // todo!()
    if !is_valid(pte) {
        return false;
    }

    if read && ((pte & PTE_R) == 0) {
        return false;
    }

    if write && ((pte & PTE_W) == 0) {
        return false;
    }

    if execute && ((pte & PTE_X) == 0) {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_pte_basic() {
        let pte = make_pte(0x12345, PTE_V | PTE_R | PTE_W);
        assert_eq!(extract_ppn(pte), 0x12345);
        assert_eq!(extract_flags(pte), PTE_V | PTE_R | PTE_W);
    }

    #[test]
    fn test_make_pte_zero() {
        let pte = make_pte(0, 0);
        assert_eq!(pte, 0);
        assert_eq!(extract_ppn(pte), 0);
        assert_eq!(extract_flags(pte), 0);
    }

    #[test]
    fn test_make_pte_all_flags() {
        let all = PTE_V | PTE_R | PTE_W | PTE_X | PTE_U | PTE_G | PTE_A | PTE_D;
        let pte = make_pte(0xABC, all);
        assert_eq!(extract_ppn(pte), 0xABC);
        assert_eq!(extract_flags(pte), all);
    }

    #[test]
    fn test_make_pte_large_ppn() {
        let ppn = (1u64 << 44) - 1; // 最大 PPN
        let pte = make_pte(ppn, PTE_V);
        assert_eq!(extract_ppn(pte), ppn);
    }

    #[test]
    fn test_is_valid() {
        assert!(is_valid(make_pte(1, PTE_V)));
        assert!(!is_valid(make_pte(1, PTE_R))); // R 设置但 V 未设置
        assert!(!is_valid(0));
    }

    #[test]
    fn test_is_leaf() {
        assert!(is_leaf(make_pte(1, PTE_V | PTE_R)));
        assert!(is_leaf(make_pte(1, PTE_V | PTE_X)));
        assert!(is_leaf(make_pte(1, PTE_V | PTE_R | PTE_W | PTE_X)));
        // 非叶子：仅 V 设置，R/W/X 都清零
        assert!(!is_leaf(make_pte(1, PTE_V)));
        assert!(!is_leaf(make_pte(1, PTE_V | PTE_A | PTE_D)));
    }

    #[test]
    fn test_check_permission_read() {
        let pte = make_pte(1, PTE_V | PTE_R);
        assert!(check_permission(pte, true, false, false));
        assert!(!check_permission(pte, false, true, false));
        assert!(!check_permission(pte, false, false, true));
    }

    #[test]
    fn test_check_permission_rw() {
        let pte = make_pte(1, PTE_V | PTE_R | PTE_W);
        assert!(check_permission(pte, true, true, false));
        assert!(!check_permission(pte, true, true, true));
    }

    #[test]
    fn test_check_permission_all() {
        let pte = make_pte(1, PTE_V | PTE_R | PTE_W | PTE_X);
        assert!(check_permission(pte, true, true, true));
        assert!(check_permission(pte, true, false, false));
        assert!(check_permission(pte, false, false, false)); // 无要求 = OK
    }

    #[test]
    fn test_check_permission_invalid() {
        // V 未设置，即使 R/W/X 标志存在也应返回 false
        let pte = make_pte(1, PTE_R | PTE_W | PTE_X);
        assert!(!check_permission(pte, true, false, false));
    }
}
