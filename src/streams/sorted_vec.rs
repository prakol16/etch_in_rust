use super::stream_defs::IndexedStream;

#[derive(Debug, Clone)]
pub struct SortedVecGalloper<'a, T> {
    inds: &'a [T],
    cur: usize,
}

impl<'a, T> SortedVecGalloper<'a, T> {
    pub fn new(inds: &'a [T]) -> Self {
        SortedVecGalloper { inds, cur: 0 }
    }
}

impl<T: Ord + Clone> IndexedStream for SortedVecGalloper<'_, T> {
    type I = T;
    type V = ();

    fn valid(&self) -> bool {
        self.cur < self.inds.len()
    }

    fn ready(&self) -> bool {
        true
    }

    fn seek(&mut self, index: &T, strict: bool) {
        // First, advance to find an upper bound
        let mut step = 1;
        let mut new_cur = self.cur;
        while new_cur < self.inds.len() && (self.inds[new_cur] < *index || (strict && self.inds[new_cur] == *index)) {
            new_cur += step;
            step *= 2;
        }
        // Correct overshoot
        let mut left = self.cur + step / 2;
        let mut right = if new_cur >= self.inds.len() { self.inds.len() } else { new_cur + 1 };

        // Then, do a standard binary search within the found bounds
        while left < right {
            let mid = left + (right - left) / 2;
            if self.inds[mid] < *index || (strict && self.inds[mid] == *index) {
                left = mid + 1;
            } else {
                right = mid;
            }
        }
        self.cur = left;
    }

    fn index(&self) -> T {
        self.inds[self.cur].clone()
    }

    fn value(&self) -> () {}
}