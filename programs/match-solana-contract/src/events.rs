use anchor_lang::prelude::*;

#[event]
pub struct StoreCreated {
    pub seller_address: Pubkey,
    pub store_id: u64,
    pub store_name: String,
    pub latitude: i128,
    pub longitude: i128,
}

#[event]
pub struct RequestCreated {
    pub request_id: u64,
    pub buyer_address: Pubkey,
    pub request_name: String,
    pub latitude: i128,
    pub longitude: i128,
    pub images: Vec<String>,
    pub lifecycle: u8,
    pub description: String,
    pub buyer_id: u64,
    pub seller_ids: Vec<u64>,
    pub sellers_price_quote: u64,
    pub locked_seller_id: u64,
    pub created_at: u64,
    pub updated_at: u64,
}

#[event]
pub struct OfferCreated {
    pub offer_id: u64,
    pub seller_address: Pubkey,
    pub store_name: String,
    pub price: u64,
    pub request_id: u64,
    pub images: Vec<String>,
    pub seller_id: u64,
    pub seller_ids: Vec<u64>,
}

#[event]
pub struct RequestAccepted {
    pub request_id: u64,
    pub offer_id: u64,
    pub seller_id: u64,
    pub updated_at: u64,
    pub sellers_price_quote: u64,
}

#[event]
pub struct OfferAccepted {
    pub offer_id: u64,
    pub buyer_address: Pubkey,
    pub is_accepted: bool,
}
#[event]
pub struct LocationEnabled {
    pub user_id: u64,
    pub location_enabled: bool,
}
