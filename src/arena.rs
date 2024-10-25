pub struct Arena<T> {
    inner: Vec<Option<T>>,
    last_remove: Option<usize>,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Arena {
            inner: Vec::new(),
            last_remove: None,
        }
    }
    pub fn insert(&mut self, e: T) -> usize {
        if let Some(vacancy) = self.last_remove {
            self.inner[vacancy] = Some(e);
            self.last_remove = None;
            vacancy
        } else {
            for i in 0..self.inner.len() {
                if self.inner[i].is_none() {
                    self.inner[i] = Some(e);
                    return i;
                }
            }
            let last_index = self.inner.len();
            self.inner.push(Some(e));
            last_index
        }
    }
    pub fn remove(&mut self, i: usize) -> Option<T> {
        if i >= self.inner.len() {
            return None;
        }
        self.inner[i].take()
    }
}
