use core::mem;
use super::align_up;
use super::Locked;
use alloc::alloc::{GlobalAlloc,Layout};
use core::ptr;

struct ListNode {
    size: usize,
    next: Option<&'static mut ListNode>,
}

impl ListNode {
    const fn new(size: usize) -> Self {
        ListNode {size, next: None}
    }

    fn start_addr(&self) -> usize {
        self as *const Self as usize
    }

    fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }
}

pub struct LinkedListAllocator {
    head: ListNode,
}

impl LinkedListAllocator {
    pub const fn new() -> Self {
        Self {
            head: ListNode::new(0),
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, head_size: usize) {
        self.add_free_region(heap_start, head_size);
    }
    //把给定的内存空间放到list的头部
    unsafe fn add_free_region(&mut self, addr: usize, size: usize) {
        // 确认空闲的区域能容纳ListNode，我理解的就是需要有一致的对齐方式
        assert!(align_up(addr, mem::align_of::<ListNode>()) == addr);
        assert!(size >= mem::size_of::<ListNode>());

        // 创建一个新的node 放在list的前面
        let mut node =ListNode::new(size);
        node.next = self.head.next.take();
        let node_ptr = addr as *mut ListNode;
        node_ptr.write(node);
        self.head.next = Some(&mut *node_ptr);
    }

    // 查找能存下给定大小空间的ListNode
    fn find_region(&mut self, size: usize, align: usize) 
        -> Option<(&'static mut ListNode,usize)>
    {
        let mut current = &mut self.head;

        while let Some(ref mut region) = current.next {
            if let Ok(alloc_start) = Self::alloc_from_region(&region, size, align) {
                let next = region.next.take();
                let ret = Some((current.next.take().unwrap(),alloc_start));
                current.next = next;
                return ret;
            } else {
            current = current.next.as_mut().unwrap();
            }
        }
        None
    }

    // 返回从ListNode中分配的内存首地址。
    fn alloc_from_region(region: &ListNode, size: usize, align: usize)
        -> Result<usize,()>
    {
        let alloc_start = align_up(region.start_addr(), align);
        let alloc_end = alloc_start.checked_add(size).ok_or(())?;

        if alloc_end > region.end_addr() {
            return Err(());
        }

        let excess_size = region.end_addr() - alloc_end;
        if excess_size > 0 && excess_size < mem::size_of::<ListNode>() {
            return Err(());
        }

        Ok(alloc_start)
    }

    // 对齐
    fn size_align(layout: Layout)->(usize, usize) {
        let layout = layout
            .align_to(mem::align_of::<ListNode>())
            .expect("adjusting alignment failed")
            .pad_to_align();
        let size = layout.size().max(mem::size_of::<ListNode>());
        (size, layout.align())
    }
}

unsafe impl GlobalAlloc for Locked<LinkedListAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let(size, align) = LinkedListAllocator::size_align(layout);
        let mut allocator = self.lock();

        if let Some((region, alloc_start)) = allocator.find_region(size, align) {
            let alloc_end = alloc_start.checked_add(size).expect("overflow");
            let excess_size = region.end_addr() - alloc_end;
            if excess_size > 0 {
                allocator.add_free_region(alloc_end, excess_size)
            }
            alloc_start as *mut u8
        } else {
            ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let (size,_) = LinkedListAllocator::size_align(layout);
        self.lock().add_free_region(ptr as usize, size)
    }
}