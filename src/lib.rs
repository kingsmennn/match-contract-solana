pub mod constants;
pub mod events;
pub mod states;
use anchor_lang::prelude::*;

declare_id!("6VxaVo2xoWwz1jsU3SGBNRaBigEG8mcwm8mBLMP7Nha7");
use crate::{constants::*, events::*, states::*};

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
        let counter = &mut ctx.accounts.user_counter;

        if user.id != 0 {
            return err!(MarketplaceError::UserAlreadyExists);
        }

        if account_type != AccountType::Buyer && account_type != AccountType::Seller {
            return err!(MarketplaceError::InvalidAccountType);
        }

        user.id = counter.current;
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
            lifecycle: request.lifecycle.clone() as u8,
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

    // // Add other functions similarly...

    // // Helper function to generate unique IDs (just a placeholder)
}

#[derive(Accounts)]
#[instruction()]
pub struct CreateUser<'info> {
    #[account(init, payer = user_signer, space = 8 +  std::mem::size_of::<User>())]
    pub user: Box<Account<'info, User>>,
    #[account(mut)]
    pub user_signer: Signer<'info>,
    #[account(mut)]
    pub user_counter: Box<Account<'info, Counter>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction()]
pub struct UpdateUser<'info> {
    #[account(mut)]
    pub user: Box<Account<'info, User>>,
    #[account(mut)]
    pub signer: Signer<'info>,
}

#[derive(Accounts)]
#[instruction()]
pub struct CreateStore<'info> {
    #[account(mut)]
    pub user: Box<Account<'info, User>>,
    #[account(init, payer = user, space = 8 + std::mem::size_of::<Store>())]
    pub store: Box<Account<'info, Store>>,
    #[account(mut)]
    pub store_counter: Box<Account<'info, Counter>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction()]
pub struct CreateRequest<'info> {
    #[account(mut)]
    pub user: Box<Account<'info, User>>,
    #[account(init, payer = user, space = 8 + std::mem::size_of::<Request>())]
    pub request: Box<Account<'info, Request>>,
    #[account(mut)]
    pub request_counter: Box<Account<'info, Counter>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction()]
pub struct CreateOffer<'info> {
    #[account(mut)]
    pub user: Box<Account<'info, User>>,
    #[account(mut)]
    pub request: Box<Account<'info, Request>>,
    #[account(init, payer = user, space = 8 + std::mem::size_of::<Offer>())]
    pub offer: Box<Account<'info, Offer>>,
    #[account(mut)]
    pub offer_counter: Box<Account<'info, Counter>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction()]
pub struct AcceptOffer<'info> {
    #[account(mut)]
    pub user: Box<Account<'info, User>>,
    #[account(mut)]
    pub offer: Box<Account<'info, Offer>>,
    #[account(mut)]
    pub request: Box<Account<'info, Request>>,
    pub system_program: Program<'info, System>,
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
    // Add other custom errors here...
}
