# Test Commands Reference

## Running Tests Without Redeploying

Since your contract is already deployed to devnet, you can run tests without redeploying:

### Option 1: Skip Build and Deploy (Fastest)
```bash
anchor test --skip-build --skip-deploy
```

### Option 2: Build but Skip Deploy
```bash
anchor test --skip-deploy
```

### Option 3: Full Test (Build + Deploy + Test)
```bash
anchor test
```

## Test Commands Breakdown

### Skip Build
```bash
anchor test --skip-build
```
- Skips Rust compilation
- Use when you haven't changed the smart contract code
- Saves ~30-60 seconds

### Skip Deploy
```bash
anchor test --skip-deploy
```
- Uses the already deployed contract on devnet
- No need to redeploy if contract hasn't changed
- Saves deployment time and SOL

### Skip Local Validator
```bash
anchor test --skip-local-validator
```
- Uses the configured devnet RPC (Helius in your case)
- No local validator needed
- Tests against real devnet

### Combined Flags
```bash
# Fastest: Skip everything, just run tests
anchor test --skip-build --skip-deploy --skip-local-validator

# Skip build and deploy, but start local validator
anchor test --skip-build --skip-deploy
```

## Current Configuration

**Network**: Devnet (Helius RPC)
**Program ID**: `91uNkK5URavMx6onv6c8XTZ6VkEj4RzA6Xa4pQydWs2s`
**RPC URL**: `https://devnet.helius-rpc.com/?api-key=67cdd4a6-271e-4635-b0d3-6d007ef93fa8`

## Recommended Workflow

### During Development (Contract Changes)
```bash
# Build and deploy new changes
anchor build
anchor deploy

# Run tests against new deployment
anchor test --skip-build --skip-deploy
```

### Testing Only (No Contract Changes)
```bash
# Just run the tests
anchor test --skip-build --skip-deploy
```

### Quick Test Run
```bash
# Fastest way to run tests
yarn test
# This runs: ts-mocha -p ./tsconfig.json -t 1000000 "tests/**/*.ts"
```

## Package.json Scripts

You can also add custom scripts to `package.json`:

```json
{
  "scripts": {
    "test": "anchor test --skip-build --skip-deploy",
    "test:full": "anchor test",
    "test:quick": "ts-mocha -p ./tsconfig.json -t 1000000 \"tests/**/*.ts\""
  }
}
```

Then run:
```bash
yarn test        # Skip build and deploy
yarn test:full   # Full test with build and deploy
yarn test:quick  # Just run test files
```

## Troubleshooting

### If tests fail with "Program not found"
```bash
# Redeploy the contract
anchor deploy
```

### If you get RPC errors
```bash
# Check your Helius API key is valid
# Or switch to public devnet RPC temporarily
```

### If you need to reset state
```bash
# Close old accounts and redeploy
anchor deploy --program-name velirion-presale
```

## Testing Against Different Networks

### Localnet (for development)
```bash
# Update Anchor.toml cluster to "localnet"
anchor test
```

### Devnet (current setup)
```bash
# Already configured
anchor test --skip-build --skip-deploy
```

### Mainnet (production - be careful!)
```bash
# Update Anchor.toml cluster to "mainnet"
# Make sure you have enough SOL
anchor test --skip-build --skip-deploy
```

## Time Savings

| Command | Build | Deploy | Test | Total Time |
|---------|-------|--------|------|------------|
| `anchor test` | ✅ 30s | ✅ 10s | ✅ 60s | ~100s |
| `anchor test --skip-build` | ❌ | ✅ 10s | ✅ 60s | ~70s |
| `anchor test --skip-deploy` | ✅ 30s | ❌ | ✅ 60s | ~90s |
| `anchor test --skip-build --skip-deploy` | ❌ | ❌ | ✅ 60s | ~60s |
| `yarn test` (direct) | ❌ | ❌ | ✅ 60s | ~60s |

## Best Practices

1. **Use `--skip-deploy` when testing** - Your contract is already on devnet
2. **Use `--skip-build` when no code changes** - Saves compilation time
3. **Run full `anchor test` only when deploying** - To ensure everything is in sync
4. **Use `yarn test` for quick iterations** - Fastest way to run tests

---

**Current Setup**: Tests run against deployed contract on devnet using Helius RPC ✅

