use num_bigint::{BigInt, U64Digits};
use num_traits::{Bounded, FromPrimitive, Num, One, Pow, Signed, ToPrimitive, Zero};
use std::{
    convert::Into,
    fmt::{Debug, Display},
    iter::Sum,
    ops::{
        Add, AddAssign, BitAnd, BitOr, BitXor, Div, Mul, MulAssign, Neg, Shl, Shr, Sub, SubAssign,
    },
};

#[cfg(feature = "ibig-felt")]
mod ibig_felt;

#[cfg(feature = "ibig-felt")]
use crate::ibig_felt::FeltIBig;

#[cfg(feature = "ibig-felt")]
pub type Felt = FeltIBig;

#[cfg(feature = "bigint-felt")]
mod bigint_felt;

#[cfg(feature = "bigint-felt")]
use crate::bigint_felt::FeltBigInt;

#[cfg(feature = "bigint-felt")]
pub type Felt = FeltBigInt;

pub const PRIME_STR: &str = "0x800000000000011000000000000000000000000000000000000000000000001";
pub const FIELD: (u128, u128) = ((1 << 123) + (17 << 64), 1);

#[derive(Clone, Debug, PartialEq)]
pub struct ParseFeltError;

pub trait NewFelt<B = Self> {
    fn new<T: Into<B>>(value: T) -> Self;
}

pub trait FeltOps {
    fn modpow(&self, exponent: &Felt, modulus: &Felt) -> Self;
    fn mod_floor(&self, other: &Felt) -> Self;
    fn div_floor(&self, other: &Felt) -> Self;
    fn div_mod_floor(&self, other: &Felt) -> (Felt, Felt);
    fn iter_u64_digits(&self) -> U64Digits;
    fn to_signed_bytes_le(&self) -> Vec<u8>;
    fn to_bytes_be(&self) -> Vec<u8>;
    fn parse_bytes(buf: &[u8], radix: u32) -> Option<Felt>;
    fn from_bytes_be(bytes: &[u8]) -> Self;
    fn to_str_radix(&self, radix: u32) -> String;
    fn div_rem(&self, other: &Felt) -> (Felt, Felt);
    fn to_bigint(&self) -> BigInt;
    fn to_bigint_unsigned(&self) -> BigInt;
    fn mul_inverse(&self) -> Self;
    fn sqrt(&self) -> Self;
}

macro_rules! assert_felt_impl {
    ($type:ty) => {
        const _: () = {
            fn assert_new_felt<T: NewFelt>() {}
            fn assert_felt_ops<T: FeltOps>() {}
            fn assert_add<T: Add>() {}
            fn assert_add_ref<'a, T: Add<&'a $type>>() {}
            fn assert_add_u32<T: Add<u32>>() {}
            fn assert_add_usize<T: Add<usize>>() {}
            fn assert_add_ref_usize<T: Add<usize>>() {}
            fn assert_add_assign<'a, T: AddAssign<&'a $type>>() {}
            fn assert_neg<T: Neg>() {}
            fn assert_sub<T: Sub<$type, Output = $type>>() {}
            fn assert_sub_ref<'a, T: Sub<&'a $type, Output = $type>>() {}
            fn assert_sub_assign<'a, T: SubAssign<&'a $type>>() {}
            fn assert_mul<T: Mul<$type, Output = $type>>() {}
            fn assert_mul_ref<'a, T: Mul<&'a $type, Output = $type>>() {}
            fn assert_mul_assign<'a, T: MulAssign<&'a $type>>() {}
            fn assert_div<T: Div>() {}
            fn assert_div_ref<T: Div>() {}
            fn assert_shl_u32<T: Shl<u32>>() {}
            fn assert_shl_usize<T: Shl<usize>>() {}
            fn assert_shl_ref_usize<T: Shl<usize>>() {}
            fn assert_shr_u32<T: Shr<u32>>() {}
            fn assert_bitand<'a, T: BitAnd<&'a $type, Output = $type>>() {}
            fn assert_bitand_ref<T: BitAnd>() {}
            fn assert_bitor<T: BitOr>() {}
            fn assert_bitxor<T: BitXor>() {}
            fn assert_sum<T: Sum<$type>>() {}
            fn assert_pow<T: Pow<u32>>() {}
            fn assert_pow_ref<T: Pow<u32>>() {}
            fn assert_num<T: Num>() {}
            fn assert_zero<T: Zero>() {}
            fn assert_one<T: One>() {}
            fn assert_bounded<T: Bounded>() {}
            fn assert_signed<T: Signed>() {}
            fn assert_from_primitive<T: FromPrimitive>() {}
            fn assert_to_primitive<T: ToPrimitive>() {}
            fn assert_display<T: Display>() {}
            fn assert_debug<T: Debug>() {}

            // RFC 2056
            #[allow(dead_code)]
            fn assert_all() {
                assert_new_felt::<$type>();
                //                assert_felt_ops::<$type>();
                assert_add::<$type>();
                assert_add_ref::<$type>();
                assert_add_u32::<$type>();
                assert_add_usize::<$type>();
                assert_add_ref_usize::<&$type>();
                assert_add_assign::<$type>();
                assert_sub::<$type>();
                assert_sub_ref::<$type>();
                assert_sub_assign::<$type>();
                assert_mul::<$type>();
                assert_mul_ref::<$type>();
                assert_mul_assign::<$type>();
                assert_div::<$type>();
                assert_div_ref::<&$type>();
                assert_neg::<$type>();
                assert_bitand::<$type>();
                assert_bitand_ref::<&$type>();
                assert_bitor::<&$type>();
                assert_bitxor::<&$type>();
                assert_sum::<$type>();
                assert_pow::<$type>();
                assert_pow_ref::<&$type>();
                assert_num::<$type>();
                assert_zero::<$type>();
                assert_one::<$type>();
                assert_bounded::<$type>();
                assert_signed::<$type>();
                assert_from_primitive::<$type>();
                assert_to_primitive::<$type>();
                assert_shl_u32::<$type>();
                assert_shl_usize::<$type>();
                assert_shl_ref_usize::<&$type>();
                assert_shr_u32::<$type>();
                assert_display::<$type>();
                assert_debug::<$type>();
            }
        };
    };
}

assert_felt_impl!(Felt);
// assert_felt_impl!(FeltIBig);

pub trait NewStr {
    fn new_str(num: &str, base: u8) -> Self;
}

macro_rules! assert_felt_test_impl {
    ($type:ty) => {
        const _: () = {
            fn assert_new_str<T: NewStr>() {}
            fn assert_all() {
                assert_new_str::<$type>();
            }
        };
    };
}

assert_felt_test_impl!(Felt);

#[macro_use]
pub mod felt_test_utils {
    macro_rules! _felt {
        ($val: expr) => {
            Felt::new($val)
        };
    }
    //pub use felt;

    macro_rules! _felt_str {
        ($val: expr) => {
            Felt::new_str($val, 10)
        };
        ($val: expr, $opt: expr) => {
            Felt::new_str($val, $opt)
        };
    }
    //pub use felt_str;
}