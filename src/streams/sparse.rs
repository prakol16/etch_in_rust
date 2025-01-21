use super::{binary_search::binary_search, stream_defs::{IndexedStream, IntoStreamIterator, StreamResult}};



trait Sliceable {
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
trait StackableView<'a>: 'a {
    type View: Sliceable + Clone;

    fn to_view(&'a self) -> Self::View;
}

impl<'a, T> Sliceable for &'a [T] {
    type Elem = &'a T;

    fn slice(self, start: usize, size: usize) -> Self {
        &self[start..start + size]
    }

    fn get(&self, index: usize) -> Self::Elem {
        &self[index]
    }
}

impl<'a, T: Default + 'a> StackableView<'a> for Vec<T> {
    type View = &'a [T];
    
    fn to_view(&'a self) -> Self::View {
        self.as_slice()
    }
}

#[derive(Debug, Clone)]
struct DenseIndex<T> {
    // Vector of boundaries, starts with 0.
    // data.slice(boundaries[i], boundaries[i+1] - boundaries[i]) is the ith element
    boundaries: Vec<usize>,
    data: T
}

struct DenseIndexView<'a, T: StackableView<'a>> {
    boundaries: &'a [usize],
    data: T::View
}

impl<'a, T: StackableView<'a>> Clone for DenseIndexView<'a, T>
        where T::View: Clone {
    fn clone(&self) -> Self {
        DenseIndexView {
            boundaries: self.boundaries,
            data: self.data.clone()
        }
    }
}

struct DenseIndexIterator<'a, T: StackableView<'a>> {
    view: DenseIndexView<'a, T>,
    current_index: usize,
}

impl<'a, T: StackableView<'a>> Clone for DenseIndexIterator<'a, T>
        where T::View: Clone {
    fn clone(&self) -> Self {
        DenseIndexIterator {
            view: self.view.clone(),
            current_index: self.current_index
        }
    }
}

impl<'a, T: StackableView<'a>> Sliceable for DenseIndexView<'a, T> {
    type Elem = T::View;

    fn slice(self, start: usize, size: usize) -> Self {
        DenseIndexView {
            boundaries: &self.boundaries[start..start + size + 1],
            data: self.data
        }
    }

    fn get(&self, index: usize) -> Self::Elem {
        self.data.clone().slice(self.boundaries[index], self.boundaries[index + 1] - self.boundaries[index])
    }
}

impl<'a, T: StackableView<'a>> StackableView<'a> for DenseIndex<T> {
    type View = DenseIndexView<'a, T>;

    fn to_view(&'a self) -> Self::View {
        DenseIndexView {
            boundaries: &self.boundaries,
            data: self.data.to_view()
        }
    }
}

impl<'a, T> IndexedStream for DenseIndexIterator<'a, T>
        where T: StackableView<'a>,
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
struct SparseIndex<T> {
    boundaries: Vec<usize>,
    // Always size one less than that of `boundaries`
    inds: Vec<usize>,
    // The data associated with index at inds[i]
    // is data.slice(boundaries[i], boundaries[i+1] - boundaries[i])
    data: T,
}

#[derive(Debug)]
struct SparseIndexView<'a, T: StackableView<'a>> {
    boundaries: &'a [usize],
    inds: &'a [usize],
    data: T::View
}

impl<'a, T: StackableView<'a>> Clone for SparseIndexView<'a, T>
        where T::View: Clone {
    fn clone(&self) -> Self {
        SparseIndexView {
            boundaries: self.boundaries,
            inds: self.inds,
            data: self.data.clone()
        }
    }
}

impl<'a, T: StackableView<'a>> Sliceable for SparseIndexView<'a, T> {
    type Elem = T::View;

    fn slice(self, start: usize, size: usize) -> Self {
        SparseIndexView {
            boundaries: &self.boundaries[start..start + size + 1],
            inds: &self.inds[start..start + size],
            data: self.data
        }
    }

    fn get(&self, index: usize) -> Self::Elem {
        self.data.clone().slice(self.boundaries[self.inds[index]], 
            self.boundaries[self.inds[index] + 1] - self.boundaries[self.inds[index]])
    }
}

impl<'a, T: StackableView<'a>> StackableView<'a> for SparseIndex<T> {
    type View = SparseIndexView<'a, T>;

    fn to_view(&'a self) -> Self::View {
        SparseIndexView {
            boundaries: &self.boundaries,
            inds: &self.inds,
            data: self.data.to_view()
        }
    }
}

struct SparseIndexIterator<'a, T: StackableView<'a>> {
    view: SparseIndexView<'a, T>,
    current_index: usize
}

impl<'a, T: StackableView<'a>> Clone for SparseIndexIterator<'a, T> {
    fn clone(&self) -> Self {
        SparseIndexIterator {
            view: self.view.clone(),
            current_index: self.current_index
        }
    }
}

impl<'a, T> IndexedStream for SparseIndexIterator<'a, T>
        where T: StackableView<'a>,
              T::View: IntoStreamIterator {
    type I = usize;

    type V = <T::View as IntoStreamIterator>::StreamType;

    fn current(&self) -> super::stream_defs::StreamResult<Self::I, Self::V> {
        if self.current_index < self.view.inds.len() {
            StreamResult::Yield {
                index: self.current_index,
                value: Some(self.view.get(self.current_index).into_stream_iterator())
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
