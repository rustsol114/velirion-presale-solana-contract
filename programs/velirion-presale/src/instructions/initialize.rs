use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use crate::state::*;
use crate::constants::*;
use crate::error::PresaleError;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// CHECK: Validated in handler
    pub token_mint: UncheckedAccount<'info>,
    
    /// CHECK: Validated in handler
    pub usdc_mint: UncheckedAccount<'info>,
    
    #[account(
        init,
        payer = authority,
        space = PresaleConfig::SIZE,
        seeds = [PRESALE_CONFIG_SEED],
        bump
    )]
    pub presale_config: Account<'info, PresaleConfig>,
    
    /// CHECK: This is a PDA for receiving SOL payments
    #[account(
        seeds = [SOL_VAULT_SEED],
        bump
    )]
    pub sol_vault: SystemAccount<'info>,
    
    #[account(
        init,
        payer = authority,
        token::mint = usdc_mint,
        token::authority = presale_config,
        seeds = [USDC_VAULT_SEED],
        bump
    )]
    pub usdc_vault: Account<'info, TokenAccount>,
    
    /// CHECK: Treasury token account that holds presale tokens (must be created separately and funded)
    /// CHECK: Validated in handler
    pub treasury: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(
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
    let presale_config = &mut ctx.accounts.presale_config;
    
    // Validate vesting percentages
    require!(
        vesting_launch_percentage + (vesting_monthly_percentage * 2) == 100,
        PresaleError::InvalidVestingSchedule
    );
    
    // Validate phases
    for (i, phase) in phases.iter().enumerate() {
        require!(
            phase.start_time < phase.end_time,
            PresaleError::InvalidPhaseConfig
        );
        require!(
            phase.price_sol > 0 && phase.price_usdc > 0,
            PresaleError::InvalidPhaseConfig
        );
        require!(
            phase.tokens_allocated > 0,
            PresaleError::InvalidPhaseConfig
        );
        
        // Ensure phases are sequential
        if i > 0 {
            require!(
                phases[i - 1].end_time <= phase.start_time,
                PresaleError::InvalidPhaseConfig
            );
        }
    }
    
    // Validate token mints and treasury account
    let treasury_data = anchor_spl::token::TokenAccount::try_deserialize(&mut &ctx.accounts.treasury.data.borrow()[..])?;
    require!(
        treasury_data.mint == ctx.accounts.token_mint.key(),
        PresaleError::InvalidTokenMint
    );
    
    presale_config.authority = ctx.accounts.authority.key();
    presale_config.token_mint = ctx.accounts.token_mint.key();
    presale_config.usdc_mint = ctx.accounts.usdc_mint.key();
    presale_config.treasury = ctx.accounts.treasury.key();
    presale_config.sol_vault = ctx.accounts.sol_vault.key();
    presale_config.usdc_vault = ctx.accounts.usdc_vault.key();
    presale_config.is_paused = false;
    presale_config.total_tokens_for_sale = total_tokens_for_sale;
    presale_config.tokens_sold = 0;
    presale_config.launch_timestamp = launch_timestamp;
    presale_config.phases = phases;
    presale_config.max_purchase_per_transaction = max_purchase_per_transaction;
    presale_config.max_purchase_per_wallet = max_purchase_per_wallet;
    presale_config.min_time_between_purchases = min_time_between_purchases;
    presale_config.vesting_launch_percentage = vesting_launch_percentage;
    presale_config.vesting_monthly_percentage = vesting_monthly_percentage;
    presale_config.bump = ctx.bumps.presale_config;
    
    msg!("Presale initialized with {} tokens", total_tokens_for_sale);
    Ok(())
}
