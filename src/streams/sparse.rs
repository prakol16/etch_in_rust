use super::{binary_search::binary_search, stream_defs::{IndexedStream, IntoStreamIterator, StreamResult}};



trait SliceLike {
    type Elem;

    // Should have the property that
    // self.slice(a, x).get(i) == self.get(a + i)
    // whenver i < x.
    fn slice(self, start: usize, size: usize) -> Self;

    fn get(&self, index: usize) -> Self::Elem;

    fn size(&self) -> usize;
}


// A type T implements VecLike
// if we can view it as a SliceLike object
// that has the same lifetime as the object.
trait VecLike<'a>: 'a {
    type OwnedElem;
    type View: SliceLike + Clone;

    fn empty() -> Self;

    fn to_view(&'a self) -> Self::View;

    fn push(&'a mut self, elem: Self::OwnedElem);

    fn extend_self(&'a mut self, elems: Self);
}

impl<'a, T> SliceLike for &'a [T] {
    type Elem = &'a T;

    fn slice(self, start: usize, size: usize) -> Self {
        &self[start..start + size]
    }

    fn get(&self, index: usize) -> Self::Elem {
        &self[index]
    }

    fn size(&self) -> usize {
        self.len()
    }
}

impl<'a, T: 'a> VecLike<'a> for Vec<T> {
    type OwnedElem = T;
    type View = &'a [T];
    
    fn empty() -> Self {
        Vec::new()
    }

    fn to_view(&'a self) -> Self::View {
        self.as_slice()
    }

    fn push(&mut self, elem: Self::OwnedElem) {
        self.push(elem);
    }

    fn extend_self(&mut self, elems: Self) {
        self.extend(elems.into_iter());
    }
}

// A FlatVec<T>, where T: VecLike, represents
// a Vec<T> ~ Vec<Vec<T::OwnedElem>> (since T represents a Vec<T::OwnedElem>) in a flattened way.
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

    fn size(&self) -> usize {
        self.boundaries.len() - 1
    }
}

impl<'a, T: for<'b> VecLike<'b>> VecLike<'a> for FlatVec<T> {
    type OwnedElem = T;
    type View = FlatVecView<'a, T>;

    fn empty() -> Self {
        FlatVec {
            boundaries: vec![0],
            data: T::empty()
        }
    }

    fn to_view(&'a self) -> Self::View {
        FlatVecView {
            boundaries: &self.boundaries,
            data: self.data.to_view()
        }
    }

    fn push(&'a mut self, elem: Self::OwnedElem) {
        let size = self.data.to_view().size();
        self.boundaries.push(size);
        self.data.extend_self(elem);
    }

    fn extend_self(&'a mut self, elems: Self) {
        let size = self.data.to_view().size();
        self.boundaries.extend(
            elems.boundaries[1..]
            .iter().map(|&i| i + size));
        self.data.extend_self(elems.data);
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
    // Always same size as `data`
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
            data: self.data.slice(start, size)
        }
    }

    fn get(&self, index: usize) -> Self::Elem {
        (self.inds[index], self.data.get(index))
    }

    fn size(&self) -> usize {
        self.inds.len()
    }
}

impl<'a, I: 'a + Copy, T: VecLike<'a>> VecLike<'a> for WithSparseIndices<I, T> {
    type OwnedElem = (I, T::OwnedElem);
    type View = SparseIndexView<'a, I, T>;

    fn empty() -> Self {
        WithSparseIndices {
            inds: Vec::new(),
            data: T::empty()
        }
    }

    fn to_view(&'a self) -> Self::View {
        SparseIndexView {
            inds: &self.inds,
            data: self.data.to_view()
        }
    }

    fn push(&'a mut self, (index, elem): Self::OwnedElem) {
        self.inds.push(index);
        self.data.push(elem);
    }

    fn extend_self(&'a mut self, elems: Self) {
        self.inds.extend(elems.inds);
        self.data.extend_self(elems.data);
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
