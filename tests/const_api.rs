use const_bounded_collections::{BoundedVec, EmptyBoundedVec};

const EMPTY: EmptyBoundedVec<u8, 4> = EmptyBoundedVec::new();
static VALUE: EmptyBoundedVec<u8, 4> = EMPTY;

#[expect(clippy::len_zero, reason = "verify len itself is const-callable")]
const _: () = {
    assert!(VALUE.len() == 0);
    assert!(VALUE.is_empty());
    assert!(VALUE.capacity() == 0);
    assert!(VALUE.as_vec().is_empty());
    assert!(VALUE.as_slice().is_empty());
    assert!(VALUE.first().is_none());
    assert!(VALUE.last().is_none());
    assert!(VALUE.get(0).is_none());
    assert!(VALUE.get(usize::MAX).is_none());
    #[cfg(feature = "panic")]
    {
        assert!(VALUE.split_at(0).0.is_empty());
    }
    assert!(VALUE.split_at_checked(0).is_some());
    assert!(VALUE.split_at_checked(usize::MAX).is_none());
};

const MUTATED_EMPTY: EmptyBoundedVec<u8, 4> = {
    let mut value = EmptyBoundedVec::new();
    assert!(value.as_mut_slice().is_empty());
    assert!(value.first_mut().is_none());
    assert!(value.last_mut().is_none());
    assert!(value.get_mut(0).is_none());
    assert!(value.get_mut(usize::MAX).is_none());
    value.reverse();
    #[cfg(feature = "panic")]
    {
        value.rotate_left(0);
        value.rotate_right(0);
        value.copy_from_slice(&[]);
        assert!(value.split_at_mut(0).1.is_empty());
    }
    assert!(value.split_at_mut_checked(0).is_some());
    assert!(value.split_at_mut_checked(usize::MAX).is_none());
    value
};

// Const functions also verify the nonempty accessors without requiring a
// heap-allocated vector to be constructed during constant evaluation.
const fn endpoints<const L: usize>(value: &BoundedVec<u8, L, 4>) -> (u8, u8) {
    let (first, tail) = value.split_first();
    let (last, head) = value.split_last();
    assert!(*first == *value.first());
    assert!(*last == *value.last());
    assert!(tail.len() + 1 == value.len());
    assert!(head.len() + 1 == value.len());
    (*first, *last)
}

const fn mutate<const L: usize>(value: &mut BoundedVec<u8, L, 4>) {
    *value.first_mut() += 1;
    *value.last_mut() += 1;
    let (first, _) = value.split_first_mut();
    *first += 1;
    let (last, _) = value.split_last_mut();
    *last += 1;
    if let Some(first) = value.get_mut(0) {
        *first += 1;
    }
    assert!(value.get_mut(usize::MAX).is_none());
}

#[test]
fn const_accessors_work_with_runtime_contents() {
    assert!(MUTATED_EMPTY.is_empty());
    let mut value = BoundedVec::<u8, 2, 4>::from_vec(vec![1, 2]).unwrap();
    assert_eq!(endpoints(&value), (1, 2));
    mutate(&mut value);
    assert_eq!(endpoints(&value), (4, 4));
    assert_eq!(value.get(0), Some(&4));
    assert_eq!(value.get(usize::MAX), None);
}

#[cfg(feature = "panic")]
const fn rearrange(value: &mut BoundedVec<u8, 4, 4>) {
    value.swap(0, 3);
    value.reverse();
    value.rotate_left(1);
    value.rotate_right(2);
    let (left, right) = value.split_at_mut(2);
    left[0] += 10;
    right[0] += 20;
    if let Some((left, right)) = value.split_at_mut_checked(2) {
        left[1] += 30;
        right[1] += 40;
    }
}

#[cfg(feature = "panic")]
const fn overwrite(value: &mut BoundedVec<u8, 4, 4>) {
    value.copy_from_slice(&[5, 6, 7, 8]);
}

#[test]
#[cfg(feature = "panic")]
fn const_slice_operations_preserve_fixed_bounds() {
    let mut value = BoundedVec::<u8, 4, 4>::from_vec(vec![1, 2, 3, 4]).unwrap();
    rearrange(&mut value);
    assert_eq!(value.as_slice(), &[14, 31, 23, 42]);
    overwrite(&mut value);
    assert_eq!(value.as_slice(), &[5, 6, 7, 8]);
    assert_eq!(value.split_at(2), (&[5, 6][..], &[7, 8][..]));
    assert_eq!(value.split_at_checked(5), None);
}
