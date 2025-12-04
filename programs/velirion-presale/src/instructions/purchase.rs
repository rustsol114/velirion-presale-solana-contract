use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer as SystemTransfer};
use anchor_spl::token::{Token, TokenAccount, Transfer};
use crate::state::*;
use crate::constants::*;
use crate::error::PresaleError;

#[derive(Accounts)]
pub struct Purchase<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    
    #[account(
        mut,
        seeds = [PRESALE_CONFIG_SEED],
        bump = presale_config.bump
    )]
    pub presale_config: Account<'info, PresaleConfig>,
    
    #[account(
        init_if_needed,
        payer = buyer,
        space = UserPurchase::SIZE,
        seeds = [USER_PURCHASE_SEED, buyer.key().as_ref()],
        bump
    )]
    pub user_purchase: Account<'info, UserPurchase>,
    
    /// CHECK: SOL vault for receiving SOL payments
    #[account(
        mut,
        seeds = [SOL_VAULT_SEED],
        bump
    )]
    pub sol_vault: SystemAccount<'info>,
    
    #[account(
        mut,
        seeds = [USDC_VAULT_SEED],
        bump
    )]
    pub usdc_vault: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = buyer_usdc_account.owner == buyer.key() @ PresaleError::Unauthorized,
        constraint = buyer_usdc_account.mint == presale_config.usdc_mint @ PresaleError::InvalidTokenMint
    )]
    pub buyer_usdc_account: Account<'info, TokenAccount>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(
    ctx: Context<Purchase>,
    token_amount: u64,
    payment_type: PaymentType,
) -> Result<()> {
    let presale_config = &mut ctx.accounts.presale_config;
    let user_purchase = &mut ctx.accounts.user_purchase;
    let clock = Clock::get()?;
    
    // Check if presale is paused
    require!(!presale_config.is_paused, PresaleError::PresalePaused);
    
    // Get current active phase
    let phase_index = presale_config
        .get_current_phase(clock.unix_timestamp)
        .ok_or(PresaleError::NoActivePhase)?;
    
    // Extract all needed values from presale_config before borrowing phase to avoid borrow conflicts
    let launch_timestamp = presale_config.launch_timestamp;
    let max_purchase_per_transaction = presale_config.max_purchase_per_transaction;
    let max_purchase_per_wallet = presale_config.max_purchase_per_wallet;
    let min_time_between_purchases = presale_config.min_time_between_purchases;
    let price_per_token = presale_config
        .get_phase_price(phase_index, payment_type)
        .ok_or(PresaleError::InvalidPhaseConfig)?;
    let vesting_launch_percentage = presale_config.vesting_launch_percentage;
    let vesting_monthly_percentage = presale_config.vesting_monthly_percentage;
    
    let phase = &mut presale_config.phases[phase_index];
    
    // Validate token amount
    require!(
        token_amount <= max_purchase_per_transaction,
        PresaleError::ExceedsMaxPerTransaction
    );
    
    // Check wallet limit
    let new_total = user_purchase.total_purchased
        .checked_add(token_amount)
        .ok_or(PresaleError::MathOverflow)?;
    
    require!(
        new_total <= max_purchase_per_wallet,
        PresaleError::ExceedsMaxPerWallet
    );
    
    // Check minimum time between purchases
    if user_purchase.last_purchase_time > 0 {
        let time_since_last = clock
            .unix_timestamp
            .checked_sub(user_purchase.last_purchase_time)
            .ok_or(PresaleError::MathOverflow)?;
        
        require!(
            time_since_last >= min_time_between_purchases,
            PresaleError::TooSoonSinceLastPurchase
        );
    }
    
    // Check phase availability
    let remaining_in_phase = phase
        .tokens_allocated
        .checked_sub(phase.tokens_sold)
        .ok_or(PresaleError::MathOverflow)?;
    
    require!(
        token_amount <= remaining_in_phase,
        PresaleError::InsufficientTokensInPhase
    );
    
    // Calculate payment amount
    // Price is already in the correct units (lamports for SOL, micro-USDC for USDC)
    // token_amount is in the token's native units (assuming 9 decimals)
    
    // Calculate total payment: price_per_token * token_amount
    // Both are in their native units, so we multiply directly
    let payment_amount = price_per_token
        .checked_mul(token_amount)
        .and_then(|p| p.checked_div(1_000_000_000)) // Convert from token units (9 decimals) to payment units
        .ok_or(PresaleError::MathOverflow)?;
    
    require!(payment_amount > 0, PresaleError::InsufficientPayment);
    
    // Process payment
    match payment_type {
        PaymentType::Sol => {
            // Transfer SOL using system program
            let cpi_accounts = SystemTransfer {
                from: ctx.accounts.buyer.to_account_info(),
                to: ctx.accounts.sol_vault.to_account_info(),
            };
            let cpi_program = ctx.accounts.system_program.to_account_info();
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
            transfer(cpi_ctx, payment_amount)?;
        }
        PaymentType::Usdc => {
            // Transfer USDC
            let cpi_accounts = Transfer {
                from: ctx.accounts.buyer_usdc_account.to_account_info(),
                to: ctx.accounts.usdc_vault.to_account_info(),
                authority: ctx.accounts.buyer.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
            anchor_spl::token::transfer(cpi_ctx, payment_amount)?;
        }
    }
    
    // Update user purchase record
    if user_purchase.wallet == Pubkey::default() {
        user_purchase.wallet = ctx.accounts.buyer.key();
        user_purchase.bump = ctx.bumps.user_purchase;
    }
    
    user_purchase.total_purchased = new_total;
    user_purchase.last_purchase_time = clock.unix_timestamp;
    
    match payment_type {
        PaymentType::Sol => {
            user_purchase.total_spent_sol = user_purchase
                .total_spent_sol
                .checked_add(payment_amount)
                .ok_or(PresaleError::MathOverflow)?;
        }
        PaymentType::Usdc => {
            user_purchase.total_spent_usdc = user_purchase
                .total_spent_usdc
                .checked_add(payment_amount)
                .ok_or(PresaleError::MathOverflow)?;
        }
    }
    
    // Calculate and set up vesting schedule
    let launch_amount = token_amount
        .checked_mul(vesting_launch_percentage as u64)
        .and_then(|a| a.checked_div(100))
        .ok_or(PresaleError::MathOverflow)?;
    
    let monthly_amount = token_amount
        .checked_mul(vesting_monthly_percentage as u64)
        .and_then(|a| a.checked_div(100))
        .ok_or(PresaleError::MathOverflow)?;
    
    // Update vesting schedule
    // Launch vesting (40% at launch)
    let launch_release_time = launch_timestamp;
    if user_purchase.vesting_schedule[0].release_time == 0 {
        user_purchase.vesting_schedule[0] = VestingEntry {
            amount: launch_amount,
            release_time: launch_release_time,
            claimed: false,
        };
    } else {
        user_purchase.vesting_schedule[0].amount = user_purchase.vesting_schedule[0]
            .amount
            .checked_add(launch_amount)
            .ok_or(PresaleError::MathOverflow)?;
    }
    
    // Monthly vesting (30% each month, 30 days = 2,592,000 seconds)
    let monthly_interval: i64 = 2_592_000;
    for i in 1..=VESTING_MONTHS as usize {
        let release_time = launch_timestamp
            .checked_add((i as i64).checked_mul(monthly_interval).ok_or(PresaleError::MathOverflow)?)
            .ok_or(PresaleError::MathOverflow)?;
        
        if user_purchase.vesting_schedule[i].release_time == 0 {
            user_purchase.vesting_schedule[i] = VestingEntry {
                amount: monthly_amount,
                release_time: release_time,
                claimed: false,
            };
        } else {
            user_purchase.vesting_schedule[i].amount = user_purchase.vesting_schedule[i]
                .amount
                .checked_add(monthly_amount)
                .ok_or(PresaleError::MathOverflow)?;
        }
    }
    
    // Update presale config
    phase.tokens_sold = phase
        .tokens_sold
        .checked_add(token_amount)
        .ok_or(PresaleError::MathOverflow)?;
    
    presale_config.tokens_sold = presale_config
        .tokens_sold
        .checked_add(token_amount)
        .ok_or(PresaleError::MathOverflow)?;
    
    msg!(
        "Purchase: {} tokens for {} (payment type: {:?})",
        token_amount,
        payment_amount,
        payment_type
    );
    
    Ok(())
}

