use rand_xoshiro::{rand_core::RngCore, Xoshiro256PlusPlus};
use xxhash_rust::xxh3::xxh3_128;

use crate::prng::make_prng;

pub struct SubRandomnessProvider {
    rng: Xoshiro256PlusPlus,
}

impl SubRandomnessProvider {
    pub fn provide(&mut self) -> [u8; 32] {
        let mut out = [0u8; 32];
        self.rng.fill_bytes(&mut out);
        out
    }
}

impl Iterator for SubRandomnessProvider {
    type Item = [u8; 32];

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.provide())
    }
}

/// Takes a randomness and a key. Returns an arbitrary number of sub-randomnesses.
/// The key is mixed into the randomness such that calling this function with different keys
/// leads to different outputs. Calling it with the same key and randomness leads to the same outputs.
///
/// # Examples
///
/// Rolling two dice
///
/// ```
/// use nois::{int_in_range, randomness_from_str, sub_randomness_with_key};
///
/// let randomness = randomness_from_str("9e8e26615f51552aa3b18b6f0bcf0dae5afbe30321e8d7ea7fa51ebeb1d8fe62").unwrap();
///
/// let mut provider = sub_randomness_with_key(randomness, "Key");
///
/// let dice1_subrandomness = provider.provide();
/// let dice2_subrandomness = provider.provide();
///
/// let dice1_result = int_in_range(dice1_subrandomness, 1, 6);
/// let dice2_result = int_in_range(dice2_subrandomness, 1, 6);
/// ```
pub fn sub_randomness_with_key(
    mut randomness: [u8; 32],
    key: impl AsRef<[u8]>,
) -> Box<SubRandomnessProvider> {
    let hashed_key = xxh3_128(key.as_ref()).to_be_bytes();
    for (pos, byte) in hashed_key.iter().enumerate() {
        randomness[pos] ^= byte;
    }

    let rng = make_prng(randomness);

    Box::new(SubRandomnessProvider { rng })
}

