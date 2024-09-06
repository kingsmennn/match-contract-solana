use anchor_lang::prelude::*;

#[constant]
pub const USER_TAG: &[u8] = b"USER_STATE";

#[constant]
pub const ADMIN_TAG: &[u8] = b"ADMIN_TAG";

#[constant]
pub const STORE_TAG: &[u8] = b"STORE_STATE";

#[constant]
pub const TIME_TO_LOCK: u64 = 900;

#[constant]
pub const REQUEST_TAG: &[u8] = b"REQUEST_STATE";

#[constant]
pub const OFFER_TAG: &[u8] = b"OFFER_STATE";
