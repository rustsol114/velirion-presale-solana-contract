use anchor_lang::prelude::*;

#[error_code]
pub enum PresaleError {
    #[msg("Presale is currently paused")]
    PresalePaused,
    
    #[msg("No active phase at this time")]
    NoActivePhase,
    
    #[msg("Purchase amount exceeds maximum per transaction")]
    ExceedsMaxPerTransaction,
    
    #[msg("Purchase would exceed maximum per wallet")]
    ExceedsMaxPerWallet,
    
    #[msg("Minimum time between purchases not met")]
    TooSoonSinceLastPurchase,
    
    #[msg("Invalid payment type")]
    InvalidPaymentType,
    
    #[msg("Insufficient payment amount")]
    InsufficientPayment,
    
    #[msg("Not enough tokens available in this phase")]
    InsufficientTokensInPhase,
    
    #[msg("Presale has not ended yet")]
    PresaleNotEnded,
    
    #[msg("No tokens available to claim")]
    NoTokensToClaim,
    
    #[msg("Vesting entry already claimed")]
    AlreadyClaimed,
    
    #[msg("Invalid phase configuration")]
    InvalidPhaseConfig,
    
    #[msg("Unauthorized access")]
    Unauthorized,
    
    #[msg("Invalid token mint")]
    InvalidTokenMint,
    
    #[msg("Invalid treasury account")]
    InvalidTreasury,
    
    #[msg("Math overflow")]
    MathOverflow,
    
    #[msg("Invalid vesting schedule")]
    InvalidVestingSchedule,
}
