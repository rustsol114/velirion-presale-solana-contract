# Velirion Presale Smart Contract

A comprehensive Solana-based token presale program built with Anchor framework, featuring multi-phase sales, dual payment options (SOL/USDC), vesting schedules, and advanced security controls.

## 🌟 Features

### Core Functionality
- **Multi-Phase Presale**: Support for up to 10 configurable presale phases with different pricing
- **Dual Payment Options**: Accept both SOL and USDC payments
- **Vesting Schedule**: Automatic token vesting with customizable release schedules
  - Launch allocation (default 40%)
  - Monthly releases (default 30% per month for 2 months)
- **Purchase Limits**: Configurable per-transaction and per-wallet limits
- **Rate Limiting**: Minimum time between purchases to prevent spam
- **Emergency Controls**: Pause/unpause functionality for security
- **Token Burning**: Burn unsold tokens after presale completion

### Security Features
- ✅ Authority-based access control
- ✅ PDA (Program Derived Address) for secure vault management
- ✅ Comprehensive input validation
- ✅ Math overflow protection
- ✅ Phase timing validation
- ✅ Purchase limit enforcement

## 📋 Program Overview

**Program ID**: `BUn45bk9GmkxjM14ixXZkj3G7ykpRYbgt6FSk9kWJcK1`

**Network**: Devnet (configurable for mainnet)

**Framework**: Anchor 0.32.1

## 🏗️ Architecture

### State Accounts

#### PresaleConfig
Main configuration account storing presale parameters:
- Authority wallet
- Token mint addresses (presale token & USDC)
- Treasury and vault addresses
- Phase configurations (up to 10 phases)
- Purchase limits and timing constraints
- Vesting parameters

#### UserPurchase
Per-user account tracking:
- Total tokens purchased
- SOL and USDC spent amounts
- Last purchase timestamp
- Vesting schedule with 3 entries (launch + 2 months)
- Claimed status for each vesting entry

### Payment Vaults
- **SOL Vault**: PDA-based system account for SOL payments
- **USDC Vault**: Token account for USDC payments

## 📝 Instructions

### 1. Initialize
Sets up the presale with all configuration parameters.

**Parameters:**
- `phases`: Array of 10 presale phases with pricing and timing
- `total_tokens_for_sale`: Total token allocation for presale
- `max_purchase_per_transaction`: Maximum tokens per single purchase
- `max_purchase_per_wallet`: Maximum tokens per wallet
- `min_time_between_purchases`: Cooldown period between purchases (seconds)
- `launch_timestamp`: Token launch date for vesting calculation
- `vesting_launch_percentage`: Percentage released at launch (e.g., 40)
- `vesting_monthly_percentage`: Percentage released monthly (e.g., 30)

**Accounts Required:**
- Authority (signer, payer)
- Token mint
- USDC mint
- Presale config (PDA)
- SOL vault (PDA)
- USDC vault (PDA)
- Treasury token account

### 2. Purchase
Allows users to buy tokens during active phases.

**Parameters:**
- `token_amount`: Amount of tokens to purchase
- `payment_type`: `Sol` or `Usdc`

**Validations:**
- Presale not paused
- Active phase exists
- Within transaction limit
- Within wallet limit
- Sufficient phase allocation
- Cooldown period met
- Sufficient payment

**Accounts Required:**
- Buyer (signer)
- Presale config
- User purchase account (auto-created if needed)
- SOL vault
- USDC vault
- Buyer's USDC account (if paying with USDC)

### 3. Claim Vested
Allows users to claim their vested tokens when unlocked.

**Process:**
- Checks current timestamp against vesting schedule
- Calculates claimable amount
- Marks entries as claimed
- Transfers tokens from treasury to user

**Accounts Required:**
- Buyer (signer)
- Presale config
- User purchase account
- Token mint
- Buyer's token account
- Treasury

### 4. Pause / Unpause
Emergency controls for the presale (authority only).

**Accounts Required:**
- Authority (signer)
- Presale config

### 5. Burn Unsold
Burns remaining unsold tokens after presale ends (authority only).

**Validations:**
- All phases have ended
- Unsold tokens exist
- Treasury has sufficient balance

**Accounts Required:**
- Authority (signer)
- Presale config
- Token mint
- Treasury

### 6. Update Config
Updates presale parameters (authority only).

**Parameters (all optional):**
- `max_purchase_per_transaction`
- `max_purchase_per_wallet`
- `min_time_between_purchases`

### 7. Get Purchase Status
View-only instruction to check user's purchase information.

**Returns:**
- Total purchased
- Total spent (SOL & USDC)
- Claimable tokens
- Remaining allocation
- Last purchase time
- Current phase
- Presale status

## 🔧 Installation & Setup

### Prerequisites
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"

# Install Anchor
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install 0.32.1
avm use 0.32.1

# Install Node.js dependencies
yarn install
```

### Build
```bash
anchor build
```

### Test
```bash
anchor test
```

### Deploy to Devnet
```bash
# Configure Solana CLI for devnet
solana config set --url devnet

# Airdrop SOL for deployment
solana airdrop 2

# Deploy
anchor deploy
```

## 💡 Usage Example

### Initialize Presale
```typescript
const phases = [
  {
    priceSol: new BN(100_000_000), // 0.1 SOL per token
    priceUsdc: new BN(50_000), // $0.05 per token
    startTime: new BN(Date.now() / 1000),
    endTime: new BN(Date.now() / 1000 + 86400 * 7), // 7 days
    tokensAllocated: new BN(1_000_000 * 10**9),
    tokensSold: new BN(0),
  },
  // ... 9 more phases
];

