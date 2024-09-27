pub mod constants;
pub mod events;
pub mod states;
pub mod errors;
use anchor_lang::prelude::*;

declare_id!("EPDpaEoRGQbZHBG1wJkd4Vae44UPmTLMmDreLcjrkfAg");
use crate::{constants::*, events::*, states::*, errors::*};
// use solana_program::pubkey;
use std::mem::size_of;

// const ADMIN_PUBKEY: Pubkey = pubkey!("7iT5H86QPoNFjGt1X2cMEJot4mr5Ns4uzhLN3GJKQ5kk");
#[program]
pub mod marketplace {
    use super::*;

    pub fn initialize_counters(ctx: Context<InitializeCounters>) -> Result<()> {
        let user_counter = &mut ctx.accounts.user_counter;
        let store_counter = &mut ctx.accounts.store_counter;
        let request_counter = &mut ctx.accounts.request_counter;
        let offer_counter = &mut ctx.accounts.offer_counter;
    
        // Initialize counters to one
        user_counter.current = 1;
        store_counter.current = 1;
        request_counter.current = 1;
        offer_counter.current = 1;
    
        msg!("Counters initialized: Users, Stores, Requests, Offers");
        
        Ok(())
    }
    

    pub fn create_user(
        ctx: Context<CreateUser>,
        username: String,
        phone: String,
        latitude: i128,
        longitude: i128,
        account_type: AccountType,
    ) -> Result<()> {
        let user = &mut ctx.accounts.user;
        let user_counter = &mut ctx.accounts.user_counter;

        if user.id != 0 {
            return err!(MarketplaceError::UserAlreadyExists);
        }

        if account_type != AccountType::Buyer && account_type != AccountType::Seller {
            return err!(MarketplaceError::InvalidAccountType);
        }
     

        user.id = user_counter.current;

        user_counter.current = user_counter.current.checked_add(1).unwrap();

        user.username = username;
        user.phone = phone;
        user.location = Location {
            latitude,
            longitude,
        };
        user.account_type = account_type;
        user.created_at = Clock::get().unwrap().unix_timestamp;
        user.updated_at = Clock::get().unwrap().unix_timestamp;
        user.authority = ctx.accounts.authority.key();
        user.location_enabled = true;

        msg!("UserCreated: {:?}", user);

        Ok(())
    }

    pub fn update_user(
        ctx: Context<UpdateUser>,
        username: String,
        phone: String,
        latitude: i128,
        longitude: i128,
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
        user.authority = ctx.accounts.authority.key();

        msg!("UserUpdated: {:?}", user);

        Ok(())
    }


