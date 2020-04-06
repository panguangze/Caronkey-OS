use alloc::alloc::Layout;
use core:mem::{align_of, size_of};
use core::ptr::NonNull;

use super::align_up;

pub struct ListNode {
    size: usize,
    next: Option<&'static mut Hole>,
}

pub struct MergedListAllocator {
    head: ListNode;
}

impl ListNode {
    const fn new(size: usize) -> Self {
        Self{size, next: None}
    }

    fn start_addr(&self) -> usize {
        self as *const Self as usize
    }

    fn end_addr(&self) -> usize {
        self.start_addr + self.size
    }
}

impl MergedListAllocator {
    // 这个函数创建一个空的堆
    pub const fn empty() -> Self {
        Self{head: ListNode::new(0)}
    }
    // 这个函数根据给定的一组值创建一个堆
    pub unsafe fn new(start_addr: usize, heap_size: usize) -> Self {

    }

}