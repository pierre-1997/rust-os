pub trait GetBit {
    /// Gets a single bit from `self`.
    ///
    /// NOTE: `idx` is 0-indexed "from the right".
    ///
    /// TODO: Should we return a `Self` instead of a `bool`?
    fn get_bit(&self, idx: usize) -> bool;

    /// Gets multiple bits.
    ///
    /// NOTE: `first_idx` is the index of the first bit to get.
    fn get_bits(&self, first_idx: usize, len: usize) -> Self;
}

pub trait SetBit {
    /// Sets a single bit in `self`.
    ///
    /// NOTE: `idx` is 0-indexed "from the right".
    fn set_bit(&mut self, idx: usize, value: bool);

    /// Sets multiple bits at the given index.
    ///
    /// NOTE: `first_idx` is the index of the first bit that'll set.
    fn set_bits(&mut self, first_idx: u32, len: u32, value: Self);
}

macro_rules! impl_get_bit {
    ($t:ty) => {
        impl GetBit for $t {
            fn get_bit(&self, idx: usize) -> bool {
                (self & (1 << idx)) != 0
            }

            fn get_bits(&self, first_idx: usize, len: usize) -> Self {
                let mask = (1 << len) - 1;

                (self >> ((first_idx + 1) - len)) & mask
            }
        }
    };
}

impl_get_bit!(u8);
// impl_get_bit!(u16);
impl_get_bit!(u32);
impl_get_bit!(u64);

macro_rules! impl_set_bit {
    ($t:ty) => {
        impl SetBit for $t {
            fn set_bit(&mut self, idx: usize, value: bool) {
                *self = (*self & !(1 << idx)) | (if value { 1 } else { 0 }) << idx;
            }

            fn set_bits(&mut self, first_idx: u32, len: u32, value: Self) {
                let mask = Self::MAX >> (Self::BITS - len);
                let mask = !(mask << ((first_idx + 1) - len));

                *self = (*self & mask) | (value << (first_idx + 1 - len));
            }
        }
    };
}

impl_set_bit!(u8);
// impl_set_bit!(u16);
// impl_set_bit!(u32);
impl_set_bit!(u64);

#[cfg(test)]
mod tests {
    use crate::testing::TestCase;

    use super::*;

    #[test_case]
    fn test_get_bit() -> TestCase {
        TestCase {
            name: "Test GetBit trait by getting single bits",
            test: || {
                assert_eq!(0u8.get_bit(3), false);
                assert_eq!(0u8.get_bit(7), false);
                assert_eq!(0xFFu8.get_bit(3), true);
                assert_eq!(0xFFu8.get_bit(6), true);
                assert_eq!(0x9Au8.get_bit(7), true);
                assert_eq!(0x9Au8.get_bit(4), true);
            },
        }
    }

    #[test_case]
    fn test_get_bits() -> TestCase {
        TestCase {
            name: "Test GetBit trait by getting multiple bits",
            test: || {
                assert_eq!(0u8.get_bits(3, 2), 0);
                assert_eq!(0xFFu8.get_bits(3, 2), 3);
                assert_eq!(0x30u8.get_bits(5, 2), 3);

                assert_eq!(0x0000000012345678u64.get_bits(31, 32), 0x12345678);
                assert_eq!(0x1234567800000000u64.get_bits(63, 32), 0x12345678);
            },
        }
    }

    #[test_case]
    fn test_set_bit() -> TestCase {
        TestCase {
            name: "Test SetBit trait by setting single bits",
            test: || {
                let mut val = 0x00u8;
                val.set_bit(3, true);
                assert_eq!(val, 8);
                val.set_bit(3, false);
                assert_eq!(val, 0);
            },
        }
    }

    #[test_case]
    fn test_set_bits() -> TestCase {
        TestCase {
            name: "Test SetBit trait by setting multiple bits",
            test: || {
                let mut val = 0x00u8;
                val.set_bits(5, 2, 3);
                assert_eq!(val, 0x30);
                val.set_bits(5, 2, 0);
                assert_eq!(val, 0);

                let mut v = 0u64;
                v.set_bits(31, 32, 0x12345678);
                assert_eq!(v, 0x12345678);

                let mut v = 0u64;
                v.set_bits(63, 32, 0x12345678);
                assert_eq!(v, 0x1234567800000000);
            },
        }
    }

    #[test_case]
    fn test_clear_set_bit() -> TestCase {
        TestCase {
            name: "Test SetBit trait by clearing bits",
            test: || {
                let mut val = 0xFFu8;

                val.set_bit(0, false);
                val.set_bit(1, false);
                val.set_bit(2, false);
                val.set_bit(3, false);
                val.set_bit(4, false);
                val.set_bit(5, false);
                val.set_bit(6, false);
                val.set_bit(7, false);
                assert_eq!(val, 0);
            },
        }
    }
}
