use const_bounded_collections::{BoundedVec, BoundedVecOutOfBounds, EmptyBoundedVec};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

const EXTRACTED: Vec<u8> = EmptyBoundedVec::<u8, 4>::new().to_vec();
const TAKEN: (EmptyBoundedVec<u8, 4>, Vec<u8>) = {
    let mut bounded = EmptyBoundedVec::new();
    let inner = bounded.take_vec();
    (bounded, inner)
};
const CONSTRUCTED: Result<EmptyBoundedVec<u8, 4>, (Vec<u8>, BoundedVecOutOfBounds)> =
    EmptyBoundedVec::from_vec_preserving_input(Vec::new());
const REJECTED: Result<BoundedVec<u8, 1, 4>, (Vec<u8>, BoundedVecOutOfBounds)> =
    BoundedVec::<u8, 1, 4>::from_vec_preserving_input(Vec::new());

#[derive(Debug)]
struct Item(Arc<AtomicUsize>);

impl Drop for Item {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn ownership_operations_are_const_callable() {
    assert!(EXTRACTED.is_empty());
    assert!(TAKEN.0.is_empty());
    assert!(TAKEN.1.is_empty());
    assert!(CONSTRUCTED.unwrap().is_empty());
    assert_eq!(
        REJECTED.unwrap_err(),
        (
            vec![],
            BoundedVecOutOfBounds::LowerBoundError {
                lower_bound: 1,
                got_smaller_by: 1,
            }
        )
    );
}

#[test]
fn extraction_and_take_transfer_allocation_without_dropping_elements() {
    let drops = Arc::new(AtomicUsize::new(0));
    let mut input = Vec::with_capacity(32);
    input.push(Item(drops.clone()));
    let address = input.as_ptr();
    let capacity = input.capacity();

    let bounded = BoundedVec::<_, 1, 4>::from_vec_preserving_input(input).unwrap();
    let output = bounded.to_vec();
    assert_eq!(output.as_ptr(), address);
    assert_eq!(output.capacity(), capacity);

    let mut bounded = EmptyBoundedVec::<_, 4>::from_vec_preserving_input(output).unwrap();
    let output = bounded.take_vec();
    assert!(bounded.is_empty());
    assert_eq!(bounded.capacity(), 0);
    assert_eq!(output.as_ptr(), address);
    assert_eq!(output.capacity(), capacity);
    bounded.try_push(Item(drops.clone())).unwrap();
    drop(bounded);
    assert_eq!(drops.load(Ordering::SeqCst), 1);

    let bounded = EmptyBoundedVec::<_, 4>::from_vec(output).unwrap();
    let output = bounded.to_vec();
    assert_eq!(output.as_ptr(), address);
    assert_eq!(output.capacity(), capacity);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    drop(output);
    assert_eq!(drops.load(Ordering::SeqCst), 2);
}

#[test]
fn rejected_input_preserves_allocation_and_drop_behavior() {
    let drops = Arc::new(AtomicUsize::new(0));
    let mut input = Vec::with_capacity(32);
    input.push(Item(drops.clone()));
    input.push(Item(drops.clone()));
    let address = input.as_ptr();
    let capacity = input.capacity();

    let (input, error) = EmptyBoundedVec::<_, 1>::from_vec_preserving_input(input).unwrap_err();
    assert_eq!(
        error,
        BoundedVecOutOfBounds::UpperBoundError {
            upper_bound: 1,
            got_larger_by: 1
        }
    );
    let (input, error) = BoundedVec::<_, 1, 1>::from_vec_preserving_input(input).unwrap_err();
    assert_eq!(
        error,
        BoundedVecOutOfBounds::UpperBoundError {
            upper_bound: 1,
            got_larger_by: 1
        }
    );
    let (input, error) = BoundedVec::<_, 3, 4>::from_vec_preserving_input(input).unwrap_err();
    assert_eq!(
        error,
        BoundedVecOutOfBounds::LowerBoundError {
            lower_bound: 3,
            got_smaller_by: 1
        }
    );
    assert_eq!(input.as_ptr(), address);
    assert_eq!(input.capacity(), capacity);
    assert_eq!(input.len(), 2);
    assert_eq!(drops.load(Ordering::SeqCst), 0);

    // The existing constructor still drops rejected input.
    assert!(BoundedVec::<_, 3, 4>::from_vec(input).is_err());
    assert_eq!(drops.load(Ordering::SeqCst), 2);
}
