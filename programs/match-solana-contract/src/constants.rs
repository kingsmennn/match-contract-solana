use anchor_lang::prelude::*;

#[constant]
pub const TIME_TO_LOCK: u64 = 60;

#[constant]
pub const USER_TAG: &[u8] = b"USER_STATE";

#[constant]
pub const ADMIN_TAG: &[u8] = b"ADMIN_TAG";

#[constant]
pub const STORE_TAG: &[u8] = b"STORE_STATE";

#[constant]
pub const REQUEST_TAG: &[u8] = b"REQUEST_STATE";

#[constant]
pub const REQUEST_PAYMENT_TAG: &[u8] = b"REQUEST_PAYMENT_STATE";

#[constant]
pub const LOCATION_PREFERENCE_TAG: &[u8] = b"LOCATION_PREFERENCE_STATE";

#[constant]
pub const OFFER_TAG: &[u8] = b"OFFER_STATE";

#[constant]
pub const USER_COUNTER: &[u8] = b"USER_COUNTER";

#[constant]
pub const STORE_COUNTER: &[u8] = b"STORE_COUNTER";

#[constant]
pub const REQUEST_COUNTER: &[u8] = b"REQUEST_COUNTER";

#[constant]
pub const REQUEST_PAYMENT_COUNTER: &[u8] = b"REQUEST_PAYMENT_COUNTER";

#[constant]
pub const OFFER_COUNTER: &[u8] = b"OFFER_COUNTER";
