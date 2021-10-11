
Introduction
===

Notice: Work in progress!

An attempt at writing:

* Compile time einsum based expressions.
* A toolbox of convenience macros
  
for the ndarray library.

Examples
====

Einsum example:

```rust

use ndarray as nd;

use extended_einsum::ein;

let a = nd::array![[1., 2., 3.], [4., 5., 6.]];

let b = nd::array![[7., 8.], [9., 10.], [11., 12.]];

let c = nd::array![[13., 14., 15.]];

let result = ein! { a[D1, D2] * b[D2, D3] + c[D3] ~ [D1, D3] };
```

Block matrix example:

```rust

use ndarray as nd;

use extended_einsum::block_mat;

let a = nd::array![[1., 2., 3.], [4., 5., 6.]];

let b = nd::array![[7., 8., 9.], [10., 11., 12.]];

let c = nd::array![[13., 14., 15.], [16., 17., 18.]];

let d = nd::array![[19., 20., 21.], [22., 23., 24.]];

// Matrix construction

let abcd = block_mat! { [a, b], [c, d] };
```


Roadmap
===

* Display warnings when dimensions are inconsistent across ein blocks.
  
* Handle a wider range of loop constructs and function calls:

```rust
// named elements
let D2 = register_element_names! { V1, V2, V3, V4 };

// wildcards
let b = ein!{ A[D1, *] * B[D1, D1] ~ [*]};

// functions
fn process_d2<T1, T2>(d2_row: T1, scalar: T2) {
    d2_row[V1] * d2_row[V2]
        + d2_row[V3].pow(2)
        + scalar
}

let c = ein! {
    let temp[D1, D3] = forall[D1] {
        process_d2(A[D1, D2], x[D1]) ~ [D3];
    };

    temp[D1, D3] + b[D2] ~ [D1, D2, D3]
};
```

* Improve upon the naive "for loop" approach, perhaps E-graph based optimisation of expressions with the Rust "egg" library.

* Handle the nalgebra package as well as the ndarray package

  * Handle both dynamic and static array types

* Add benchmarks to compare to more native operations.


License
=======

Dual-licensed to be compatible with the Rust project.

Licensed under the Apache License, Version 2.0
http://www.apache.org/licenses/LICENSE-2.0 or the MIT license
http://opensource.org/licenses/MIT, at your
option. This file may not be copied, modified, or distributed
except according to those terms.
