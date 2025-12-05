# Test Guide - Quick Reference

## 🚀 Quick Start

### 1. Install Dependencies
```bash
yarn install
```

### 2. Build the Program
```bash
anchor build
```

### 3. Run Tests
```bash
anchor test
```

## 📋 Test Commands

### Basic Testing
```bash
# Run all tests (builds, deploys, and tests)
anchor test

# Run tests without building
anchor test --skip-build

# Run tests with detailed output
anchor test -- --nocapture
```

### Network-Specific Testing
```bash
# Test on localnet (default)
anchor test

# Test on devnet
anchor test --provider.cluster devnet

# Test on specific RPC
anchor test --provider.cluster https://api.devnet.solana.com
```

### Advanced Testing
```bash
# Skip local validator (use existing one)
anchor test --skip-local-validator

# Run with specific test file
anchor test tests/velirion-presale.ts

# Run with Rust logs
RUST_LOG=debug anchor test
```

## 🧪 Test Structure

```
tests/
├── velirion-presale.ts    # Main test suite
├── utils.ts               # Helper functions
└── README.md              # Detailed test documentation
```

## 📊 Test Scenarios Covered

### ✅ Initialization
- Create presale with 10 phases
- Validate configuration
- Prevent double initialization

### ✅ Purchases
- Buy with SOL
- Buy with USDC
- Enforce transaction limits
- Enforce wallet limits
- Enforce cooldown periods

### ✅ Access Control
- Pause/unpause (authority only)
- Update config (authority only)
- Burn tokens (authority only)

### ✅ Vesting
- Calculate claimable amounts
- Claim vested tokens
- Prevent early claims

### ✅ Status Queries
- Get purchase information
- View vesting schedules
- Check remaining allocation

## 🔧 Troubleshooting

### Issue: Tests timeout
**Solution**: Increase timeout in Anchor.toml
```toml
[scripts]
test = "yarn run ts-mocha -p ./tsconfig.json -t 1000000 \"tests/**/*.ts\""
```

### Issue: Insufficient SOL
**Solution**: Airdrop more SOL
```bash
solana airdrop 2
```

### Issue: Account already exists
**Solution**: Use fresh accounts or clean up
```bash
# Start fresh validator
solana-test-validator --reset
```

### Issue: Program not deployed
**Solution**: Build and deploy
```bash
anchor build
anchor deploy
```

### Issue: RPC rate limits
**Solution**: Use local validator
```bash
# Terminal 1
solana-test-validator

# Terminal 2
anchor test --skip-local-validator
```

## 📝 Test Output Example

```
velirion-presale
  Initialize
    ✓ Initializes the presale successfully (1234ms)
    ✓ Fails to initialize twice (567ms)
  Purchase with SOL
    ✓ Allows buyer to purchase tokens with SOL (890ms)
    ✓ Enforces maximum per transaction limit (456ms)
    ✓ Enforces minimum time between purchases (123ms)
  Purchase with USDC
    ✓ Allows buyer to purchase tokens with USDC (789ms)
  Pause/Unpause
    ✓ Allows authority to pause presale (234ms)
    ✓ Prevents purchases when paused (345ms)
    ✓ Allows authority to unpause presale (456ms)
    ✓ Prevents non-authority from pausing (567ms)
  Update Config
    ✓ Allows authority to update configuration (678ms)
    ✓ Prevents non-authority from updating config (789ms)
  Get Purchase Status
    ✓ Returns purchase status for a user (890ms)
  Claim Vested
    ✓ Prevents claiming before vesting period (901ms)
  Summary
    ✓ Displays final presale statistics (123ms)

  15 passing (12s)
```

## 🎯 Testing Checklist

Before deploying to mainnet:

- [ ] All tests pass on localnet
- [ ] All tests pass on devnet
- [ ] Test with multiple buyers
- [ ] Test edge cases (limits, timing)
- [ ] Test error conditions
- [ ] Test pause/unpause functionality
- [ ] Test configuration updates
- [ ] Test vesting claims
- [ ] Verify token transfers
- [ ] Check vault balances
- [ ] Review transaction logs
- [ ] Audit smart contract code

## 🔍 Debugging Tips

### View Program Logs
```bash
# In separate terminal
solana logs | grep "velirion"
```

### Check Account Data
```bash
# View presale config
solana account <PRESALE_CONFIG_ADDRESS>

# View user purchase
solana account <USER_PURCHASE_ADDRESS>
```

### Inspect Transactions
```bash
solana confirm -v <TRANSACTION_SIGNATURE>
```

### Check Token Balances
```bash
spl-token accounts
spl-token balance <TOKEN_MINT>
```

## 📚 Additional Resources

- [Main README](./README.md) - Project overview
- [Test README](./tests/README.md) - Detailed test documentation
- [Anchor Docs](https://www.anchor-lang.com/docs) - Framework documentation
- [Solana Cookbook](https://solanacookbook.com/) - Code examples

## 🤝 Contributing Tests

When adding new tests:

1. Follow existing patterns
2. Use helper functions from `utils.ts`
3. Add descriptive test names
4. Test both success and failure cases
5. Update documentation
6. Ensure tests are deterministic
7. Clean up resources

## ⚡ Performance Tips

- Use `--skip-build` when code hasn't changed
- Run on localnet for faster iteration
- Use `--skip-local-validator` with existing validator
- Parallel test execution (if applicable)
- Cache test accounts between runs

---

**Happy Testing! 🎉**

