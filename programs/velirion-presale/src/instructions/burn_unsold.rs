use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint, Burn};
use crate::state::*;
use crate::constants::*;
use crate::error::PresaleError;

#[derive(Accounts)]
pub struct BurnUnsold<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        seeds = [PRESALE_CONFIG_SEED],
        bump = presale_config.bump,
        constraint = presale_config.authority == authority.key() @ PresaleError::Unauthorized
    )]
    pub presale_config: Account<'info, PresaleConfig>,
    
    pub token_mint: Account<'info, Mint>,
    
    /// CHECK: Treasury account that holds the unsold tokens
    #[account(
        mut,
        constraint = treasury.key() == presale_config.treasury @ PresaleError::InvalidTreasury,
        constraint = treasury.mint == token_mint.key() @ PresaleError::InvalidTokenMint
    )]
    pub treasury: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<BurnUnsold>) -> Result<()> {
    let presale_config = &ctx.accounts.presale_config;
    let clock = Clock::get()?;
    
    // Check if presale has ended (all phases ended)
    let last_phase = presale_config.phases[9];
    require!(
        clock.unix_timestamp > last_phase.end_time,
        PresaleError::PresaleNotEnded
    );
    
    // Calculate unsold tokens
    let unsold_tokens = presale_config
        .total_tokens_for_sale
        .checked_sub(presale_config.tokens_sold)
        .ok_or(PresaleError::MathOverflow)?;
    
    require!(unsold_tokens > 0, PresaleError::NoTokensToClaim);
    
    // Check treasury has enough tokens
    require!(
        ctx.accounts.treasury.amount >= unsold_tokens,
        PresaleError::InsufficientTokensInPhase
    );
    
    // Burn unsold tokens
    let bump = presale_config.bump;
    let seeds: &[&[u8]] = &[
        PRESALE_CONFIG_SEED,
        &[bump],
    ];
    let signer = &[&seeds[..]];
    
    let cpi_accounts = Burn {
        mint: ctx.accounts.token_mint.to_account_info(),
        from: ctx.accounts.treasury.to_account_info(),
        authority: ctx.accounts.presale_config.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    anchor_spl::token::burn(cpi_ctx, unsold_tokens)?;
    
    msg!("Burned {} unsold tokens", unsold_tokens);
    
    Ok(())
}

