use itertools::Itertools;
use num::bigint::BigUint;
use num::{Integer, One, Zero};
use std::convert::TryInto;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::{Product, Sum};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::field::field_types::{Field, PrimeField};
use crate::field::goldilocks_field::GoldilocksField;

/// EPSILON = 9 * 2**28 - 1
const EPSILON: u64 = 2415919103;

/// A field designed for use with the Crandall reduction algorithm.
///
/// Its order is
/// ```ignore
/// P = 2**256 - 2**32 - 2**9 - 2**8 - 2**7 - 2**6 - 2**4 - 1
/// ```
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Secp256K1Base(pub [u32; 8]);

impl Secp256K1Base {
    const ORDER_BIGUINT: BigUint = BigUint::from_slice(&[
        0xFFFFFC2F,
        0xFFFFFFFE,
        0xFFFFFFFF,
        0xFFFFFFFF,
        0xFFFFFFFF,
        0xFFFFFFFF,
        0xFFFFFFFF,
        0xFFFFFFFF,
    ]);

    fn to_canonical_biguint(&self) -> BigUint {
        BigUint::from_slice(&self.0).mod_floor(&Self::ORDER_BIGUINT)
    }

    fn from_biguint(val: BigUint) -> Self {
        Self(val.to_u32_digits().iter().cloned().pad_using(8, |_| 0).collect::<Vec<_>>()[..8].try_into().expect("error converting to u32 array; should never happen"))
    }
}

impl Default for Secp256K1Base {
    fn default() -> Self {
        Self::ZERO
    }
}

impl PartialEq for Secp256K1Base {
    fn eq(&self, other: &Self) -> bool {
        self.to_canonical_biguint() == other.to_canonical_biguint()
    }
}

impl Eq for Secp256K1Base {}

impl Hash for Secp256K1Base {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_canonical_biguint().iter_u64_digits().for_each(|digit| state.write_u64(digit))
    }
}

impl Display for Secp256K1Base {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.to_canonical_biguint(), f)
    }
}

impl Debug for Secp256K1Base {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.to_canonical_biguint(), f)
    }
}

impl Field for Secp256K1Base {
    // TODO: fix
    type PrimeField = GoldilocksField;

    const ZERO: Self = Self::from_biguint(BigUint::zero());
    const ONE: Self = Self::from_biguint(BigUint::one());
    const TWO: Self = Self::from_biguint(BigUint::one() + BigUint::one());
    const NEG_ONE: Self = Self::from_biguint(Self::ORDER_BIGUINT - BigUint::one());

    // TODO: fix
    const CHARACTERISTIC: u64 = 0;
    const TWO_ADICITY: usize = 1;

    const MULTIPLICATIVE_GROUP_GENERATOR: Self = todo!();//Self(5);
    const POWER_OF_TWO_GENERATOR: Self = todo!();//Self(10281950781551402419);

    fn order() -> BigUint {
        Self::ORDER_BIGUINT
    }

    fn try_inverse(&self) -> Option<Self> {
        if self.is_zero() {
            return None;
        }

        // Fermat's Little Theorem
        Some(self.exp_biguint(&(Self::ORDER_BIGUINT - BigUint::one() - BigUint::one())))
    }

    #[inline]
    fn from_canonical_u64(n: u64) -> Self {
        Self([n as u32, (n >> 32) as u32, 0, 0, 0, 0, 0, 0])
    }

    #[inline]
    fn from_noncanonical_u128(n: u128) -> Self {
        Self([
            n as u32,
            (n >> 32) as u32,
            (n >> 64) as u32,
            (n >> 96) as u32,
            0,
            0,
            0,
            0,
        ])
    }

    #[inline]
    fn from_noncanonical_u96(n: (u64, u32)) -> Self {
        Self([
            n.0 as u32,
            (n.0 >> 32) as u32,
            n.1,
            0,
            0,
            0,
            0,
            0,
        ])
    }

    fn rand_from_rng<R: Rng>(rng: &mut R) -> Self {
        let mut array = [0u32; 8];
        rng.fill(&mut array);
        let mut rand_biguint = BigUint::from_slice(&array);
        while rand_biguint > Self::ORDER_BIGUINT {
            rng.fill(&mut array);
            rand_biguint = BigUint::from_slice(&array);
        }
        Self(array)
    }
}

impl Neg for Secp256K1Base {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        if self.is_zero() {
            Self::ZERO
        } else {
            Self::from_biguint(Self::ORDER_BIGUINT - self.to_canonical_biguint())
        }
    }
}

impl Add for Secp256K1Base {
    type Output = Self;

    #[inline]

    fn add(self, rhs: Self) -> Self {
        let mut result = self.to_canonical_biguint() + rhs.to_canonical_biguint();
        if result > Self::ORDER_BIGUINT {
            result -= Self::ORDER_BIGUINT;
        }
        Self::from_biguint(result)
    }
}

impl AddAssign for Secp256K1Base {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sum for Secp256K1Base {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, |acc, x| acc + x)
    }
}

impl Sub for Secp256K1Base {
    type Output = Self;

    #[inline]
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn sub(self, rhs: Self) -> Self {
        Self::from_biguint(self.to_canonical_biguint() + Self::ORDER_BIGUINT - rhs.to_canonical_biguint())
    }
}

impl SubAssign for Secp256K1Base {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Mul for Secp256K1Base {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self {
        Self::from_biguint((self.to_canonical_biguint() * rhs.to_canonical_biguint()).mod_floor(&Self::ORDER_BIGUINT))
    }
}

impl MulAssign for Secp256K1Base {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl Product for Secp256K1Base {
    #[inline]
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(|acc, x| acc * x).unwrap_or(Self::ONE)
    }
}

impl Div for Secp256K1Base {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn div(self, rhs: Self) -> Self::Output {
        self * rhs.inverse()
    }
}

impl DivAssign for Secp256K1Base {
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

#[cfg(test)]
mod tests {
    use crate::{test_field_arithmetic, test_prime_field_arithmetic};

    test_prime_field_arithmetic!(crate::field::secp256k1::Secp256K1Base);
    test_field_arithmetic!(crate::field::secp256k1::Secp256K1Base);
}
