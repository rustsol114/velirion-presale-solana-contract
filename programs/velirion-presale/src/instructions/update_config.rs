use anchor_lang::prelude::*;
use crate::state::*;
use crate::constants::*;
use crate::error::PresaleError;

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [PRESALE_CONFIG_SEED],
        bump = presale_config.bump,
        constraint = presale_config.authority == authority.key() @ PresaleError::Unauthorized
    )]
    pub presale_config: Account<'info, PresaleConfig>,
}

pub fn handler(
    ctx: Context<UpdateConfig>,
    max_purchase_per_transaction: Option<u64>,
    max_purchase_per_wallet: Option<u64>,
    min_time_between_purchases: Option<i64>,
) -> Result<()> {
    let presale_config = &mut ctx.accounts.presale_config;
    
    if let Some(max_tx) = max_purchase_per_transaction {
        presale_config.max_purchase_per_transaction = max_tx;
    }
    
    if let Some(max_wallet) = max_purchase_per_wallet {
        presale_config.max_purchase_per_wallet = max_wallet;
    }
    
    if let Some(min_time) = min_time_between_purchases {
        require!(min_time >= 0, PresaleError::InvalidPhaseConfig);
        presale_config.min_time_between_purchases = min_time;
    }
    
    msg!("Presale config updated");
    Ok(())
}