    pub fn create_store(
        ctx: Context<CreateStore>,
        name: String,
        description: String,
        phone: String,
        latitude: i128,
        longitude: i128
    ) -> Result<()> {
        let user = &mut ctx.accounts.user;
        let store_counter = &mut ctx.accounts.store_counter;

        if user.account_type != AccountType::Seller {
            return err!(MarketplaceError::OnlySellersAllowed);
        }

        let store = &mut ctx.accounts.store;

        store.id = store_counter.current;
        store.name = name;
        store.description = description;
        store.phone = phone;
        store.location = Location {
            latitude,
            longitude,
        };
        store.authority = ctx.accounts.authority.key();

        store_counter.current = store_counter.current.checked_add(1).unwrap();

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
        latitude: i128,
        longitude: i128,
    ) -> Result<()> {
        let user = &mut ctx.accounts.user;
        let request_counter = &mut ctx.accounts.request_counter;

        if user.account_type != AccountType::Buyer {
            return err!(MarketplaceError::OnlyBuyersAllowed);
        }

        let request = &mut ctx.accounts.request;

        request.id = request_counter.current;
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
        request.authority = ctx.accounts.authority.key();

        request_counter.current = request_counter.current.checked_add(1).unwrap();

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

    pub fn delete_request(ctx: Context<RemoveRequest>) -> Result<()> {
        let request = &mut ctx.accounts.request;
        let authority = &ctx.accounts.authority;
    
        if request.authority != authority.key() {
            return err!(MarketplaceError::InvalidUser);
        }
    
        if request.lifecycle != RequestLifecycle::Pending {
            return err!(MarketplaceError::RequestLocked);
        }
    
        Ok(())
    }

    pub fn mark_request_as_completed(ctx: Context<MarkAsCompleteRequest>) -> Result<()> {
        let request = &mut ctx.accounts.request;
        let authority = &ctx.accounts.authority;
    
        if request.authority != authority.key() {
            return err!(MarketplaceError::InvalidUser);
        }
    
        if request.lifecycle != RequestLifecycle::AcceptedByBuyer {
            return err!(MarketplaceError::RequestNotAccepted);
        }

        if request.updated_at + TIME_TO_LOCK  > Clock::get().unwrap().unix_timestamp as u64 {
            return err!(MarketplaceError::RequestNotLocked);
        }
    
        request.lifecycle = RequestLifecycle::Completed;
        request.updated_at = Clock::get().unwrap().unix_timestamp as u64;
    
        Ok(())
    }

    pub fn toggle_location(ctx: Context<ToggleLocation>, enabled: bool) -> Result<()> {
        let  user  = &mut ctx.accounts.user;

        user.location_enabled = enabled;

        emit!(LocationEnabled {
            location_enabled: enabled,
            user_id: user.id,
        });
        
        Ok(())
    }

    pub fn get_location_preference(ctx: Context<GetLocationPreference>) -> Result<bool> {
        let user = &ctx.accounts.user;
        
        Ok(user.location_enabled)
    }
    


    pub fn create_offer(
        ctx: Context<CreateOffer>,
        price: i64,
        images: Vec<String>,
        store_name: String,
    ) -> Result<()> {
        let user = &mut ctx.accounts.user;
        let offer_counter = &mut ctx.accounts.offer_counter;

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

        offer.id = offer_counter.current;
        offer.price = price;
        offer.images = images;
        offer.request_id = request.id;
        offer.store_name = store_name;
        offer.seller_id = user.id;
        offer.is_accepted = false;
        offer.created_at = Clock::get().unwrap().unix_timestamp as u64;
        offer.updated_at = Clock::get().unwrap().unix_timestamp as u64;
        offer.authority = ctx.accounts.authority.key();

        if request.lifecycle == RequestLifecycle::Pending {
            request.lifecycle = RequestLifecycle::AcceptedBySeller;
        }

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

        offer_counter.current = offer_counter.current.checked_add(1).unwrap();

        Ok(())
    }

    pub fn accept_offer(ctx: Context<AcceptOffer>) -> Result<()> {
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


        if request.seller_ids.len() != ctx.remaining_accounts.len() {
            return err!(MarketplaceError::IncorrectNumberOfSellers);
        }

        for account_info in ctx.remaining_accounts.iter() {
            let mut data = account_info.try_borrow_mut_data()?;

            let mut previous_offer = Offer::try_deserialize(&mut data.as_ref()).expect("Error Deserializing Data");
            
            if previous_offer.is_accepted  && previous_offer.request_id == request.id {
                previous_offer.is_accepted = false;
                previous_offer.try_serialize(&mut data.as_mut())?;
                emit!(OfferAccepted {
                    offer_id: previous_offer.id,
                    buyer_address: *ctx.accounts.user.to_account_info().key,
                    is_accepted: false,
                });
            }
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
}

#[derive(Accounts)]
#[instruction()]
pub struct CreateUser<'info> {
    #[account(
        init,
        seeds = [USER_TAG,authority.key.as_ref()],
        bump,
        payer = authority,
        space = 8 + size_of::<User>())]
    pub user: Box<Account<'info, User>>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [USER_COUNTER],
        bump,
    )]
    pub user_counter: Box<Account<'info, Counter>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction()]
pub struct UpdateUser<'info> {
    #[account(
        mut,
        seeds = [USER_TAG,authority.key().as_ref()],
        bump,
        has_one = authority
    )]
    pub user: Box<Account<'info, User>>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
