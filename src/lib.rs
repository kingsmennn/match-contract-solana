use anchor_lang::prelude::*;

declare_id!("6VxaVo2xoWwz1jsU3SGBNRaBigEG8mcwm8mBLMP7Nha7");

#[program]
pub mod marketplace {
    use super::*;

    pub fn create_user(
        ctx: Context<CreateUser>,
        username: String,
        phone: String,
        latitude: i64,
        longitude: i64,
        account_type: AccountType,
    ) -> Result<()> {
        let user = &mut ctx.accounts.user;

        if user.id != 0 {
            return err!(MarketplaceError::UserAlreadyExists);
        }

        if account_type != AccountType::Buyer && account_type != AccountType::Seller {
            return err!(MarketplaceError::InvalidAccountType);
        }

        user.id = get_next_user_id();
        user.username = username;
        user.phone = phone;
        user.location = Location {
            latitude,
            longitude,
        };
        user.account_type = account_type;
        user.created_at = Clock::get().unwrap().unix_timestamp;
        user.updated_at = Clock::get().unwrap().unix_timestamp;

        msg!("UserCreated: {:?}", user);

        Ok(())
    }

    pub fn update_user(
        ctx: Context<UpdateUser>,
        username: String,
        phone: String,
        latitude: i64,
        longitude: i64,
        account_type: AccountType,
    ) -> Result<()> {
        let user = &mut ctx.accounts.user;

        if user.id == 0 {
            return err!(MarketplaceError::InvalidUser);
        }

        user.username = username;
        user.phone = phone;
        user.location = Location {
            latitude,
            longitude,
        };
        user.updated_at = Clock::get().unwrap().unix_timestamp;
        user.account_type = account_type;

        msg!("UserUpdated: {:?}", user);

        Ok(())
    }

    pub fn create_store(
        ctx: Context<CreateStore>,
        name: String,
        description: String,
        phone: String,
        latitude: i64,
        longitude: i64,
    ) -> Result<()> {
        let user = &mut ctx.accounts.user;

        if user.account_type != AccountType::Seller {
            return err!(MarketplaceError::OnlySellersAllowed);
        }

        let store = &mut ctx.accounts.store;

        store.id = ctx.accounts.store_counter.current;
        store.name = name;
        store.description = description;
        store.phone = phone;
        store.location = Location {
            latitude,
            longitude,
        };

        ctx.accounts.store_counter.current += 1;

        emit!(StoreCreated {
            seller_address: *ctx.accounts.user.to_account_info().key,
            store_id: store.id,
            store_name: store.name.clone(),
            latitude: store.location.latitude,
            longitude: store.location.longitude,
        });

        Ok(())
    }

    pub fn create_request(
        ctx: Context<CreateRequest>,
        name: String,
        description: String,
        images: Vec<String>,
        latitude: i64,
        longitude: i64,
    ) -> Result<()> {
        let user = &mut ctx.accounts.user;

        if user.account_type != AccountType::Buyer {
            return err!(MarketplaceError::OnlyBuyersAllowed);
        }

        let request = &mut ctx.accounts.request;

        request.id = ctx.accounts.request_counter.current;
        request.name = name;
        request.buyer_id = user.id;
        request.sellers_price_quote = 0;
        request.seller_ids = Vec::new();
        request.offer_ids = Vec::new();
        request.locked_seller_id = 0;
        request.description = description;
        request.images = images;
        request.created_at = Clock::get().unwrap().unix_timestamp as u64;
        request.lifecycle = RequestLifecycle::Pending;
        request.location = Location {
            latitude,
            longitude,
        };
        request.updated_at = Clock::get().unwrap().unix_timestamp as u64;

        ctx.accounts.request_counter.current += 1;

        emit!(RequestCreated {
            request_id: request.id,
            buyer_address: *ctx.accounts.user.to_account_info().key,
            request_name: request.name.clone(),
            latitude: request.location.latitude,
            longitude: request.location.longitude,
            images: request.images.clone(),
            lifecycle: request.lifecycle as u8,
            description: request.description.clone(),
            buyer_id: request.buyer_id,
            seller_ids: request.seller_ids.clone(),
            sellers_price_quote: request.sellers_price_quote,
            locked_seller_id: request.locked_seller_id,
            created_at: request.created_at,
            updated_at: request.updated_at,
        });

        Ok(())
    }

    pub fn create_offer(
        ctx: Context<CreateOffer>,
        price: i64,
        images: Vec<String>,
        request_id: u64,
        store_name: String,
    ) -> Result<()> {
        let user = &mut ctx.accounts.user;

        if user.account_type != AccountType::Seller {
            return err!(MarketplaceError::OnlySellersAllowed);
        }

        let request = &mut ctx.accounts.request;

        if Clock::get().unwrap().unix_timestamp as u64 > request.updated_at + TIME_TO_LOCK
            && request.lifecycle == RequestLifecycle::AcceptedByBuyer
        {
            return err!(MarketplaceError::RequestLocked);
        }

        let offer = &mut ctx.accounts.offer;

        offer.id = ctx.accounts.offer_counter.current;
        offer.price = price;
        offer.images = images;
        offer.request_id = request.id;
        offer.store_name = store_name;
        offer.seller_id = user.id;
        offer.is_accepted = false;
        offer.created_at = Clock::get().unwrap().unix_timestamp as u64;
        offer.updated_at = Clock::get().unwrap().unix_timestamp as u64;

        request.seller_ids.push(offer.seller_id);

        emit!(OfferCreated {
            offer_id: offer.id,
            seller_address: *ctx.accounts.user.to_account_info().key,
            store_name: offer.store_name.clone(),
            price: offer.price,
            request_id: offer.request_id,
            images: offer.images.clone(),
            seller_id: offer.seller_id,
            seller_ids: request.seller_ids.clone(),
        });

        ctx.accounts.offer_counter.current += 1;

        Ok(())
    }

