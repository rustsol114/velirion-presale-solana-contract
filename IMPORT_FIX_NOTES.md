# Import Fix Notes - Utils.ts

## Issue
CommonJS module import error with `@coral-xyz/anchor`:
```
SyntaxError: Named export 'BN' not found. The requested module '@coral-xyz/anchor' is a CommonJS module
```

## Root Cause
The `@coral-xyz/anchor` package exports some types as CommonJS, which doesn't support all named exports in ES modules. Specifically, `BN` and `web3` need to be imported from their original packages.

## Solution

### Before (Incorrect)
```typescript
import * as anchor from "@coral-xyz/anchor";
import { BN } from "@coral-xyz/anchor";  // ❌ Not available as named export
```

### After (Correct)
```typescript
import * as anchor from "@coral-xyz/anchor";
import BN from "bn.js";  // ✅ Import from source package
import { Connection, PublicKey, LAMPORTS_PER_SOL } from "@solana/web3.js";  // ✅ Import from source
```

## Changes Made

### 1. Updated Imports in `tests/utils.ts`
- Changed `BN` import from `@coral-xyz/anchor` to `bn.js`
- Changed `web3` types from `anchor.web3.*` to direct imports from `@solana/web3.js`
- Updated all function signatures to use the new imports

### 2. Added Dependencies in `package.json`
```json
{
  "dependencies": {
    "@coral-xyz/anchor": "^0.32.1",
    "@solana/spl-token": "^0.4.9",
    "@solana/web3.js": "^1.95.5",  // ✅ Added
    "bn.js": "^5.2.1"               // ✅ Added
  }
}
```

### 3. Updated Function Signatures

**Before:**
```typescript
export async function airdropSol(
  connection: anchor.web3.Connection,
  publicKey: anchor.web3.PublicKey,
  amount: number
): Promise<void>
```

**After:**
```typescript
export async function airdropSol(
  connection: Connection,
  publicKey: PublicKey,
  amount: number
): Promise<void>
```

## Why This Works

1. **BN (Big Number)**: The `bn.js` library is the actual source of the `BN` class. Anchor re-exports it, but as a CommonJS module, the named export doesn't work properly in ES modules.

2. **web3.js**: Similarly, Solana's web3.js types should be imported directly from `@solana/web3.js` rather than through Anchor's re-export.

3. **Type Safety**: By importing from the source packages, we get proper TypeScript types and avoid CommonJS/ESM interop issues.

## Files Modified

1. ✅ `tests/utils.ts` - Updated imports and function signatures
2. ✅ `package.json` - Added `bn.js` and `@solana/web3.js` dependencies

## Verification

```bash
# Install dependencies
yarn install

# Check for linter errors
# No errors found ✅
```

## Best Practices

When working with Anchor:
- Import `BN` from `bn.js` directly
- Import web3.js types from `@solana/web3.js` directly
- Import Anchor-specific types from `@coral-xyz/anchor`
- Use `import BN from "bn.js"` (default import) not `import { BN }`

## Related Issues

This is a known issue with:
- CommonJS/ESM interoperability
- Re-exported modules in Anchor
- TypeScript module resolution

## References

- [bn.js Documentation](https://github.com/indutny/bn.js/)
- [Solana Web3.js Documentation](https://solana-labs.github.io/solana-web3.js/)
- [Anchor Framework](https://www.anchor-lang.com/)

---

**Status**: ✅ All import errors fixed
**Date**: December 5, 2025

