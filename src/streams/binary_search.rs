/// Helper function to compare with a strict parameter
#[inline]
fn lt_or_possibly_eq<I: Ord>(x: &I, y: &I, allow_eq: bool) -> bool {
    match x.cmp(y) {
        std::cmp::Ordering::Less => true,
        std::cmp::Ordering::Equal => allow_eq,
        std::cmp::Ordering::Greater => false,
    }
}


pub(crate) fn binary_search<T: Ord>(arr: &[T], target: &T, strict: bool) -> usize {
    // First, advance to find an upper bound
    let mut step = 1;
    let mut new_cur = 0;
    while new_cur < arr.len() && lt_or_possibly_eq(&arr[new_cur], target, strict) {
        new_cur += step;
        step *= 2;
    }
    // Correct overshoot
    let mut left = step / 2;
    let mut right = if new_cur >= arr.len() { arr.len() } else { new_cur + 1 };

    // Then, do a standard binary search within the found bounds
    while left < right {
        let mid = left + (right - left) / 2;
        if lt_or_possibly_eq(&arr[mid], target, strict){
            left = mid + 1;
        } else {
            right = mid;
        }
    }
    left
}