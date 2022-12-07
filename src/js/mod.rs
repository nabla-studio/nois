#![cfg(feature = "js")]
///! This module contains a wrapper for this library for JavaScript.
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[allow(dead_code)] // exported via wasm_bindgen
pub fn coinflip(randomness: &str) -> Result<String, JsValue> {
    Ok(implementations::coinflip_impl(randomness)?)
}

// Returns a value from 1 to 6 (inclusive)
#[wasm_bindgen]
#[allow(dead_code)] // exported via wasm_bindgen
pub fn roll_dice(randomness: &str) -> Result<u8, JsValue> {
    Ok(implementations::roll_dice_impl(randomness)?)
}

/// Returns an integer between begin (inclusive) and end (exclusive).
///
/// Both bounds must be in the uint32 range.
#[wasm_bindgen]
#[allow(dead_code)] // exported via wasm_bindgen
pub fn int_in_range(randomness: &str, begin: u32, end: u32) -> Result<u32, JsValue> {
    Ok(implementations::int_in_range_impl(randomness, begin, end)?)
}

/// Returns a Decimal d with 0 <= d < 1.
/// The Decimal is in string representation and has 18 decimal digits.
#[wasm_bindgen]
#[allow(dead_code)] // exported via wasm_bindgen
pub fn random_decimal(randomness: &str) -> Result<String, JsValue> {
    Ok(implementations::random_decimal_impl(randomness)?.to_string())
}

/// Returns sub-randomness that is derives from the given randomness.
#[wasm_bindgen]
#[allow(dead_code)] // exported via wasm_bindgen
pub fn sub_randomness(randomness: &str, count: u32) -> Result<Box<[JsValue]>, JsValue> {
    let strings = implementations::sub_randomness_impl(randomness, count)?;
    Ok(strings
        .into_iter()
        .map(|s| JsValue::from_str(&s))
        .collect::<Vec<_>>()
        .into_boxed_slice())
}

mod implementations {
    use crate::{coinflip, int_in_range, random_decimal, roll_dice, sub_randomness};
    use cosmwasm_std::Decimal;

    pub struct JsError(String);

    impl From<hex::FromHexError> for JsError {
        fn from(source: hex::FromHexError) -> Self {
            Self(source.to_string())
        }
    }

    impl From<JsError> for wasm_bindgen::JsValue {
        fn from(source: JsError) -> wasm_bindgen::JsValue {
            wasm_bindgen::JsValue::from_str(&source.0)
        }
    }

    fn cast_vec_to_array(data: Vec<u8>) -> Result<[u8; 32], JsError> {
        let len = data.len();
        data.try_into().map_err(|_e| {
            let msg = format!("Expected a randomness of length 32 bytes (64 hex characters) but got {} ({} hex characters)", len, 2*len);
            JsError(msg)
        })
    }

    pub fn coinflip_impl(randomness_hex: &str) -> Result<String, JsError> {
        let randomness = hex::decode(randomness_hex)?;
        let randomness_array = cast_vec_to_array(randomness)?;
        let side = coinflip(randomness_array);
        Ok(side.to_string())
    }

    pub fn roll_dice_impl(randomness_hex: &str) -> Result<u8, JsError> {
        let randomness = hex::decode(randomness_hex)?;
        let randomness_array = cast_vec_to_array(randomness)?;
        Ok(roll_dice(randomness_array))
    }

    pub fn int_in_range_impl(randomness_hex: &str, begin: u32, end: u32) -> Result<u32, JsError> {
        let randomness = hex::decode(randomness_hex)?;
        let randomness_array = cast_vec_to_array(randomness)?;
        Ok(int_in_range(randomness_array, begin..end))
    }

    pub fn random_decimal_impl(randomness_hex: &str) -> Result<Decimal, JsError> {
        let randomness = hex::decode(randomness_hex)?;
        let randomness_array = cast_vec_to_array(randomness)?;
        Ok(random_decimal(randomness_array))
    }

    pub fn sub_randomness_impl(randomness_hex: &str, count: u32) -> Result<Vec<String>, JsError> {
        let randomness = hex::decode(randomness_hex)?;
        let randomness_array = cast_vec_to_array(randomness)?;
        let count = count as usize;
        let mut out = Vec::with_capacity(count);
        for sub_randomness in sub_randomness(randomness_array).take(count) {
            out.push(hex::encode(sub_randomness));
        }
        Ok(out)
    }
}