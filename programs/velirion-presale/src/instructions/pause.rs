use anchor_lang::prelude::*;
use crate::state::*;
use crate::constants::*;
use crate::error::PresaleError;

#[derive(Accounts)]
pub struct Pause<'info> {
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

pub fn handler(ctx: Context<Pause>) -> Result<()> {
    let presale_config = &mut ctx.accounts.presale_config;
    presale_config.is_paused = true;
    msg!("Presale paused");
    Ok(())
}

#[derive(Accounts)]
pub struct Unpause<'info> {
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

pub fn handler_unpause(ctx: Context<Unpause>) -> Result<()> {
    let presale_config = &mut ctx.accounts.presale_config;
    presale_config.is_paused = false;
    msg!("Presale unpaused");
    Ok(())
}

