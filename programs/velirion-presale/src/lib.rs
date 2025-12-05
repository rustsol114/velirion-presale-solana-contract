pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use error::*;
pub use instructions::*;
pub use state::*;

declare_id!("91uNkK5URavMx6onv6c8XTZ6VkEj4RzA6Xa4pQydWs2s");

#[program]
pub mod velirion_presale {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        phases: [PresalePhase; 10],
        total_tokens_for_sale: u64,
        max_purchase_per_transaction: u64,
        max_purchase_per_wallet: u64,
        min_time_between_purchases: i64,
        launch_timestamp: i64,
        vesting_launch_percentage: u8,
        vesting_monthly_percentage: u8,
    ) -> Result<()> {
        super::instructions::initialize::handler(
            ctx,
            phases,
            total_tokens_for_sale,
            max_purchase_per_transaction,
            max_purchase_per_wallet,
            min_time_between_purchases,
            launch_timestamp,
            vesting_launch_percentage,
            vesting_monthly_percentage,
        )
    }

    pub fn purchase(
        ctx: Context<Purchase>,
        token_amount: u64,
        payment_type: PaymentType,
    ) -> Result<()> {
        super::instructions::purchase::handler(ctx, token_amount, payment_type)
    }

    pub fn claim_vested(ctx: Context<ClaimVested>) -> Result<()> {
        super::instructions::claim_vested::handler(ctx)
    }

    pub fn pause(ctx: Context<Pause>) -> Result<()> {
        super::instructions::pause::handler(ctx)
    }

    pub fn unpause(ctx: Context<Unpause>) -> Result<()> {
        super::instructions::pause::handler_unpause(ctx)
    }

    pub fn burn_unsold(ctx: Context<BurnUnsold>) -> Result<()> {
        super::instructions::burn_unsold::handler(ctx)
    }

    pub fn get_purchase_status(ctx: Context<GetPurchaseStatus>) -> Result<()> {
        super::instructions::get_status::handler(ctx)
    }

    pub fn update_config(
        ctx: Context<UpdateConfig>,
        max_purchase_per_transaction: Option<u64>,
        max_purchase_per_wallet: Option<u64>,
        min_time_between_purchases: Option<i64>,
    ) -> Result<()> {
        super::instructions::update_config::handler(
            ctx,
            max_purchase_per_transaction,
            max_purchase_per_wallet,
            min_time_between_purchases,
        )
    }
}
