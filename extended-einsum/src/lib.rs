#[macro_use]
extern crate extended_einsum_macros;
pub use extended_einsum_macros::*;

// let d = block_mat!{ [A, B], [C, D] };

#[macro_export]
macro_rules! block_mat {

    // multiple rows
    ([$($first_row:ident), +], $([$($other_rows:ident), +]), +) => {
        nd::concatenate![
            nd::Axis(0),
            block_mat!{[$($first_row), +]},
            block_mat!{$([$($other_rows), +]), +}
        ]
    };

    // single row - concat columns
    ([$($columns:ident), +]) => {
        nd::concatenate![nd::Axis(1), $($columns),+]
    };

}

#[cfg(test)]
mod tests {

    use ndarray as nd;

    use crate::{block_mat, ein};

    #[test]
    fn basic_tests() {
        // register_dims! { D1, D2, D3 };

        // register_dims! { D1, D2, D3<V1, V2, V3, V4> };

        // basic einsum

        let a = nd::array![[1., 2., 3.], [4., 5., 6.]];

        let b = nd::array![[7., 8.], [9., 10.], [11., 12.]];

        let c = nd::array![13., 14.];

        // let result = { let D1_length = a . shape () [0usize] ; let D2_length = a . shape () [1usize] ; assert ! (b . shape () [0usize] == D2_length) ; let D3_length = b . shape () [1usize] ; assert ! (c . shape () [0usize] == D3_length) ; let mut result = nd :: Array::<f32, _> :: zeros ((D1_length , D3_length)) ; for D3_index in 0 .. D3_length { for D2_index in 0 .. D2_length { for D1_index in 0 .. D1_length { result [[D1_index , D3_index]] += a [[D1_index , D2_index]] * b [[D2_index , D3_index]] + c [[D3_index]] ; } } } result };

        let result = ein! { a[D1, D2] * b[D2, D3] + c[D3] ~ [D1, D3] };

        let result_expected = a.dot(&b) + c;

        assert_eq!(result, result_expected);

        let c = nd::array![[13., 0.], [14., 0.]];

        let result = ein! { a[D1, D2] * b[D2, D3] + c[D3, D4] ~ [D1, D3] };

        assert_eq!(result, result_expected);

        // TODO add check with wrong dims raises error

        // let b = ein!{ A[D1, *] * B[D1, D1] ~ [*]}

        // fn process_d2<T1, T2>(d2_row: T1, scalar: T2) {
        //     d2_row[0] * d2_row[1]
        //         + d2_row[2].pow(2)
        //         + scalar
        // }

        // let c = ein! {

        //     let temp[D1, D3] = forall[D1] {
        //         process_d2(A[D1, D2], x[D1]) ~ [D3];
        //     };

        //     temp[D1, D3] + b[D2] ~ [D1, D2, D3]

        // };
    }

    #[test]
    fn test_matrix_construction() {
        let a = nd::array![[1., 2., 3.], [4., 5., 6.]];

        let b = nd::array![[7., 8., 9.], [10., 11., 12.]];

        let c =
            nd::array![[13., 14., 15.], [16., 17., 18.]];

        let d =
            nd::array![[19., 20., 21.], [22., 23., 24.]];

        // Matrix construction

        let ab_expected =
            nd::concatenate![nd::Axis(1), a, b];

        let cd_expected =
            nd::concatenate![nd::Axis(1), c, d];

        let ab = block_mat! { [a, b] };

        assert_eq!(ab, ab_expected);

        let abcd_expected = nd::concatenate![
            nd::Axis(0),
            ab_expected,
            cd_expected
        ];

        let abcd = block_mat! { [a, b], [c, d] };

        assert_eq!(abcd, abcd_expected);
    }
}
