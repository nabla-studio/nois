use rand::Rng;

use crate::prng::make_prng;
use std::{collections::HashMap, fmt::Debug};

/// Picks `n` elements from a given list.
///
/// This consumes the vector of elements for efficientcy reasons. Applications that do
/// not need the original data anymore benefit from an efficient in-place implementation.
///
/// ## Examples
///
/// Pick 6 out of 49:
///
/// ```
/// use nois::{randomness_from_str, pick};
///
/// let randomness = randomness_from_str("9e8e26615f51552aa3b18b6f0bcf0dae5afbe30321e8d7ea7fa51ebeb1d8fe62").unwrap();
///
/// // We are randomly shuffling a vector of integers [1,2,3,4]
/// let data = vec![
///   1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11,
///   12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22,
///   23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33,
///   34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44,
///   45, 46, 47, 48, 49
/// ];
/// let picked = pick(randomness, 6, data);
/// // The length of the vector is the same but the order of the elements has changed
/// assert_eq!(picked.len(), 6);
/// assert_eq!(picked, vec![7, 33, 18, 22, 8, 10]);
/// ```
///
/// Pick two winners from a vector of strings:
///
/// ```
/// use nois::{randomness_from_str, pick};
///
/// let randomness = randomness_from_str("9e8e26615f51552aa3b18b6f0bcf0dae5afbe30321e8d7ea7fa51ebeb1d8fe62").unwrap();
///
/// let data = vec!["bob".to_string(), "mary".to_string(), "su".to_string(), "marc".to_string()];
/// let picked = pick(randomness, 2, data);
/// // The length of the vector is the same but the order of the elements has changed
/// assert_eq!(picked.len(), 2);
/// assert_eq!(picked, vec!["su".to_string(), "bob".to_string()]);
/// ```
pub fn pick_old<T>(randomness: [u8; 32], n: usize, mut data: Vec<T>) -> Vec<T> {
    if n > data.len() {
        panic!("attempt to pick more elements than the input length");
    }
    let mut rng = make_prng(randomness);
    for i in ((data.len() - n)..data.len()).rev() {
        let j = rng.gen_range(0..=i);
        data.swap(i, j);
    }

    // Get last n elements
    data.split_off(data.len() - n)
}

pub fn pick<T: Debug + Clone>(randomness: [u8; 32], n: usize, data: &Vec<T>) -> Vec<T> {
    if n > data.len() {
        panic!("attempt to pick more elements than the input length");
    }

    let mut rng = make_prng(randomness);
    let mut swap_map = HashMap::new();
    let mut picked = Vec::with_capacity(n);

    let start = data.len();
    let end = start - n;
    println!("Start: {:?}", start);
    println!("end: {:?}", end);

    for pointer_index in (end..start).rev() {
        println!("------------");
        println!("pointer_index: {}", pointer_index);

        let random_index = rng.gen_range(0..=pointer_index);
        println!("random_index: {}", random_index);
        println!("swap_map: {:?}", swap_map);

        let chosen_index = *swap_map.get(&random_index).unwrap_or(&random_index);
        println!("chosen_index: {}", chosen_index);

        let picked_value = data[chosen_index].clone(); // Clone the value
        picked.push(picked_value);
        println!("Picked Values: {:?}", picked);

        // Set swap mapping for random_index to pointer_index
        swap_map.insert(
            random_index,
            *swap_map.get(&random_index).unwrap_or(&pointer_index),
        );
        println!("Insert - swap_map: {:?}", swap_map);
        // Remove pointer_index from swap_map
        swap_map.remove(&pointer_index);
        println!("Remove - swap_map: {:?}", swap_map);

        // Print
    }

    picked
}

#[cfg(test)]
mod tests {
    use crate::{shuffle, RANDOMNESS1, RANDOMNESS2};

    use super::*;

    #[test]
    fn pick_works() {
        let data: Vec<i32> = vec![];
        let picked = pick(RANDOMNESS1, 0, &data);
        assert_eq!(picked, Vec::<i32>::new());

        let data = vec![5];
        let picked = pick(RANDOMNESS1, 1, &data);
        assert_eq!(picked, vec![5]);

        let data = vec![1, 2, 3, 4];
        let picked = pick(RANDOMNESS1, 3, &data);
        assert_eq!(picked.len(), 3);
        assert_ne!(picked, vec![2, 3, 4]);

        // Element type is neither Copy nor Clone, i.e. the result is moved ot of the input data.
        #[derive(PartialEq, Debug, Clone)]
        struct Continent(String);
        let data = vec![
            Continent("Africa".into()),
            Continent("America".to_string()),
            Continent("Antarctica".to_string()),
            Continent("Australia".to_string()),
            Continent("Eurasia".to_string()),
        ];
        let picked = pick(RANDOMNESS1, 2, &data);
        assert_eq!(picked.len(), 2);
        assert_eq!(
            picked,
            vec![Continent("Eurasia".into()), Continent("America".into())]
        );
    }

    #[test]
    #[should_panic = "attempt to pick more elements than the input length"]
    fn pick_panicks_for_n_greater_than_len() {
        let data = vec![1, 2, 3, 4];
        let _picked = pick(RANDOMNESS1, 5, &data);
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

        for subrand in sub_randomness(RANDOMNESS1).take(TEST_SAMPLE_SIZE) {
            result.push(pick(subrand, N_PICKED_ELEMENTS, &data));
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
    fn pick_all_performs_full_shuffle_works() {
        let data = vec![0, 1, 2, 3, 4, 5];
        let picked = pick(RANDOMNESS2, data.len(), &data);
        let shuffled = shuffle(RANDOMNESS2, data);
        assert_eq!(picked, shuffled);

        let data = vec!["return", "if", "break", "match", "mut", "let"];
        let picked = pick(RANDOMNESS1, data.len(), &data);
        let shuffled = shuffle(RANDOMNESS1, data);
        assert_eq!(picked, shuffled);

        let data = Vec::<u32>::new();
        let picked = pick(RANDOMNESS1, data.len(), &data);
        let shuffled = shuffle(RANDOMNESS1, data);
        assert_eq!(picked, shuffled);

        let data = vec![true, false];
        let picked = pick(RANDOMNESS1, data.len(), &data);
        let shuffled = shuffle(RANDOMNESS1, data);
        assert_eq!(picked, shuffled);

        let data = vec![()];
        let picked = pick(RANDOMNESS1, data.len(), &data);
        let shuffled = shuffle(RANDOMNESS1, data);
        assert_eq!(picked, shuffled);
    }
}
