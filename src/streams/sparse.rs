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


trait MutVecLike {
    type OwnedElem;
    type MutElem;

    fn emplace_back(&mut self, construct: impl FnOnce(&mut Self::MutElem) -> Self::OwnedElem);

    fn size(&self) -> usize;
}

// A type T implements VecLike
// if we can view it as a SliceLike object
// that has the same lifetime as the object.
trait VecLike<'a>: 'a {
    type View: SliceLike + Clone;
    type MutView: MutVecLike;

    fn empty() -> Self;

    fn to_view(&'a self) -> Self::View;

    fn to_mut_view(&'a mut self) -> Self::MutView;
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

impl<'a, T> MutVecLike for &'a mut Vec<T> {
    type OwnedElem = T;
    type MutElem = ();
    
    fn emplace_back(&mut self, construct: impl FnOnce(&mut Self::MutElem) -> Self::OwnedElem) {
        self.push(construct(&mut ()));
    }

    fn size(&self) -> usize {
        self.len()
    }
}

impl<'a, T: 'a> VecLike<'a> for Vec<T> {
    type View = &'a [T];
    type MutView = &'a mut Vec<T>;
    
    fn empty() -> Self {
        Vec::new()
    }

    fn to_view(&'a self) -> Self::View {
        self.as_slice()
    }

    fn to_mut_view(&'a mut self) -> Self::MutView {
        self
    }
}

// A FlatVec<T>, where T: VecLike, represents
// a Vec<T> ~ Vec<Vec<T::Elem>> (since T represents a Vec<T::OwnedElem>) in a flattened way.
#[derive(Debug, Clone)]
struct FlatVec<T> {
    // Vector of boundaries, starts with 0.
    // data.slice(boundaries[i], boundaries[i+1] - boundaries[i]) is the ith element
    boundaries: Vec<usize>,
    data: T
}

#[derive(Debug)]
struct FlatVecView<'a, T: VecLike<'a>> {
    boundaries: &'a [usize],
    data: T::View
}

struct FlatVecMutView<'a, T: VecLike<'a>> {
    boundaries: &'a mut Vec<usize>,
    data: T::MutView
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

impl<'a, T: VecLike<'a>> MutVecLike for FlatVecMutView<'a, T> {
    type OwnedElem = ();
    type MutElem = T::MutView;
    
    fn emplace_back(&mut self, construct: impl FnOnce(&mut Self::MutElem)) {
        construct(&mut self.data);
        self.boundaries.push(self.data.size());
    }

    fn size(&self) -> usize {
        self.boundaries.len() - 1
    }
}

impl<'a, T: for<'b> VecLike<'b>> VecLike<'a> for FlatVec<T> {
    type MutView = FlatVecMutView<'a, T>;
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
    
    fn to_mut_view(&'a mut self) -> Self::MutView {
        FlatVecMutView {
            boundaries: &mut self.boundaries,
            data: self.data.to_mut_view()
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

struct SparseIndexMutView<'a, I, T: VecLike<'a>> {
    inds: &'a mut Vec<I>,
    data: T::MutView
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

impl<'a, I: Copy, T: VecLike<'a>> MutVecLike for SparseIndexMutView<'a, I, T> {
    type OwnedElem = (I, <T::MutView as MutVecLike>::OwnedElem);
    type MutElem = <T::MutView as MutVecLike>::MutElem;
    
    fn emplace_back(&mut self, construct: impl FnOnce(&mut Self::MutElem) -> Self::OwnedElem) {
        self.data.emplace_back(|v| {
            let (index, x) = construct(v);
            self.inds.push(index);
            x
        });
    }
    
    fn size(&self) -> usize {
        self.inds.len()
    }
}

impl<'a, I: 'a + Copy, T: VecLike<'a>> VecLike<'a> for WithSparseIndices<I, T> {
    type MutView = SparseIndexMutView<'a, I, T>;
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
    
    fn to_mut_view(&'a mut self) -> Self::MutView {
        SparseIndexMutView {
            inds: &mut self.inds,
            data: self.data.to_mut_view()
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
    use std::vec;

    use super::{DenseVec, FlatVec, MutVecLike, SparseVec, VecLike, WithSparseIndices};
    use crate::streams::{sparse::SliceLike, stream_defs::IntoStreamIterator};

    #[test]
    fn test_basic() {
        let mut vec_2d: FlatVec<Vec<i32>> = FlatVec::empty();
        vec_2d.to_mut_view().emplace_back(|v| {
            v.push(3);
            v.push(4);
            v.push(10);
        });
        vec_2d.to_mut_view().emplace_back(|v| {
            v.push(5);
        });
        vec_2d.to_mut_view().emplace_back(|v| {
            v.push(7);
            v.push(8);
        });
        assert_eq!(vec_2d.to_view().size(), 3);
        assert_eq!(vec_2d.to_view().get(0), &[3, 4, 10]);
        assert_eq!(vec_2d.to_view().get(1), &[5]);
        assert_eq!(vec_2d.to_view().get(2), &[7, 8]);
        assert_eq!(vec_2d.to_mut_view().size(), 3);

        let slice = vec_2d.to_view().slice(1, 1);
        assert_eq!(slice.size(), 1);
        assert_eq!(slice.get(0), &[5]);
    }

    #[test]
    fn test_sparse_1d() {
        let mut vec_1d: WithSparseIndices<isize, Vec<u32>> = WithSparseIndices::empty();
        vec_1d.to_mut_view().emplace_back(|_| { (100, 3u32) });
        vec_1d.to_mut_view().emplace_back(|_| { (200, 5u32) });
        vec_1d.to_mut_view().emplace_back(|_| { (300, 7u32) });
        assert_eq!(vec_1d.to_view().size(), 3);
        assert_eq!(vec_1d.to_view().get(0), (100, &3u32));
        assert_eq!(vec_1d.to_view().get(1), (200, &5u32));
        assert_eq!(vec_1d.to_view().get(2), (300, &7u32));
    }

    #[test]
    fn test_sparse_csr_mat() {
        let mut vec_2d: SparseVec<isize, Vec<i32>> = WithSparseIndices::empty();
        vec_2d.to_mut_view().emplace_back(|v| {
            v.push(3);
            v.push(4);
            v.push(10);
            return (100, ());
        });
        vec_2d.to_mut_view().emplace_back(|v| {
            v.push(5);
            return (200, ());
        });
        vec_2d.to_mut_view().emplace_back(|v| {
            v.push(7);
            v.push(8);
            return (300, ());
        });
        assert_eq!(vec_2d.to_view().size(), 3);
        assert_eq!(vec_2d.to_view().get(0), (100, &[3i32, 4, 10] as &[i32]));
        assert_eq!(vec_2d.to_view().get(1), (200, &[5] as &[i32]));
        assert_eq!(vec_2d.to_view().get(2), (300, &[7, 8] as &[i32]));
        assert_eq!(vec_2d.to_mut_view().size(), 3);
    }

    #[test]
    fn test_deeply_nested() {
        // An dense array of sparse csr matrices.
        let mut vec_3d: DenseVec<SparseVec<i32, Vec<u32>>> = DenseVec::empty();
        vec_3d.to_mut_view().emplace_back(|v| {
            // Insert a sparse matrix in the 0th position
            v.emplace_back(|v| {
                v.push(3);
                v.push(4);
                v.push(10);
                return (100, ());
            });
            v.emplace_back(|v| {
                v.push(5);
                return (200, ());
            });
        });
        vec_3d.to_mut_view().emplace_back(|v| {
            // Insert a sparse matrix in the 1st position
            v.emplace_back(|v| {
                v.push(7);
                v.push(8);
                return (300, ());
            });
        });
        assert_eq!(vec_3d.to_view().size(), 2);
        assert_eq!(vec_3d.to_view().get(0).size(), 2);
        assert_eq!(vec_3d.to_view().get(1).size(), 1);
        assert_eq!(vec_3d.to_view().get(0).get(0), (100, &[3u32, 4, 10] as &[u32]));
        assert_eq!(vec_3d.to_view().get(0).get(1), (200, &[5u32] as &[u32]));
        assert_eq!(vec_3d.to_view().get(1).get(0), (300, &[7u32, 8] as &[u32]));
    }

    fn test_dcsr_mat() {
        let mut vec_2d: SparseVec<isize, WithSparseIndices<usize, Vec<i32>>> = WithSparseIndices::empty();
        vec_2d.to_mut_view().emplace_back(|v| {
            v.emplace_back(|v| {
                return (50, -3i32);
            });
            v.emplace_back(|v| {
                return (100, -4i32);
            });
            v.emplace_back(|v| {
                return (200, -10i32);
            });
            return (1000, ());
        });
        vec_2d.to_mut_view().emplace_back(|v| {
            v.emplace_back(|v| {
                return (300, -5i32);
            });
            return (2000, ());
        });
        vec_2d.to_mut_view().emplace_back(|v| {
            v.emplace_back(|v| {
                return (400, -7i32);
            });
            v.emplace_back(|v| {
                return (500, -8i32);
            });
            return (3000, ());
        });
        assert_eq!(vec_2d.to_view().size(), 3);
        assert_eq!(vec_2d.to_view().get(0).0, 1000);
        assert_eq!(vec_2d.to_view().get(0).1.size(), 3);
        assert_eq!(vec_2d.to_view().get(0).1.get(0), (50, &-3i32));
        assert_eq!(vec_2d.to_view().get(0).1.get(1), (100, &-4i32));
        assert_eq!(vec_2d.to_view().get(0).1.get(2), (200, &-10i32));
        assert_eq!(vec_2d.to_view().get(1).0, 2000);
        assert_eq!(vec_2d.to_view().get(1).1.size(), 1);
        assert_eq!(vec_2d.to_view().get(1).1.get(0), (300, &-5i32));
        assert_eq!(vec_2d.to_view().get(2).0, 3000);
        assert_eq!(vec_2d.to_view().get(2).1.size(), 2);
        assert_eq!(vec_2d.to_view().get(2).1.get(0), (400, &-7i32));
        assert_eq!(vec_2d.to_view().get(2).1.get(1), (500, &-8i32));
    }
}