#[instruction()]
pub struct CreateStore<'info> {
    #[account(
        mut,
        seeds = [USER_TAG,authority.key().as_ref()],
        bump,
        has_one = authority
    )]
    pub user: Box<Account<'info, User>>,
    #[account(init, payer = authority ,space = 8 + size_of::<Store>(),        
    seeds = [STORE_TAG, authority.key().as_ref(),&store_counter.current.to_le_bytes()],
    bump,)]
    pub store: Box<Account<'info, Store>>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [STORE_COUNTER],
        bump,
    )]
    pub store_counter: Box<Account<'info, Counter>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction()]
pub struct CreateRequest<'info> {
    #[account(
        mut,
        seeds = [USER_TAG,authority.key().as_ref()],
        bump,
        has_one = authority
    )]
    pub user: Box<Account<'info, User>>,
    #[account(init, payer = authority ,space = 8 + size_of::<Request>() + 1024,        
    seeds = [REQUEST_TAG, authority.key().as_ref(),&request_counter.current.to_le_bytes()],
    bump,)]
    pub request: Box<Account<'info, Request>>,
    #[account(
        mut,
        seeds = [REQUEST_COUNTER],
        bump,
    )]
    pub request_counter: Box<Account<'info, Counter>>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RemoveRequest<'info> {
    #[account(
        mut,
        has_one = authority,
        seeds = [REQUEST_TAG, authority.key().as_ref(), &request.id.to_le_bytes()],
        bump,
        close = authority
    )]
    pub request: Box<Account<'info, Request>>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ToggleLocation<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [USER_TAG,authority.key().as_ref()],
        bump,
        has_one = authority
    )]
    pub user: Box<Account<'info, User>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct GetLocationPreference<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [USER_TAG,authority.key().as_ref()],
        bump,
        has_one = authority
    )]
    pub user: Box<Account<'info, User>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MarkAsCompleteRequest<'info> {
    #[account(
        mut,
        has_one = authority,
        seeds = [REQUEST_TAG, authority.key().as_ref(), &request.id.to_le_bytes()],
        bump,
    )]
    pub request: Box<Account<'info, Request>>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction()]
pub struct CreateOffer<'info> {
    #[account(
        mut,
        seeds = [USER_TAG,authority.key().as_ref()],
        bump,
        has_one = authority
    )]
    pub user: Box<Account<'info, User>>,
    #[account(mut)]
    pub request: Box<Account<'info, Request>>,
    #[account(init, payer = authority ,space = 8 + size_of::<Offer>() + 1024,     
    seeds = [OFFER_TAG, authority.key().as_ref(),&offer_counter.current.to_le_bytes()],
    bump,)]
    pub offer: Box<Account<'info, Offer>>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [OFFER_COUNTER],
        bump,
    )]
    pub offer_counter: Box<Account<'info, Counter>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction()]
pub struct AcceptOffer<'info> {
    #[account(
        mut,
        seeds = [USER_TAG,authority.key().as_ref()],
        bump,
        has_one = authority
    )]
    pub user: Box<Account<'info, User>>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub offer: Box<Account<'info, Offer>>,
    #[account(mut)]
    pub request: Box<Account<'info, Request>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeCounters<'info> {
    #[account(
        init,
        seeds = [USER_COUNTER],
        bump,
        payer = authority,
        space = 8 + size_of::<Counter>()
    )]
    pub user_counter: Box<Account<'info, Counter>>,

    #[account(
        init,
        seeds = [STORE_COUNTER],
        bump,
        payer = authority,
        space = 8 + size_of::<Counter>()
    )]
    pub store_counter: Box<Account<'info, Counter>>,

    #[account(
        init,
        seeds = [REQUEST_COUNTER],
        bump,
        payer = authority,
        space = 8 + size_of::<Counter>()
    )]
    pub request_counter: Box<Account<'info, Counter>>,

    #[account(
        init,
        seeds = [OFFER_COUNTER],
        bump,
        payer = authority,
        space = 8 + size_of::<Counter>()
    )]
    pub offer_counter: Box<Account<'info, Counter>>,

    #[account(mut)]
    // #[account(mut, address = ADMIN_PUBKEY)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}