    pub fn accept_offer(ctx: Context<AcceptOffer>, offer_id: u64) -> Result<()> {
        let user = &mut ctx.accounts.user;
        let offer = &mut ctx.accounts.offer;
        let request = &mut ctx.accounts.request;

        if user.account_type != AccountType::Buyer {
            return err!(MarketplaceError::OnlyBuyersAllowed);
        }

        if request.buyer_id != user.id {
            return err!(MarketplaceError::UnauthorizedBuyer);
        }

        if offer.is_accepted {
            return err!(MarketplaceError::OfferAlreadyAccepted);
        }

        if Clock::get().unwrap().unix_timestamp as u64 > request.updated_at + TIME_TO_LOCK
            && request.lifecycle == RequestLifecycle::AcceptedByBuyer
        {
            return err!(MarketplaceError::RequestLocked);
        }

        for prev_offer_id in request.offer_ids.iter() {
            let previous_offer = &mut ctx.accounts.offers[*prev_offer_id as usize];
            previous_offer.is_accepted = false;
            emit!(OfferAccepted {
                offer_id: previous_offer.id,
                buyer_address: *ctx.accounts.user.to_account_info().key,
                is_accepted: false,
            });
        }

        offer.is_accepted = true;
        offer.updated_at = Clock::get().unwrap().unix_timestamp as u64;
        request.offer_ids.push(offer.id);
        request.locked_seller_id = offer.seller_id;
        request.sellers_price_quote = offer.price;
        request.lifecycle = RequestLifecycle::AcceptedByBuyer;
        request.updated_at = Clock::get().unwrap().unix_timestamp as u64;

        emit!(RequestAccepted {
            request_id: request.id,
            offer_id: offer.id,
            seller_id: offer.seller_id,
            updated_at: request.updated_at,
            sellers_price_quote: request.sellers_price_quote,
        });

        emit!(OfferAccepted {
            offer_id: offer.id,
            buyer_address: *ctx.accounts.user.to_account_info().key,
            is_accepted: true,
        });

        Ok(())
    }

    // Add other functions similarly...

    // Helper function to generate unique IDs (just a placeholder)
    fn get_next_user_id() -> u64 {
        // Implement your ID generation logic here
        1
    }
}

#[derive(Accounts)]
pub struct CreateUser<'info> {
    #[account(init, payer = user_signer, space = 8 +  std::mem::size_of<User>())]
    pub user: Account<'info, User>,
    #[account(mut)]
    pub user_signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateUser<'info> {
    #[account(mut)]
    pub user: Account<'info, User>,
    #[account(mut)]
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct CreateStore<'info> {
    #[account(mut)]
    pub user: Account<'info, User>,
    #[account(init, payer = user, space = 8 + std::mem::size_of<Store>())]
    pub store: Account<'info, Store>,
    #[account(mut)]
    pub store_counter: Account<'info, Counter>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateRequest<'info> {
    #[account(mut)]
    pub user: Account<'info, User>,
    #[account(init, payer = user, space = 8 + std::mem::size_of<Request>())]
    pub request: Account<'info, Request>,
    #[account(mut)]
    pub request_counter: Account<'info, Counter>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateOffer<'info> {
    #[account(mut)]
    pub user: Account<'info, User>,
    #[account(mut)]
    pub request: Account<'info, Request>,
    #[account(init, payer = user, space = 8 + std::mem::size_of<Offer>())]
    pub offer: Account<'info, Offer>,
    #[account(mut)]
    pub offer_counter: Account<'info, Counter>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AcceptOffer<'info> {
    #[account(mut)]
    pub user: Account<'info, User>,
    #[account(mut)]
    pub offer: Account<'info, Offer>,
    #[account(mut)]
    pub request: Account<'info, Request>,
    #[account(mut)]
    pub offers: Program<'info, Offer>, // For iterating previous offers
    pub system_program: Program<'info, System>,
}

// Define your account types here
#[account]
pub struct User {
    pub id: u64,
    pub username: String,
    pub phone: String,
    pub location: Location,
    pub created_at: i64,
    pub updated_at: i64,
    pub account_type: AccountType,
}

#[account]
pub struct Store {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub phone: String,
    pub location: Location,
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
}

#[account]
pub struct Counter {
    pub current: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum RequestLifecycle {
    Pending,
    AcceptedByBuyer,
    Completed,
}

impl Default for RequestLifecycle {
    fn default() -> Self {
        RequestLifecycle::Pending
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Location {
    pub latitude: i64,
    pub longitude: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum AccountType {
    Buyer,
    Seller,
}

// Define your custom errors
#[error_code]
pub enum MarketplaceError {
    #[msg("User already exists.")]
    UserAlreadyExists,
    #[msg("Invalid account type.")]
    InvalidAccountType,
    #[msg("Invalid user.")]
    InvalidUser,
    // Add other custom errors here...
}
