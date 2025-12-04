use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint, Transfer};
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
    
    pub token_mint: Account<'info, Mint>,
    
    #[account(
        mut,
        constraint = buyer_token_account.owner == buyer.key() @ PresaleError::Unauthorized,
        constraint = buyer_token_account.mint == token_mint.key() @ PresaleError::InvalidTokenMint
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,
    
    /// CHECK: Treasury account that holds the tokens
    #[account(
        mut,
        constraint = treasury.key() == presale_config.treasury @ PresaleError::InvalidTreasury
    )]
    pub treasury: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<ClaimVested>) -> Result<()> {
    let user_purchase = &mut ctx.accounts.user_purchase;
    let clock = Clock::get()?;
    
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

