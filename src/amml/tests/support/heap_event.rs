// one thing FakeHeap saw: a block handed out (address, bytes), a block freed, or a destructor a fake ran on an
// object
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeapEvent {
    Allocate(usize, usize),
    Free(usize),
    Destroy(usize),
}
