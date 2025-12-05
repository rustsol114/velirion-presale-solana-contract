# Velirion Presale Tests

Comprehensive test suite for the Velirion Presale smart contract.

## 📋 Test Coverage

### 1. Initialize Tests
- ✅ Successfully initializes presale with all parameters
- ✅ Prevents double initialization
- ✅ Validates phase configurations
- ✅ Validates vesting percentages

### 2. Purchase Tests (SOL)
- ✅ Allows token purchase with SOL payment
- ✅ Enforces maximum per transaction limit
- ✅ Enforces maximum per wallet limit
- ✅ Enforces minimum time between purchases
- ✅ Updates user purchase records
- ✅ Updates presale statistics
- ✅ Transfers SOL to vault

### 3. Purchase Tests (USDC)
- ✅ Allows token purchase with USDC payment
- ✅ Transfers USDC to vault
- ✅ Updates user purchase records
- ✅ Validates USDC token accounts

### 4. Pause/Unpause Tests
- ✅ Authority can pause presale
- ✅ Prevents purchases when paused
- ✅ Authority can unpause presale
- ✅ Prevents non-authority from pausing
- ✅ Prevents non-authority from unpausing

### 5. Update Config Tests
- ✅ Authority can update configuration
- ✅ Updates max per transaction
- ✅ Updates max per wallet
- ✅ Updates min time between purchases
- ✅ Prevents non-authority from updating

### 6. Get Purchase Status Tests
- ✅ Returns accurate purchase information
- ✅ Calculates claimable amounts
- ✅ Shows remaining allocation

### 7. Claim Vested Tests
- ✅ Prevents claiming before vesting period
- ✅ Allows claiming after vesting unlock (time-dependent)
- ✅ Marks entries as claimed
- ✅ Transfers correct amounts

## 🚀 Running Tests

### Run all tests
```bash
anchor test
```

### Run tests with logs
```bash
anchor test -- --nocapture
```

### Run specific test file
```bash
anchor test --skip-build tests/velirion-presale.ts
```

### Run tests on localnet
```bash
# Terminal 1: Start local validator
solana-test-validator

# Terminal 2: Run tests
anchor test --skip-local-validator
```

### Run tests on devnet
```bash
anchor test --provider.cluster devnet
```

## 📦 Test Setup

The test suite automatically:
1. Creates test keypairs for authority and buyers
2. Airdrops SOL to test accounts
3. Creates token mints (presale token and USDC)
4. Creates and funds token accounts
5. Derives all necessary PDAs
6. Initializes the presale

## 🛠️ Test Utilities

The `utils.ts` file provides helper functions:

### Time Utilities
- `sleep(seconds)` - Wait for specified time
- `getCurrentTimestamp()` - Get current Unix timestamp
- `getFutureTimestamp(days)` - Create future timestamp

### Token Utilities
- `toTokenAmount(amount, decimals)` - Convert to base units
- `fromTokenAmount(amount, decimals)` - Convert from base units
- `createTestPhases()` - Generate test phase configurations

### SOL Utilities
- `lamportsToSol(lamports)` - Convert lamports to SOL
- `solToLamports(sol)` - Convert SOL to lamports
- `airdropSol()` - Airdrop with retry logic
- `getBalance()` - Get account balance

### Assertion Utilities
- `assertBNEqual()` - Assert BN equality
- `assertBNGreaterThan()` - Assert BN greater than
- `assertBNLessThan()` - Assert BN less than

### Logging Utilities
- `logTransaction()` - Log transaction details
- `logAccountInfo()` - Log account information

## 📊 Test Accounts

### Authority
- Initializes the presale
- Can pause/unpause
- Can update configuration
- Can burn unsold tokens

### Buyer 1
- Tests SOL purchases
- Tests purchase limits
- Tests cooldown periods

### Buyer 2
- Tests USDC purchases
- Tests independent user tracking

## 🔍 Test Data

### Presale Configuration
- **Total Tokens**: 10,000,000 tokens
- **Max Per Transaction**: 10,000 tokens
- **Max Per Wallet**: 100,000 tokens
- **Min Time Between**: 60 seconds
- **Vesting Launch**: 40%
- **Vesting Monthly**: 30%

### Phase Configuration
- **10 Phases**: Each 7 days long
- **Phase Allocation**: 1,000,000 tokens each
- **SOL Price**: Starting at 0.1 SOL, increasing by 0.01 per phase
- **USDC Price**: Starting at $0.05, increasing by $0.005 per phase

## ⚠️ Important Notes

### Time-Dependent Tests
Some tests require waiting for time periods:
- **Cooldown tests**: Wait 60+ seconds between purchases
- **Vesting tests**: Require waiting until launch timestamp

### Airdrop Limits
On devnet/mainnet, airdrop limits apply:
- Maximum 2 SOL per airdrop on devnet
- Use faucets for additional SOL

### Account Cleanup
Tests create accounts that persist on devnet/mainnet. Consider:
- Using localnet for development
- Cleaning up test accounts
- Using separate test wallets

## 🐛 Debugging

### Enable detailed logs
```bash
RUST_LOG=debug anchor test
```

### View program logs
```bash
solana logs | grep "Program log:"
```

### Check account data
```bash
solana account <ACCOUNT_ADDRESS>
```

### Inspect transaction
```bash
solana confirm -v <TRANSACTION_SIGNATURE>
```

## 📝 Adding New Tests

1. Create test case in appropriate describe block
2. Use helper functions from `utils.ts`
3. Follow existing patterns for account setup
4. Add assertions for expected behavior
5. Test both success and failure cases
6. Update this README with new coverage

## 🔗 Related Files

- `velirion-presale.ts` - Main test suite
- `utils.ts` - Test utilities and helpers
- `../programs/velirion-presale/` - Smart contract source

## 📚 Resources

- [Anchor Testing Guide](https://www.anchor-lang.com/docs/testing)
- [Solana Web3.js](https://solana-labs.github.io/solana-web3.js/)
- [SPL Token](https://spl.solana.com/token)
- [Mocha Documentation](https://mochajs.org/)
- [Chai Assertions](https://www.chaijs.com/)

