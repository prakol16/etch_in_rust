#[derive(Debug, Clone)]
pub struct SparseCOOMat<T> {
    /// The data in the sparse vector
    /// Assumes that the indices are sorted in ascending order
    pub rows: Vec<usize>,
    pub cols: Vec<usize>,
    pub vals: Vec<T>,
}




