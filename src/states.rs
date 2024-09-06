use anchor_lang::prelude::*;
#[account]
#[derive(Debug)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub phone: String,
    pub location: Location,
    pub created_at: i64,
    pub updated_at: i64,
    pub account_type: AccountType,
    pub authority: Pubkey,
    pub store_id: u64,
}

#[account]
pub struct Store {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub phone: String,
    pub location: Location,
    pub authority: Pubkey,
}

#[account]
pub struct Request {
    pub id: u64,
    pub name: String,
    pub buyer_id: u64,
    pub description: String,
    pub images: Vec<String>,
    pub sellers_price_quote: i64,
    pub seller_ids: Vec<u64>,
    pub offer_ids: Vec<u64>,
    pub locked_seller_id: u64,
    pub location: Location,
    pub created_at: u64,
    pub updated_at: u64,
    pub lifecycle: RequestLifecycle,
}

#[account]
pub struct Offer {
    pub id: u64,
    pub request_id: u64,
    pub price: i64,
    pub images: Vec<String>,
    pub store_name: String,
    pub seller_id: u64,
    pub is_accepted: bool,
    pub created_at: u64,
    pub updated_at: u64,
    pub authority: Pubkey,
}

#[account]
pub struct Counter {
    pub current: u64,
}
#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Location {
    pub latitude: i64,
    pub longitude: i64,
}

#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum AccountType {
    Buyer,
    Seller,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum RequestLifecycle {
    Pending = 0,
    AcceptedByBuyer = 1,
    Completed = 2,
}

impl Default for RequestLifecycle {
    fn default() -> Self {
        RequestLifecycle::Pending
    }
}