await program.methods
  .initialize(
    phases,
    new BN(10_000_000 * 10**9), // 10M tokens
    new BN(10_000 * 10**9), // Max 10k per tx
    new BN(100_000 * 10**9), // Max 100k per wallet
    new BN(60), // 1 minute cooldown
    new BN(launchTimestamp),
    40, // 40% at launch
    30  // 30% monthly
  )
  .accounts({...})
  .rpc();
```

### Purchase Tokens
```typescript
await program.methods
  .purchase(
    new BN(1000 * 10**9), // 1000 tokens
    { sol: {} } // or { usdc: {} }
  )
  .accounts({...})
  .rpc();
```

### Claim Vested Tokens
```typescript
await program.methods
  .claimVested()
  .accounts({...})
  .rpc();
```

## 📊 Vesting Schedule

The vesting schedule is automatically calculated based on purchase time and launch timestamp:

| Period | Percentage | Release Time |
|--------|-----------|--------------|
| Launch | 40% | Launch timestamp |
| Month 1 | 30% | Launch + 30 days |
| Month 2 | 30% | Launch + 60 days |

**Total**: 100% vested over 2 months after launch

## 🔐 Security Considerations

1. **Authority Control**: Only the authority can pause, unpause, burn tokens, and update config
2. **PDA Vaults**: Funds are stored in PDAs controlled by the program
3. **Purchase Limits**: Prevents whale accumulation and ensures fair distribution
4. **Rate Limiting**: Prevents spam and manipulation
5. **Phase Validation**: Ensures phases are sequential and properly configured
6. **Math Safety**: All arithmetic operations include overflow checks
7. **Token Validation**: Ensures correct token mints are used

## ⚠️ Error Codes

| Code | Error | Description |
|------|-------|-------------|
| 6000 | PresalePaused | Presale is currently paused |
| 6001 | NoActivePhase | No active phase at this time |
| 6002 | ExceedsMaxPerTransaction | Purchase exceeds max per transaction |
| 6003 | ExceedsMaxPerWallet | Purchase exceeds max per wallet |
| 6004 | TooSoonSinceLastPurchase | Cooldown period not met |
| 6005 | InvalidPaymentType | Invalid payment type |
| 6006 | InsufficientPayment | Insufficient payment amount |
| 6007 | InsufficientTokensInPhase | Not enough tokens in phase |
| 6008 | PresaleNotEnded | Presale has not ended yet |
| 6009 | NoTokensToClaim | No tokens available to claim |
| 6010 | AlreadyClaimed | Vesting entry already claimed |
| 6011 | InvalidPhaseConfig | Invalid phase configuration |
| 6012 | Unauthorized | Unauthorized access |
| 6013 | InvalidTokenMint | Invalid token mint |
| 6014 | InvalidTreasury | Invalid treasury account |
| 6015 | MathOverflow | Math overflow occurred |
| 6016 | InvalidVestingSchedule | Invalid vesting schedule |

## 📁 Project Structure

```
velirion-presale/
├── programs/
│   └── velirion-presale/
│       └── src/
│           ├── lib.rs              # Program entry point
│           ├── state.rs            # State account definitions
│           ├── error.rs            # Error definitions
│           ├── constants.rs        # Program constants
│           ├── instructions.rs     # Instruction exports
│           └── instructions/
│               ├── initialize.rs   # Initialize presale
│               ├── purchase.rs     # Purchase tokens
│               ├── claim_vested.rs # Claim vested tokens
│               ├── pause.rs        # Pause/unpause
│               ├── burn_unsold.rs  # Burn unsold tokens
│               ├── update_config.rs # Update configuration
│               └── get_status.rs   # Get purchase status
├── tests/                          # Integration tests
├── Anchor.toml                     # Anchor configuration
└── Cargo.toml                      # Rust dependencies
```

## 🧪 Testing

The program includes comprehensive tests covering:
- Presale initialization
- Token purchases with SOL and USDC
- Purchase limit enforcement
- Vesting and claiming
- Pause/unpause functionality
- Token burning
- Configuration updates

Run tests with:
```bash
anchor test
```

## 🚀 Deployment Checklist

- [ ] Update program ID in `lib.rs` and `Anchor.toml`
- [ ] Configure correct network in `Anchor.toml`
- [ ] Set appropriate phase timings and pricing
- [ ] Prepare treasury token account with sufficient tokens
- [ ] Verify USDC mint address for target network
- [ ] Test all instructions on devnet
- [ ] Audit smart contract code
- [ ] Deploy to mainnet
- [ ] Verify deployment
- [ ] Initialize presale with correct parameters

## 📄 License

This project is licensed under the MIT License.

## 🤝 Contributing

Contributions are welcome! Please ensure:
- Code follows Rust and Anchor best practices
- All tests pass
- New features include tests
- Documentation is updated

## 📞 Support

For issues, questions, or contributions, please open an issue on GitHub.

## ⚡ Performance Notes

**Stack Size Warnings**: The program currently has some functions that exceed the recommended stack size (4096 bytes). This is noted during compilation but does not prevent deployment. For production use, consider:
- Optimizing large data structures
- Using heap allocation for larger arrays
- Breaking down complex functions

## 🔄 Version History

### v0.1.0 (Current)
- Initial release
- Multi-phase presale support
- Dual payment options (SOL/USDC)
- Vesting schedule implementation
- Emergency controls
- Token burning functionality

---

**Built with ❤️ using Anchor Framework**

