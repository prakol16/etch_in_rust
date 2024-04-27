use super::{sparse_vec::SparseVecGalloper, stream_defs::{FromStreamIterator, IndexedStream, IntoStreamIterator}};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SparseCSRMat<T> {
    /// The data in the sparse vector
    /// Assumes that the indices are sorted in ascending order
    rows: Vec<usize>, // size is #rows + 1; rows[i+1] - rows[i] is the number of non-zero elements in row i
    cols: Vec<usize>, // size is the number of non-zero elements in the matrix, the column index of each non-zero element
    vals: Vec<T>,
}

impl<T> SparseCSRMat<T> {
    pub fn rows(&self) -> usize {
        self.rows.len() - 1
    }

    pub fn empty() -> Self {
        SparseCSRMat {
            rows: vec![0],
            cols: Vec::new(),
            vals: Vec::new(),
        }
    }

    pub fn with_capacity(rows: usize, capacity: usize) -> Self {
        let mut rows = Vec::with_capacity(rows + 1);
        rows.push(0);
        SparseCSRMat {
            rows,
            cols: Vec::with_capacity(capacity),
            vals: Vec::with_capacity(capacity),
        }
    }
    
}

impl<T> FromIterator<(usize, usize, T)> for SparseCSRMat<T> {
    fn from_iter<V: IntoIterator<Item = (usize, usize, T)>>(iter: V) -> Self {
        let mut rows = Vec::new();
        let mut cols = Vec::new();
        let mut vals = Vec::new();
        let mut row_counts = 0;

        for (row, col, val) in iter {
            while rows.len() <= row {
                rows.push(row_counts);
            }
            cols.push(col);
            vals.push(val);
            row_counts += 1;
        }
        rows.push(row_counts);

        SparseCSRMat { rows, cols, vals }
    }
}

#[derive(Debug, Clone)]
pub struct SparseCSRMatIterator<'a, T> {
    rows: &'a [usize],
    cols: &'a [usize],
    vals: &'a [T],
    cur: usize
}

impl<'a, T> IndexedStream for SparseCSRMatIterator<'a, T> {
    type I = usize;
    type V = SparseVecGalloper<'a, usize, T>;

    fn valid(&self) -> bool {
        self.cur < self.rows.len() - 1
    }

    fn ready(&self) -> bool {
        true
    }

    fn seek(&mut self, index: Self::I, strict: bool) {
        self.cur = if strict && index == self.cur {
            index + 1
        } else {
            std::cmp::min(std::cmp::max(self.cur, index), self.rows.len() - 1)
        }
    }

    fn next(&mut self) {
        self.cur += 1;
    }

    fn index(&self) -> Self::I {
        self.cur
    }

    fn value(&self) -> Self::V {
        let start = self.rows[self.cur];
        let end = self.rows[self.cur + 1];
        SparseVecGalloper::new(&self.cols[start..end], &self.vals[start..end])
    }
}

impl<'a, T> IntoStreamIterator for &'a SparseCSRMat<T> {
    type IndexType = usize;
    type ValueType = SparseVecGalloper<'a, usize, T>;
    type StreamType = SparseCSRMatIterator<'a, T>;

    fn into_stream_iterator(self) -> Self::StreamType {
        SparseCSRMatIterator {
            rows: &self.rows,
            cols: &self.cols,
            vals: &self.vals,
            cur: 0,
        }
    }
}

impl<'a, T, S1: IndexedStream<I = usize, V = T>> FromStreamIterator<usize, S1> for SparseCSRMat<T>
{
    fn from_stream_iterator<S: IndexedStream<I=usize, V=S1>>(iter: S) -> Self {
        let mut result = SparseCSRMat::empty();
        result.extend_from_stream_iterator(iter);
        result
    }

    fn extend_from_stream_iterator<S: IndexedStream<I=usize, V=S1>>(&mut self, iter: S) {
        let mut row_counts = *self.rows.last().expect("SparseCSR in invalid state: `rows` is empty");
        iter.for_each(|i, v| {
            while self.rows.len() <= i {
                self.rows.push(row_counts);
            }
            v.for_each(|j, x| {
                row_counts += 1;
                self.cols.push(j);
                self.vals.push(x);
            });
            self.rows.push(row_counts);
        });
    }
}