/// Takes a randomness and a key. Returns an arbitrary number of sub-randomnesses.
///
/// This is equivalent to calling [`sub_randomness_with_key`] with key `b"_^default^_"`.
///
/// # Example
///
/// Rolling two dice
///
///  ```
/// use nois::{int_in_range, randomness_from_str, sub_randomness};
///
/// let randomness = randomness_from_str("9e8e26615f51552aa3b18b6f0bcf0dae5afbe30321e8d7ea7fa51ebeb1d8fe62").unwrap();
///
/// let mut provider = sub_randomness(randomness);
///
/// let dice1_subrandomness = provider.provide();
/// let dice2_subrandomness = provider.provide();
///
/// let dice1_result = int_in_range(dice1_subrandomness, 1, 6);
/// let dice2_result = int_in_range(dice2_subrandomness, 1, 6);
/// ```
///
/// Roll 1200 dice using the iterator interface:
///
/// ```
/// use std::collections::BTreeMap;
/// use nois::{randomness_from_str, roll_dice, sub_randomness};
///
/// let randomness = randomness_from_str("9e8e26615f51552aa3b18b6f0bcf0dae5afbe30321e8d7ea7fa51ebeb1d8fe62").unwrap();
///
/// let mut results = BTreeMap::<u8, usize>::new();
/// for sub_randomness in sub_randomness(randomness).take(1200) {
///     let number = roll_dice(sub_randomness);
///     let current = results.get(&number).copied().unwrap_or_default();
///     results.insert(number, current + 1);
/// }
/// let ones = results.get(&1).copied().unwrap_or_default();
/// let twos = results.get(&2).copied().unwrap_or_default();
/// let threes = results.get(&3).copied().unwrap_or_default();
/// let fours = results.get(&4).copied().unwrap_or_default();
/// let fives = results.get(&5).copied().unwrap_or_default();
/// let sixes = results.get(&6).copied().unwrap_or_default();
/// println!("{ones} {twos} {threes} {fours} {fives} {sixes}");
/// assert!(ones > 160 && ones < 240);
/// assert!(twos > 160 && twos < 240);
/// assert!(threes > 160 && threes < 240);
/// assert!(fours > 160 && fours < 240);
/// assert!(fives > 160 && fives < 240);
/// assert!(sixes > 160 && sixes < 240);
/// assert_eq!(results.values().sum::<usize>(), 1200);
/// ```
pub fn sub_randomness(randomness: [u8; 32]) -> Box<SubRandomnessProvider> {
    sub_randomness_with_key(randomness, b"_^default^_")
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::HexBinary;

    use crate::{coinflip, pick, RANDOMNESS1};

    use super::*;

    #[test]
    fn sub_randomness_with_key_works() {
        // outputs are the same for the same randomness and key
        let mut provider1 = sub_randomness_with_key([0xA6; 32], "A");
        let mut provider2 = sub_randomness_with_key([0xA6; 32], "A");
        assert_eq!(provider1.provide(), provider2.provide());
        assert_eq!(provider1.provide(), provider2.provide());
        assert_eq!(provider1.provide(), provider2.provide());

        // outputs are different for the same randomness and different key
        let mut provider1 = sub_randomness_with_key([0xA6; 32], "/my_namespace/ab");
        let mut provider2 = sub_randomness_with_key([0xA6; 32], "/my_namespace/cd");
        assert_ne!(provider1.provide(), provider2.provide());
        assert_ne!(provider1.provide(), provider2.provide());
        assert_ne!(provider1.provide(), provider2.provide());
    }

    #[test]
    fn sub_randomness_works() {
        let randomness: [u8; 32] = [0x77; 32];
        let mut provider = sub_randomness(randomness);
        let v1 = provider.provide();
        let v2 = provider.provide();
        let v3 = provider.provide();
        let v4 = provider.provide();
        println!("v1 = {v1:?}");
        println!("v2 = {v2:?}");
        println!("v3 = {v3:?}");
        println!("v4 = {v4:?}");

        // outputs are the same for the same randomness
        let mut provider1 = sub_randomness([0xA6; 32]);
        let mut provider2 = sub_randomness([0xA6; 32]);
        assert_eq!(provider1.provide(), provider2.provide());
        assert_eq!(provider1.provide(), provider2.provide());
        assert_eq!(provider1.provide(), provider2.provide());

        // outputs differ for different randomness
        let mut provider1 = sub_randomness([0xA6; 32]);
        let mut provider2 = sub_randomness([0xCF; 32]);
        assert_ne!(provider1.provide(), provider2.provide());
        assert_ne!(provider1.provide(), provider2.provide());
        assert_ne!(provider1.provide(), provider2.provide());

        // outputs are the same for the same as sub_randomness_with_key with "_^default^_"
        let mut provider1 = sub_randomness([0xA6; 32]);
        let mut provider2 = sub_randomness_with_key([0xA6; 32], "_^default^_");
        assert_eq!(provider1.provide(), provider2.provide());
        assert_eq!(provider1.provide(), provider2.provide());
        assert_eq!(provider1.provide(), provider2.provide());
    }

    #[test]
    fn sub_randomness_implements_iterator() {
        let randomness: [u8; 32] = [0x77; 32];
        let mut provider = sub_randomness(randomness);
        let v1 = provider.next().unwrap();
        let v2 = provider.next().unwrap();
        let v3 = provider.next().unwrap();
        let v4 = provider.next().unwrap();
        println!("v1 = {v1:?}");
        println!("v2 = {v2:?}");
        println!("v3 = {v3:?}");
        println!("v4 = {v4:?}");
    }
    #[test]
    fn coinflip_distribution_is_uniform() {
        /// This test will generate a huge amount  of subrandomness
        /// and throws a coin with every subrandomness
        /// then checks that the distribution is expected within a range of 1%
        use crate::sub_randomness::sub_randomness;
        use std::collections::HashMap;

        const TEST_SAMPLE_SIZE: usize = 100_000;
        const ACCURACY: f32 = 0.01;

        let mut result = vec![];

        let mut provider = sub_randomness(RANDOMNESS1);

        for _ in 0..TEST_SAMPLE_SIZE {
            let flip_is_heads = coinflip(provider.next().unwrap()).is_heads();
            println!("{}", flip_is_heads);
            result.push(flip_is_heads);
        }

        let mut histogram = HashMap::new();

        for element in result {
            let count = histogram.entry(element).or_insert(0);
            *count += 1;
        }

        let estimated_count_for_uniform_distribution = (TEST_SAMPLE_SIZE / 2) as f32;
        let estimation_min: i32 =
            (estimated_count_for_uniform_distribution * (1_f32 - ACCURACY)) as i32;
        let estimation_max: i32 =
            (estimated_count_for_uniform_distribution * (1_f32 + ACCURACY)) as i32;
        println!(
            "estimation {}, max: {}, min: {}",
            estimated_count_for_uniform_distribution, estimation_max, estimation_min
        );
        // This will assert on all the elements of the data 1 by 1 and check if their occurence is within the 1% expected range
        for (bin, count) in histogram {
            println!("{}: {}", bin, count);
            assert!(count >= estimation_min && count <= estimation_max);
        }
    }

    #[test]
    fn pick_distribution_is_uniform() {
        /// This test will generate a huge amount  of subrandomness and picks n elements from the list
        /// It will then test that the outcome of every possibility within the picked value falls with 1% close
        /// To what it should be in a uniform distribution
        /// For this test to work properly for a 10 element size data consider choosing a TEST_SAMPLE_SIZE higher than 100_000
        use crate::sub_randomness::sub_randomness;
        use std::collections::HashMap;

        const TEST_SAMPLE_SIZE: usize = 300_000;
        const N_PICKED_ELEMENTS: usize = 3;
        const ACCURACY: f32 = 0.01;

        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];

        let mut result = vec![vec![]];

        let mut provider = sub_randomness(RANDOMNESS1);

        for _ in 0..TEST_SAMPLE_SIZE - 1 {
            let pick_result = pick(provider.next().unwrap(), N_PICKED_ELEMENTS, data.clone());

            result.push(pick_result);
        }

        let mut histogram = HashMap::new();

        for row in result {
            for element in row {
                let count = histogram.entry(element).or_insert(0);
                *count += 1;
            }
        }
        let estimated_count_for_uniform_distribution =
            (TEST_SAMPLE_SIZE * N_PICKED_ELEMENTS / data.len()) as f32;
        let estimation_min: i32 =
            (estimated_count_for_uniform_distribution * (1_f32 - ACCURACY)) as i32;
        let estimation_max: i32 =
            (estimated_count_for_uniform_distribution * (1_f32 + ACCURACY)) as i32;
        println!(
            "estimation {}, max: {}, min: {}",
            estimated_count_for_uniform_distribution, estimation_max, estimation_min
        );
        // This will assert on all the elements of the data 1 by 1 and check if their occurence is within the 1% expected range
        for (bin, count) in histogram {
            println!("{}: {}", bin, count);
            assert!(count >= estimation_min && count <= estimation_max);
        }
    }

    #[test]
    fn counter_sub_randomness() {
        let mut seed = [0u8; 32];
        for i in 0..100 as u64 {
            seed[24..].copy_from_slice(&i.to_be_bytes());
            println!("-------------------------------------------------------");
            println!("seed: {}", HexBinary::try_from(seed).unwrap());
            let mut provider = sub_randomness(seed);
            let subr = provider.provide();
            println!("subr: {}", HexBinary::try_from(subr).unwrap());
        }
        panic!();
    }
}
