
## Const bounded collections
`BoundedVec<T, L, U>` - Non-empty Rust `alloc::vec::Vec` wrapper with type guarantees on lower(`L`) and upper(`U`) bounds for items quantity (`0 < L <= U`).
`EmptyBoundedVec<T,U>` if only upper bound `U` is needed.
`NonEmptyVec<T>` if a lower bound of one and the largest supported upper bound are needed (`usize::MAX`, or `u32::MAX` with `schemars` or `borsh`).
This crate is `#![no_std]` compatible with `alloc`.

## Nightly

This crate works on stable Rust.
My analysis shows that nighly wil require full `alloc::vec::Vec` rewrite from scratch,
so not attemp to support nightly here.

## Example

```rust
use const_bounded_collections::BoundedVec;

let data: BoundedVec<u8, 2, 4> = [1u8,2].into();

assert_eq!(*data.first(), 1);
assert_eq!(*data.last(), 2);

// creates a new BoundedVec by mapping each element
let data = data.mapped(|x|x*2);
assert_eq!(data, [2u8,4].into());
```

## Crate features

- no features enabled by default
- optional(non-default) [serde](https://serde.rs/) feature that adds serialization and deserialization to `BoundedVec`.
- optional(non-default) `schemars` feature that adds JSON schema support via [schemars](https://graham.cool/schemars) (requires `serde`).
- optional(non-default) `arbitrary` feature that adds `proptest::Arbitrary` implementation to `BoundedVec`.
- optional(non-default) `borsh` feature that adds `borsh` binary encoding and decoding.
- optional(non-default) `borsh_schema` feature that adds `borsh` schema support (requires `borsh`).
- optional(non-default) `panic` feature that adds `push`, `insert`, and `Extend`, which panic on upper bound violations, plus mutable access to the underlying `Vec`, which can bypass the bounds. Fallible `try_push`, `try_insert`, and `try_extend` are available without this feature; invalid indices can still panic.
- optional(non-default) `nondeterministic` feature that adds inherent `sort_unstable`, `sort_unstable_by`, and `sort_unstable_by_key` methods.

### `panic`

Enables `push`, `insert`, `Extend<T>`, and `Extend<&T>` (for cloneable elements).
These operations panic when growth would exceed `U`. `insert` also panics when
`index > len`. `Extend` may append some elements before panicking; use
`try_extend` to leave the vector unchanged on an upper-bound error.

The feature also enables `AsMut<Vec<T>>`. Mutating that underlying vector bypasses
bound checks, so callers must preserve `L <= len <= U` themselves.

`try_push`, `try_insert`, and `try_extend` are available without this feature.
Disabling `panic` does not make every operation panic-free: invalid indices and
user-provided callbacks can still panic.

### `nondeterministic`

Enables the three inherent `sort_unstable*` methods with method-level `cfg` gates.
They preserve length but may reorder elements that compare equal. The feature
name refers to this unspecified tie order; it does not introduce randomness.

Stable `sort`, `sort_by`, `sort_by_key`, and `sort_by_cached_key` are always
available and preserve the relative order of equal elements or keys.

This feature controls the inherent methods only. `DerefMut<Target = [T]>` and
mutable slice access still expose Rust's slice sorting methods. In particular,
`vector.sort_unstable()` can resolve to the slice method when the feature is off.

Enable either feature independently, or both:

```toml
[dependencies]
const-bounded-collections = { version = "0.10", features = ["panic", "nondeterministic"] }
```

# Inspired 

- [vec1](https://github.com/rustonaut/vec1)
- [bounded-vec](https://github.com/ergoplatform/bounded-vec) - i added bounded witness to this crate
