use crate::{rbtree::rbtree_lib::RBTree, streams::stream_defs::IndexedStream};

pub fn intersect2_iterators<I: Ord + Copy>(a: &RBTree<I, ()>, b: &RBTree<I, ()>) -> usize {
    a.stream_iter()
        .zip_with(b.stream_iter(), |_, _| ())
        .fold(0, |acc, _, _| acc + 1)
}

pub fn intersect2_manual<I: Ord + Copy>(a: &RBTree<I, ()>, b: &RBTree<I, ()>) -> usize {
    a.iter()
        .filter(|(k, _)| b.contains_key(k))
        .count()
}

pub fn intersect3_iterators<I: Ord + Copy>(a: &RBTree<I, ()>, b: &RBTree<I, ()>, c: &RBTree<I, ()>) -> usize {
    a.stream_iter()
        .zip_with(b.stream_iter(), |_, _| ())
        .zip_with(c.stream_iter(), |_, _| ())
        .fold(0, |acc, _, _| acc + 1)
}

pub fn itersect3_manual<I: Ord + Copy>(a: &RBTree<I, ()>, b: &RBTree<I, ()>, c: &RBTree<I, ()>) -> usize {
    a.iter()
        .filter(|(k, _)| b.contains_key(k) && c.contains_key(k))
        .count()
}

#[cfg(test)]
mod test {
    use std::collections::BTreeSet;

    use quickcheck_macros::quickcheck;

    use crate::{examples::tree_iteration::{intersect2_iterators, intersect2_manual, intersect3_iterators, itersect3_manual}, rbtree::rbtree_lib::RBTree, streams::stream_defs::IndexedStream};

    
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

    #[quickcheck]
    fn test_intersection(a: BTreeSet<u8>, b: BTreeSet<u8>, c: BTreeSet<u8>) {
        fn make_rbset<I: Ord + Copy>(data: impl IntoIterator<Item = I>) -> RBTree<I, ()> {
            data.into_iter().map(|x| (x, ())).collect()
        }
        let tree_a = make_rbset(a);
        let tree_b = make_rbset(b);
        let tree_c = make_rbset(c);
        assert_eq!(intersect2_iterators(&tree_a, &tree_b), intersect2_manual(&tree_a, &tree_b));
        assert_eq!(intersect3_iterators(&tree_a, &tree_b, &tree_c), itersect3_manual(&tree_a, &tree_b, &tree_c));
    }
}
