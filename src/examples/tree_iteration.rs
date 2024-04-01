use crate::{rbtree::rbtree_lib::RBTree, streams::stream_defs::IndexedStream};

pub fn intersect2_iterators<I: Ord + Copy>(a: &RBTree<I, ()>, b: &RBTree<I, ()>) -> usize {
    a.stream_iter()
        .zip_with(b.stream_iter(), |_, _| ())
        .fold(0, |acc, _, _| acc + 1)
}

pub fn intersect2_manual<I: Ord + Copy>(a: &RBTree<I, ()>, b: &RBTree<I, ()>) -> usize {
    let mut result = 0;
    for (k, _) in a.iter() {
        if b.contains_key(&k) {
            result += 1;
        }
    }
    result
}

pub fn intersect3_iterators<I: Ord + Copy>(a: &RBTree<I, ()>, b: &RBTree<I, ()>, c: &RBTree<I, ()>) -> usize {
    a.stream_iter()
        .zip_with(b.stream_iter(), |_, _| ())
        .zip_with(c.stream_iter(), |_, _| ())
        .fold(0, |acc, _, _| acc + 1)
}

pub fn itersect3_manual<I: Ord + Copy>(a: &RBTree<I, ()>, b: &RBTree<I, ()>, c: &RBTree<I, ()>) -> usize {
    let mut result = 0;
    for (k, _) in a.iter() {
        if b.contains_key(&k) && c.contains_key(&k) {
            result += 1;
        }
    }
    result
}

#[test]
fn test_basic_stream() {
    let tree: RBTree<u64, u64> = RBTree::from_iter([(1, 1), (2, 1), (3, 2), (4, 3), (5, 5)].into_iter());
    let mut stream = tree.stream_iter();
    assert_eq!(stream.index(), 1);
    assert_eq!(*stream.value(), 1);

    stream.seek(3, false);
    assert_eq!(stream.index(), 3);
    assert_eq!(*stream.value(), 2);

    stream.seek(1, true);
    assert_eq!(stream.index(), 3);
    assert_eq!(*stream.value(), 2);

    stream.seek(3, true);
    assert_eq!(stream.index(), 4);
    assert_eq!(*stream.value(), 3);

    stream.next();
    assert_eq!(stream.index(), 5);
    assert_eq!(*stream.value(), 5);

    stream.seek(6, false);
    assert!(!stream.valid());
}