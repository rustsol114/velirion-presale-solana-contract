use anchor_lang::prelude::*;
use anchor_spl::token::{Token, Transfer};
use crate::state::*;
use crate::constants::*;
use crate::error::PresaleError;

#[derive(Accounts)]
pub struct ClaimVested<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    
    #[account(
        seeds = [PRESALE_CONFIG_SEED],
        bump = presale_config.bump
    )]
    pub presale_config: Account<'info, PresaleConfig>,
    
    #[account(
        mut,
        seeds = [USER_PURCHASE_SEED, buyer.key().as_ref()],
        bump = user_purchase.bump
    )]
    pub user_purchase: Account<'info, UserPurchase>,
    
    /// CHECK: Validated in handler
    pub token_mint: UncheckedAccount<'info>,
    
    #[account(mut)]
    /// CHECK: Validated in handler
    pub buyer_token_account: UncheckedAccount<'info>,
    
    /// CHECK: Treasury account that holds the tokens
    #[account(mut)]
    /// CHECK: Validated in handler
    pub treasury: UncheckedAccount<'info>,
    
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<ClaimVested>) -> Result<()> {
    let user_purchase = &mut ctx.accounts.user_purchase;
    let clock = Clock::get()?;
    
    // Validate unchecked accounts
    let buyer_token_account_data = anchor_spl::token::TokenAccount::try_deserialize(&mut &ctx.accounts.buyer_token_account.data.borrow()[..])?;
    require!(
        buyer_token_account_data.owner == ctx.accounts.buyer.key(),
        PresaleError::Unauthorized
    );
    require!(
        buyer_token_account_data.mint == ctx.accounts.token_mint.key(),
        PresaleError::InvalidTokenMint
    );
    
    require!(
        ctx.accounts.treasury.key() == ctx.accounts.presale_config.treasury,
        PresaleError::InvalidTreasury
    );
    
    let claimable_amount = user_purchase.get_claimable_amount(clock.unix_timestamp);
    require!(claimable_amount > 0, PresaleError::NoTokensToClaim);
    
    // Mark vesting entries as claimed
    for entry in &mut user_purchase.vesting_schedule {
        if !entry.claimed 
            && clock.unix_timestamp >= entry.release_time 
            && entry.amount > 0 
        {
            entry.claimed = true;
        }
    }
    
    // Transfer tokens from treasury to buyer
    let bump = ctx.accounts.presale_config.bump;
    let seeds: &[&[u8]] = &[
        PRESALE_CONFIG_SEED,
        &[bump],
    ];
    let signer = &[&seeds[..]];
    
    let cpi_accounts = Transfer {
        from: ctx.accounts.treasury.to_account_info(),
        to: ctx.accounts.buyer_token_account.to_account_info(),
        authority: ctx.accounts.presale_config.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    anchor_spl::token::transfer(cpi_ctx, claimable_amount)?;
    
    msg!("Claimed {} vested tokens", claimable_amount);
    
    Ok(())
}

