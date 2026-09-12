pub struct GcContext<'a> {
    pub bytes_allocated: &'a mut usize,
    pub next_gc: &'a mut usize,
    pub stress_gc: bool,
}

impl GcContext<'_> {
    pub fn charge_allocation(&mut self, old_size: usize, new_size: usize) {
        if new_size >= old_size {
            *self.bytes_allocated += new_size - old_size;
        } else {
            *self.bytes_allocated -= old_size - new_size;
        }
    }

    pub fn should_collect_now(&self) -> bool {
        self.stress_gc || *self.bytes_allocated > *self.next_gc
    }
}

pub trait GcCollector {
    fn gc_context(&mut self) -> GcContext<'_>;
    fn collect_garbage(&mut self);
}
