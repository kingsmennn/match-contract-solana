use anchor_lang::prelude::*;

#[constant]
pub const USER_TAG: &[u8] = b"USER_STATE";

#[constant]
pub const ADMIN_TAG: &[u8] = b"ADMIN_TAG";

#[constant]
pub const TIME_TO_LOCK: u64 = 900;
