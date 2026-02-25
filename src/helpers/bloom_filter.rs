// use std::{f64::consts::LOG10_2, hash::Hash};

// pub struct BloomFilter {
//     bits: Vec<u64>,
//     hash_count: u8,
// }

// impl BloomFilter {
//     const P: f64 = 1.0E-9;
//     const P_LOG: f64 = -9.0;
//     const M_FACTOR: f64 = Self::P / -0.09061905829;

//     pub fn new<T: Hash>(elements: &[T]) -> Self {
//         let n = elements.len() as f64;
//         let m = (n * Self::M_FACTOR).ceil();
//         let k = ((m / n) * LOG10_2).round();

//         let size = m as usize / 64;
//         let mut bits = Vec::with_capacity(size);
//         bits.resize_with(size, || 0);

//         let filter = Self {
//             hash_count: k as u8,
//             bits,
//         };

//         for e in elements {}

//         filter
//     }

//     pub fn compare<T: Hash>(&self, hash: T) -> bool {
//         false
//     }
// }
