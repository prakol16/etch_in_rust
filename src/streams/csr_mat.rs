use super::{sparse_vec::SparseVecIterator, stream_defs::{IntoStreamIterator, StreamIterator}};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SparseCSRMat<T> {
    /// The data in the sparse vector
    /// Assumes that the indices are sorted in ascending order
    pub rows: Vec<usize>, // size is #rows + 1; rows[i+1] - rows[i] is the number of non-zero elements in row i
    pub cols: Vec<usize>, // size is the number of non-zero elements in the matrix, the column index of each non-zero element
    pub vals: Vec<T>,
}

impl<T> FromIterator<(usize, usize, T)> for SparseCSRMat<T> {
    fn from_iter<V: IntoIterator<Item = (usize, usize, T)>>(iter: V) -> Self {
        let mut rows = Vec::new();
        let mut cols = Vec::new();
        let mut vals = Vec::new();
        let mut current_row: Option<usize> = None;
        let mut row_counts = 0;

        for (row, col, val) in iter {
            if Some(row) != current_row {
                rows.push(row_counts);
                current_row = Some(row);
            }
            cols.push(col);
            vals.push(val);
            row_counts += 1;
        }
        rows.push(row_counts); // Add the final count to rows

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

impl<'a, T: Clone> StreamIterator for SparseCSRMatIterator<'a, T> {
    type I = usize;
    type V = SparseVecIterator<'a, T>;

    fn valid(&self) -> bool {
        self.cur < self.rows.len() - 1
    }

    fn ready(&self) -> bool {
        true
    }

    fn skip(&mut self, index: &Self::I, strict: bool) {
        self.cur = if strict && *index == self.cur {
            *index + 1
        } else {
            std::cmp::min(std::cmp::max(self.cur, *index), self.rows.len() - 1)
        }
    }

    fn index(&self) -> Self::I {
        self.cur
    }

    fn value(&self) -> Self::V {
        let start = self.rows[self.cur];
        let end = self.rows[self.cur + 1];
        SparseVecIterator::new(&self.cols[start..end], &self.vals[start..end])
    }
}

impl<'a, T: Clone> IntoStreamIterator for &'a SparseCSRMat<T> {
    type IndexType = usize;
    type ValueType = SparseVecIterator<'a, T>;
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

