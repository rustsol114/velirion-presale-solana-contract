use anchor_lang::prelude::*;
use crate::state::*;
use crate::constants::*;

#[derive(Accounts)]
pub struct GetPurchaseStatus<'info> {
    #[account(
        seeds = [PRESALE_CONFIG_SEED],
        bump = presale_config.bump
    )]
    pub presale_config: Account<'info, PresaleConfig>,
    
    /// CHECK: User wallet to check status for
    pub user_wallet: UncheckedAccount<'info>,
    
    #[account(
        seeds = [USER_PURCHASE_SEED, user_wallet.key().as_ref()],
        bump
    )]
    pub user_purchase: Account<'info, UserPurchase>,
}

pub fn handler(ctx: Context<GetPurchaseStatus>) -> Result<()> {
    let presale_config = &ctx.accounts.presale_config;
    let user_purchase = &ctx.accounts.user_purchase;
    let clock = Clock::get()?;
    
    // Get current phase
    let current_phase = presale_config.get_current_phase(clock.unix_timestamp);
    
    // Calculate claimable amount
    let claimable = user_purchase.get_claimable_amount(clock.unix_timestamp);
    
    // Calculate remaining allocation
    let remaining = user_purchase.get_remaining_allocation(presale_config.max_purchase_per_wallet);
    
    msg!("Purchase Status:");
    msg!("  Total Purchased: {}", user_purchase.total_purchased);
    msg!("  Total Spent SOL: {}", user_purchase.total_spent_sol);
    msg!("  Total Spent USDC: {}", user_purchase.total_spent_usdc);
    msg!("  Claimable Tokens: {}", claimable);
    msg!("  Remaining Allocation: {}", remaining);
    msg!("  Last Purchase Time: {}", user_purchase.last_purchase_time);
    msg!("  Current Phase: {:?}", current_phase);
    msg!("  Presale Paused: {}", presale_config.is_paused);
    
    Ok(())
}

