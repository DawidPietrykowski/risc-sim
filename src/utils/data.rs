use crate::types::Instruction;

pub struct CircularBuffer<T> {
    buffer: Vec<T>,
    head: usize,
    tail: usize,
    pub size: usize,
}

impl<T> CircularBuffer<T>
where
    T: Default,
    T: Clone,
{
    pub fn new(size: usize) -> Self {
        CircularBuffer {
            buffer: vec![Default::default(); size],
            head: 0,
            tail: 0,
            size,
        }
    }

    #[allow(unused)]
    pub fn push(&mut self, item: T) {
        self.buffer[self.head] = item;
        self.head = (self.head + 1) % self.size;
        if self.head == self.tail {
            self.tail = (self.tail + 1) % self.size;
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.tail != self.head {
            let item = self.buffer[self.tail].clone();
            self.tail = (self.tail + 1) % self.size;
            Some(item)
        } else {
            None
        }
    }
}

pub fn print_pc_history(buffer: &mut CircularBuffer<(u64, Option<Instruction>, u64)>) {
    if buffer.size == 0 {
        return;
    }
    println!("pc history:");
    let mut last_pc = 0u64;
    while let Some((pc, ins, satp)) = buffer.pop() {
        if pc != last_pc + 0x4 {
            println!("jmp");
        }
        println!("{:x} {} {:x}", pc, ins.unwrap().name, satp);
        last_pc = pc;
    }
    println!();
}
