use anchor_lang::prelude::*;

pub const PHASE_COUNT: usize = 10;
pub const VESTING_MONTHS: u8 = 2; // 40% at launch + 2 months of 30% each

#[account]
pub struct PresaleConfig {
    pub authority: Pubkey,
    pub token_mint: Pubkey,
    pub treasury: Pubkey,
    pub usdc_mint: Pubkey,
    pub sol_vault: Pubkey,
    pub usdc_vault: Pubkey,
    pub is_paused: bool,
    pub total_tokens_for_sale: u64,
    pub tokens_sold: u64,
    pub launch_timestamp: i64,
    pub phases: [PresalePhase; PHASE_COUNT],
    pub max_purchase_per_transaction: u64,
    pub max_purchase_per_wallet: u64,
    pub min_time_between_purchases: i64, // in seconds
    pub vesting_launch_percentage: u8, // 40%
    pub vesting_monthly_percentage: u8, // 30%
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub struct PresalePhase {
    pub price_sol: u64,        // Price in lamports (1 SOL = 1_000_000_000 lamports)
    pub price_usdc: u64,       // Price in USDC (1 USDC = 1_000_000 micro-USDC)
    pub start_time: i64,
    pub end_time: i64,
    pub tokens_allocated: u64,
    pub tokens_sold: u64,
}

impl Default for PresalePhase {
    fn default() -> Self {
        PresalePhase {
            price_sol: 0,
            price_usdc: 0,
            start_time: 0,
            end_time: 0,
            tokens_allocated: 0,
            tokens_sold: 0,
        }
    }
}

#[account]
pub struct UserPurchase {
    pub wallet: Pubkey,
    pub total_purchased: u64,
    pub total_spent_sol: u64,
    pub total_spent_usdc: u64,
    pub last_purchase_time: i64,
    pub vesting_schedule: [VestingEntry; VESTING_MONTHS as usize + 1], // +1 for launch
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub struct VestingEntry {
    pub amount: u64,
    pub release_time: i64,
    pub claimed: bool,
}

impl Default for VestingEntry {
    fn default() -> Self {
        VestingEntry {
            amount: 0,
            release_time: 0,
            claimed: false,
        }
    }
}

impl PresaleConfig {
    pub const SIZE: usize = 8 + // discriminator
        32 + // authority
        32 + // token_mint
        32 + // treasury
        32 + // usdc_mint
        32 + // sol_vault
        32 + // usdc_vault
        1 +  // is_paused
        8 +  // total_tokens_for_sale
        8 +  // tokens_sold
        8 +  // launch_timestamp
        (PHASE_COUNT * std::mem::size_of::<PresalePhase>()) + // phases
        8 +  // max_purchase_per_transaction
        8 +  // max_purchase_per_wallet
        8 +  // min_time_between_purchases
        1 +  // vesting_launch_percentage
        1 +  // vesting_monthly_percentage
        1;   // bump

    pub fn get_current_phase(&self, current_time: i64) -> Option<usize> {
        for (index, phase) in self.phases.iter().enumerate() {
            if current_time >= phase.start_time && current_time <= phase.end_time {
                return Some(index);
            }
        }
        None
    }

    pub fn get_phase_price(&self, phase_index: usize, payment_type: PaymentType) -> Option<u64> {
        if phase_index >= PHASE_COUNT {
            return None;
        }
        match payment_type {
            PaymentType::Sol => Some(self.phases[phase_index].price_sol),
            PaymentType::Usdc => Some(self.phases[phase_index].price_usdc),
        }
    }
}

impl UserPurchase {
    pub const SIZE: usize = 8 + // discriminator
        32 + // wallet
        8 +  // total_purchased
        8 +  // total_spent_sol
        8 +  // total_spent_usdc
        8 +  // last_purchase_time
        ((VESTING_MONTHS as usize + 1) * std::mem::size_of::<VestingEntry>()) + // vesting_schedule
        1;   // bump

    pub fn get_claimable_amount(&self, current_time: i64) -> u64 {
        let mut claimable = 0u64;
        for entry in &self.vesting_schedule {
            if !entry.claimed && current_time >= entry.release_time && entry.amount > 0 {
                claimable += entry.amount;
            }
        }
        claimable
    }

    pub fn get_remaining_allocation(&self, max_per_wallet: u64) -> u64 {
        if self.total_purchased >= max_per_wallet {
            return 0;
        }
        max_per_wallet - self.total_purchased
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum PaymentType {
    Sol,
    Usdc,
}

