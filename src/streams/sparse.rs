use super::{binary_search::binary_search, stream_defs::{IndexedStream, IntoStreamIterator, StreamResult}};



trait SliceLike {
    type Elem;

    // Should have the property that
    // self.slice(a, x).get(i) == self.get(a + i)
    // whenver i < x.
    fn slice(self, start: usize, size: usize) -> Self;

    fn get(&self, index: usize) -> Self::Elem;
}


// A type T implements StackableView
// if Vec<T> can be flattened to just T,
// by keeping track of some metadata (i.e. the boundaries
// between each element). For example, Vec<X> is StackableView
// since Vec<Vec<X>> can be flattened to just Vec<X> by
// keeping track of where each inner vector begins and ends.
trait VecLike<'a>: 'a {
    type View: SliceLike + Clone;

    fn to_view(&'a self) -> Self::View;
}

impl<'a, T> SliceLike for &'a [T] {
    type Elem = &'a T;

    fn slice(self, start: usize, size: usize) -> Self {
        &self[start..start + size]
    }

    fn get(&self, index: usize) -> Self::Elem {
        &self[index]
    }
}

impl<'a, T: Default + 'a> VecLike<'a> for Vec<T> {
    type View = &'a [T];
    
    fn to_view(&'a self) -> Self::View {
        self.as_slice()
    }
}

#[derive(Debug, Clone)]
struct FlatVec<T> {
    // Vector of boundaries, starts with 0.
    // data.slice(boundaries[i], boundaries[i+1] - boundaries[i]) is the ith element
    boundaries: Vec<usize>,
    data: T
}

struct FlatVecView<'a, T: VecLike<'a>> {
    boundaries: &'a [usize],
    data: T::View
}

impl<'a, T: VecLike<'a>> Clone for FlatVecView<'a, T>
        where T::View: Clone {
    fn clone(&self) -> Self {
        FlatVecView {
            boundaries: self.boundaries,
            data: self.data.clone()
        }
    }
}

type DenseVec<T> = FlatVec<T>;

struct DenseIndexIterator<'a, T: VecLike<'a>> {
    view: FlatVecView<'a, T>,
    current_index: usize,
}

impl<'a, T: VecLike<'a>> Clone for DenseIndexIterator<'a, T>
        where T::View: Clone {
    fn clone(&self) -> Self {
        DenseIndexIterator {
            view: self.view.clone(),
            current_index: self.current_index
        }
    }
}

impl<'a, T: VecLike<'a>> SliceLike for FlatVecView<'a, T> {
    type Elem = T::View;

    fn slice(self, start: usize, size: usize) -> Self {
        FlatVecView {
            boundaries: &self.boundaries[start..start + size + 1],
            data: self.data
        }
    }

    fn get(&self, index: usize) -> Self::Elem {
        self.data.clone().slice(self.boundaries[index], self.boundaries[index + 1] - self.boundaries[index])
    }
}

impl<'a, T: VecLike<'a>> VecLike<'a> for FlatVec<T> {
    type View = FlatVecView<'a, T>;

    fn to_view(&'a self) -> Self::View {
        FlatVecView {
            boundaries: &self.boundaries,
            data: self.data.to_view()
        }
    }
}

impl<'a, T> IndexedStream for DenseIndexIterator<'a, T>
        where T: VecLike<'a>,
              T::View: IntoStreamIterator {
    type I = usize;

    type V = <T::View as IntoStreamIterator>::StreamType;

    fn current(&self) -> super::stream_defs::StreamResult<Self::I, Self::V> {
        if self.current_index + 1 < self.view.boundaries.len() {
            StreamResult::Yield {
                index: self.current_index,
                value: Some(self.view.get(self.current_index).into_stream_iterator())
            }
        } else {
            StreamResult::Done
        }
    }

    fn seek(&mut self, index: Self::I, strict: bool) {
        self.current_index = if strict && index == self.current_index {
            index + 1
        } else {
            std::cmp::min(std::cmp::max(self.current_index, index), self.view.boundaries.len() - 1)
        }
    }

    fn next(&mut self, _index: Self::I, _strict: bool) {
        self.current_index += 1;
    }
}

#[derive(Debug, Clone)]
struct WithSparseIndices<I, T> {
    // Always size one less than that of `data`
    inds: Vec<I>,
    data: T,
}

type SparseVec<I, T> = WithSparseIndices<I, FlatVec<T>>;

#[derive(Debug)]
struct SparseIndexView<'a, I, T: VecLike<'a>> {
    inds: &'a [I],
    data: T::View
}

impl<'a, I, T: VecLike<'a>> Clone for SparseIndexView<'a, I, T>
        where T::View: Clone {
    fn clone(&self) -> Self {
        SparseIndexView {
            inds: self.inds,
            data: self.data.clone()
        }
    }
}

impl<'a, I: Copy, T: VecLike<'a>> SliceLike for SparseIndexView<'a, I, T> {
    type Elem = (I, <T::View as SliceLike>::Elem);

    fn slice(self, start: usize, size: usize) -> Self {
        SparseIndexView {
            inds: &self.inds[start..start + size],
            data: self.data
        }
    }

    fn get(&self, index: usize) -> Self::Elem {
        (self.inds[index], self.data.get(index))
    }
}

impl<'a, I: 'a + Copy, T: VecLike<'a>> VecLike<'a> for WithSparseIndices<I, T> {
    type View = SparseIndexView<'a, I, T>;

    fn to_view(&'a self) -> Self::View {
        SparseIndexView {
            inds: &self.inds,
            data: self.data.to_view()
        }
    }
}

struct SparseIndexIterator<'a, I, T: VecLike<'a>> {
    view: SparseIndexView<'a, I, T>,
    current_index: usize
}

impl<'a, I: 'a,T: VecLike<'a>> Clone for SparseIndexIterator<'a, I, T> {
    fn clone(&self) -> Self {
        SparseIndexIterator {
            view: self.view.clone(),
            current_index: self.current_index
        }
    }
}

impl<'a, I, T> IndexedStream for SparseIndexIterator<'a, I, T>
        where T: VecLike<'a>,
              <T::View as SliceLike>::Elem: IntoStreamIterator,
              I: Copy + Ord {
    type I = I;

    type V = <<T::View as SliceLike>::Elem as IntoStreamIterator>::StreamType;

    fn current(&self) -> super::stream_defs::StreamResult<Self::I, Self::V> {
        if self.current_index < self.view.inds.len() {
            let (index, value) = self.view.get(self.current_index);
            StreamResult::Yield {
                index,
                value: Some(value.into_stream_iterator())
            }
        } else {
            StreamResult::Done
        }
    }
    
    fn seek(&mut self, index: Self::I, strict: bool) {
        self.current_index += binary_search(&self.view.inds[self.current_index..], &index, strict);
    }

    fn next(&mut self, _index: Self::I, _strict: bool) {
        self.current_index += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::WithSparseIndices;
    use crate::streams::stream_defs::IntoStreamIterator;

    fn test_basic() {
    }
}
