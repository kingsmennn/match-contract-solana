use anchor_lang::prelude::*;

#[error_code]
pub enum MarketplaceError {
    #[msg("User already exists.")]
    UserAlreadyExists,
    #[msg("Invalid account type.")]
    InvalidAccountType,
    #[msg("Invalid user.")]
    InvalidUser,
    #[msg("Only sellers allowed.")]
    OnlySellersAllowed,
    #[msg("Only buyers allowed.")]
    OnlyBuyersAllowed,
    #[msg("Unauthorized buyer.")]
    UnauthorizedBuyer,
    #[msg("Offer already accepted.")]
    OfferAlreadyAccepted,
    #[msg("Request locked.")]
    RequestLocked,
    #[msg("Incorrect number of sellers.")]
    IncorrectNumberOfSellers,
}
