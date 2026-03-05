#[derive(Debug, Clone)]
pub struct LiFo<T, const N: usize> {
    stack: [Option<T>; N],
    last_index: usize,
}

impl<T, const N: usize> LiFo<T, N> {
    pub fn new() -> Self {
        Self {
            stack: [const { None }; N],
            last_index: N - 1,
        }
    }

    pub fn push(&mut self, item: T) {
        self.last_index = (self.last_index + 1) % N;
        self.stack[self.last_index] = Some(item);
    }

    pub fn pop(&mut self) -> Option<T> {
        let item = self.stack[self.last_index].take();
        if item.is_some() {
            self.last_index = (self.last_index + N - 1) % N;
        }
        item
    }

    pub fn can_pop(&self) -> bool {
        self.stack[self.last_index].is_some()
    }
}
