import * as anchor from "@coral-xyz/anchor";
import { Program, BN, web3 } from "@coral-xyz/anchor";
import { VelirionPresale } from "../target/types/velirion_presale";
import {
  createMint,
  createAccount,
  mintTo,
  getAccount,
  createSetAuthorityInstruction,
  createInitializeAccountInstruction,
  getMinimumBalanceForRentExemptAccount,
  AuthorityType,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { assert, expect } from "chai";
import { Connection } from "@solana/web3.js";
import bs58 from "bs58";
import dotenv from "dotenv";

dotenv.config();

describe("velirion-presale", () => {
  // Configure the client to use Helius devnet RPC
  const connection = new Connection(
    "https://devnet.helius-rpc.com/?api-key=67cdd4a6-271e-4635-b0d3-6d007ef93fa8",
    "confirmed"
  );
  
  const wallet = anchor.AnchorProvider.env().wallet;
  const provider = new anchor.AnchorProvider(connection, wallet, {
    commitment: "confirmed",
  });
  anchor.setProvider(provider);

  const program = anchor.workspace.VelirionPresale as Program<VelirionPresale>;

  // Test accounts
  let authority: web3.Keypair;
  let buyer1: web3.Keypair;
  let buyer2: web3.Keypair;
  
  // Token mints
  let tokenMint: web3.PublicKey;
  let usdcMint: web3.PublicKey;
  
  // Token accounts
  let treasury: web3.PublicKey;
  let buyer1TokenAccount: web3.PublicKey;
  let buyer2TokenAccount: web3.PublicKey;
  let buyer1UsdcAccount: web3.PublicKey;
  let buyer2UsdcAccount: web3.PublicKey;
  
  // PDAs
  let presaleConfig: web3.PublicKey;
  let presaleConfigBump: number;
  let solVault: web3.PublicKey;
  let solVaultBump: number;
  let usdcVault: web3.PublicKey;
  let usdcVaultBump: number;
  let buyer1Purchase: web3.PublicKey;
  let buyer2Purchase: web3.PublicKey;

  // Constants
  const PRESALE_CONFIG_SEED = Buffer.from("presale_config");
  const USER_PURCHASE_SEED = Buffer.from("user_purchase");
  const SOL_VAULT_SEED = Buffer.from("sol_vault");
  const USDC_VAULT_SEED = Buffer.from("usdc_vault");

  // Presale configuration
  const TOTAL_TOKENS = new BN(10_000_000).mul(new BN(10 ** 9)); // 10M tokens
  const MAX_PER_TX = new BN(10_000).mul(new BN(10 ** 9)); // 10k tokens
  const MAX_PER_WALLET = new BN(100_000).mul(new BN(10 ** 9)); // 100k tokens
  const MIN_TIME_BETWEEN = new BN(60); // 60 seconds
  const VESTING_LAUNCH_PCT = 40;
  const VESTING_MONTHLY_PCT = 30;

  before(async () => {
    // Generate keypairs
    // authority = web3.Keypair.generate();
    authority = web3.Keypair.fromSecretKey(bs58.decode(process.env.AUTHORITY_SECRET_KEY || ''));
    buyer1 = web3.Keypair.fromSecretKey(bs58.decode(process.env.BUYER1_SECRET_KEY || ''));
    buyer2 = web3.Keypair.fromSecretKey(bs58.decode(process.env.BUYER2_SECRET_KEY || ''));

    // Helper function to airdrop with retries
    const airdropWithRetry = async (
      publicKey: web3.PublicKey,
      amount: number,
      retries = 3
    ) => {
      for (let i = 0; i < retries; i++) {
        try {
          const signature = await provider.connection.requestAirdrop(
            publicKey,
            amount * web3.LAMPORTS_PER_SOL
          );
          await provider.connection.confirmTransaction(signature, "confirmed");
          // Verify balance
          const balance = await provider.connection.getBalance(publicKey);
          if (balance >= amount * web3.LAMPORTS_PER_SOL * 0.9) {
            // Got at least 90% of requested amount
            return true;
          }
        } catch (err) {
          console.log(`Airdrop attempt ${i + 1} failed for ${publicKey.toString()}:`, err.message);
          if (i < retries - 1) {
            await new Promise((resolve) => setTimeout(resolve, 3000 * (i + 1)));
          }
        }
      }
      return false;
    };

    // Helper function to transfer SOL from wallet
    const transferFromWallet = async (
      to: web3.PublicKey,
      amount: number
    ) => {
      const walletPubkey = wallet.publicKey;
      const balance = await provider.connection.getBalance(walletPubkey);
      
      if (balance < amount * web3.LAMPORTS_PER_SOL * 1.1) {
        throw new Error(`Wallet has insufficient SOL. Need ${amount} SOL, have ${balance / web3.LAMPORTS_PER_SOL}`);
      }

      // Use Anchor's system program transfer
      const tx = await anchor.web3.SystemProgram.transfer({
        fromPubkey: walletPubkey,
        toPubkey: to,
        lamports: amount * web3.LAMPORTS_PER_SOL,
      });

      const transaction = new web3.Transaction().add(tx);
      const signature = await web3.sendAndConfirmTransaction(
        provider.connection,
        transaction,
        [wallet.payer as web3.Signer],
        { commitment: "confirmed" }
      );
      
      await provider.connection.confirmTransaction(signature, "confirmed");
    };

    // Fund test accounts - try airdrop first, fallback to transfer from wallet
    console.log("Funding test accounts...");
    
    // Check wallet balance first
    const walletBalance = await provider.connection.getBalance(wallet.publicKey);
    const walletSol = walletBalance / web3.LAMPORTS_PER_SOL;
    console.log(`Wallet balance: ${walletSol.toFixed(4)} SOL`);
    
    if (walletSol < 0.5) {
      console.warn(`⚠️  Warning: Wallet has low balance (${walletSol.toFixed(4)} SOL)`);
      console.warn(`   You may need to fund your wallet or request airdrops manually.`);
      console.warn(`   Recommended: At least 1 SOL for smooth testing.`);
    }
    
    // Calculate amounts based on available wallet balance
    // Reserve 0.1 SOL for transaction fees
    const availableSol = Math.max(0, walletSol - 0.1);
    const perAccount = Math.min(0.3, availableSol / 3.5); // Split available SOL between accounts
    
    const accounts = [
      { keypair: authority, name: "authority", amount: Math.max(0.2, perAccount * 1.5) },
      { keypair: buyer1, name: "buyer1", amount: Math.max(0.15, perAccount) },
      { keypair: buyer2, name: "buyer2", amount: Math.max(0.15, perAccount) },
    ];

    for (const account of accounts) {
      // Check if account already has sufficient balance
      const existingBalance = await provider.connection.getBalance(account.keypair.publicKey);
      const existingSol = existingBalance / web3.LAMPORTS_PER_SOL;
      
      if (existingSol >= account.amount * 0.5) {
        console.log(`${account.name} already has ${existingSol.toFixed(4)} SOL, skipping...`);
        continue;
      }

      console.log("public key ===>", account.keypair.publicKey);
      // Try airdrop first (may hit rate limits)
      const airdropSuccess = await airdropWithRetry(account.keypair.publicKey, account.amount);
      
      if (!airdropSuccess) {
        console.log(`Airdrop failed for ${account.name}, transferring from wallet...`);
        try {
          // Check if wallet has enough
          const currentWalletBalance = await provider.connection.getBalance(wallet.publicKey);
          const needed = account.amount * web3.LAMPORTS_PER_SOL + 5000; // Add small buffer for fees
          
          if (currentWalletBalance < needed) {
            console.warn(`Wallet has insufficient SOL for ${account.name}. Need ${(needed / web3.LAMPORTS_PER_SOL).toFixed(4)} SOL, have ${(currentWalletBalance / web3.LAMPORTS_PER_SOL).toFixed(4)} SOL`);
            // Try with whatever is available (minus fees)
            const available = Math.max(0.05, (currentWalletBalance - 50000) / web3.LAMPORTS_PER_SOL);
            if (available > 0.05) {
              await transferFromWallet(account.keypair.publicKey, available);
              console.log(`Transferred ${available.toFixed(4)} SOL to ${account.name}`);
            } else {
              throw new Error(`Cannot fund ${account.name} - insufficient wallet balance`);
            }
          } else {
            await transferFromWallet(account.keypair.publicKey, account.amount);
            console.log(`Successfully funded ${account.name} via wallet transfer`);
          }
        } catch (err: any) {
          console.error(`Failed to fund ${account.name}:`, err.message || err);
          // If wallet transfer fails, check if account has minimal balance to continue
          const balance = await provider.connection.getBalance(account.keypair.publicKey);
          if (balance < 0.1 * web3.LAMPORTS_PER_SOL) {
            throw new Error(`Cannot proceed: ${account.name} has insufficient SOL (${balance / web3.LAMPORTS_PER_SOL} SOL)`);
          }
          console.warn(`Warning: ${account.name} has minimal balance but continuing...`);
        }
      } else {
        console.log(`Successfully funded ${account.name} via airdrop`);
      }
      
      // Add delay between requests to avoid rate limiting
      await new Promise((resolve) => setTimeout(resolve, 2000));
    }
    
    console.log("All accounts funded!");

    // Create token mint (presale token)
    tokenMint = new anchor.web3.PublicKey("J1toBuV4K6dK2Tk5dGpq6mbgm3f6R8JCRfMvPtbmUYcr");
    // tokenMint = await createMint(
    //   provider.connection,
    //   authority,
    //   authority.publicKey,
    //   null,
    //   9 // 9 decimals
    // );

    // Create USDC mock mint
    usdcMint = new anchor.web3.PublicKey("Ch9MipiMpaZBkCZFPTsArZigDwEH85Yodp2RcPjSmsvr");
    // usdcMint = await createMint(
    //   provider.connection,
    //   authority,
    //   authority.publicKey,
    //   null,
    //   6 // USDC has 6 decimals
    // );

    // Derive PDAs
    [presaleConfig, presaleConfigBump] = web3.PublicKey.findProgramAddressSync(
      [PRESALE_CONFIG_SEED],
      program.programId
    );

    [solVault, solVaultBump] = web3.PublicKey.findProgramAddressSync(
      [SOL_VAULT_SEED],
      program.programId
    );

    [usdcVault, usdcVaultBump] = web3.PublicKey.findProgramAddressSync(
      [USDC_VAULT_SEED],
      program.programId
    );

    [buyer1Purchase] = web3.PublicKey.findProgramAddressSync(
      [USER_PURCHASE_SEED, buyer1.publicKey.toBuffer()],
      program.programId
    );

    [buyer2Purchase] = web3.PublicKey.findProgramAddressSync(
      [USER_PURCHASE_SEED, buyer2.publicKey.toBuffer()],
      program.programId
    );

    // Create treasury token account
    // Treasury should be owned by authority (on-curve), not presaleConfig (PDA cannot be owner)
    treasury = new anchor.web3.PublicKey("BurrNY73jXSAFeHWxWpGnzQQ7tK7CJHS4wvLoxdEXxvj");
    // treasury = await createAccount(
    //   provider.connection,
    //   authority,
    //   tokenMint,
    //   authority.publicKey,
    //   undefined,
    //   { commitment: "confirmed" }
    // );

    // Set presaleConfig PDA as the authority of the treasury
    // This allows the PDA to transfer tokens from treasury in claim_vested instruction
    // We need to manually create the instruction and add the PDA as a signer
    // const setAuthorityIx = createSetAuthorityInstruction(
    //   treasury,
    //   authority.publicKey,
    //   AuthorityType.AccountOwner,
    //   presaleConfig
    // );
    
    // const transaction = new web3.Transaction().add(setAuthorityIx);
    // const signature = await web3.sendAndConfirmTransaction(
    //   provider.connection,
    //   transaction,
    //   [authority],
    //   { commitment: "confirmed" }
    // );
    // await provider.connection.confirmTransaction(signature, "confirmed");

    // Mint tokens to treasury
    // Use toString() and parse as BigInt to avoid JavaScript number overflow
    const totalTokensAmount = BigInt(TOTAL_TOKENS.toString());
    // await mintTo(
    //   provider.connection,
    //   authority,
    //   tokenMint,
    //   treasury,
    //   authority,
    //   totalTokensAmount
    // );

    // Create buyer token accounts
    buyer1TokenAccount = await createAccount(
      provider.connection,
      authority,
      tokenMint,
      buyer1.publicKey
    );
    console.log("buyer1 token account address ===>", buyer1TokenAccount.toString());

    buyer2TokenAccount = await createAccount(
      provider.connection,
      authority,
      tokenMint,
      buyer2.publicKey
    );
    console.log("buyer2 token account address ===>", buyer2TokenAccount.toString());

    // // Create buyer USDC accounts and mint USDC
    buyer1UsdcAccount = new anchor.web3.PublicKey("5GZw6E4kSnkhPU6YdNkchSQsk9dD4m6oH8iygY6kBE6Y");
    // buyer1UsdcAccount = await createAccount(
    //   provider.connection,
    //   authority,
    //   usdcMint,
    //   buyer1.publicKey
    // );
    buyer2UsdcAccount = new anchor.web3.PublicKey("ASXFNzAxptgn2auB8YE2cNXyASxmszDyKuQpdCLxCjME");
    // buyer2UsdcAccount = await createAccount(
    //   provider.connection,
    //   authority,
    //   usdcMint,
    //   buyer2.publicKey
    // );

    // Mint USDC to buyers
    // await mintTo(
    //   provider.connection,
    //   authority,
    //   usdcMint,
    //   buyer1UsdcAccount,
    //   authority,
    //   1_000_000 * 10 ** 6 // 1M USDC
    // );

    // await mintTo(
    //   provider.connection,
    //   authority,
    //   usdcMint,
    //   buyer2UsdcAccount,
    //   authority,
    //   1_000_000 * 10 ** 6 // 1M USDC
    // );

    console.log("Setup completed!");
    console.log("Authority:", authority.publicKey.toString());
    console.log("Token Mint:", tokenMint.toString());
    console.log("USDC Mint:", usdcMint.toString());
    console.log("Treasury:", treasury.toString());
    console.log("Presale Config:", presaleConfig.toString());
  });

  describe("Initialize", () => {
    it("Initializes the presale successfully", async () => {
      const now = Math.floor(Date.now() / 1000);
      const launchTimestamp = new BN(now + 86400 * 30); // 30 days from now

      // Create 10 phases
      const phases = Array(10)
        .fill(null)
        .map((_, i) => ({
          priceSol: new BN((100_000_000 + i * 10_000_000).toString()), // Increasing price
          priceUsdc: new BN((50_000 + i * 5_000).toString()), // Increasing price
          startTime: new BN(now + i * 86400 * 7), // Each phase 7 days apart
          endTime: new BN(now + (i + 1) * 86400 * 7),
          tokensAllocated: new BN(1_000_000).mul(new BN(10 ** 9)), // 1M per phase
          tokensSold: new BN(0),
        }));

      // Workaround: Create USDC vault manually to avoid Associated Token Program error
      // The contract tries to use ATA when it sees token::authority = PDA, which fails
      // We'll create it manually and then modify Anchor's transaction to use init_if_needed
      // Actually, since contract uses `init`, we need a different approach
      
      // The solution: Pre-create the account structure, then Initialize will fail on `init`
      // but we can catch that and handle it, OR we modify the transaction after Anchor builds it
      
      // Actually, the real fix is to intercept Anchor's transaction building
      // Let's try building the transaction manually and adding our account creation first
      // Workaround: Use .instruction() to get just the program instruction without ATA
      // Then manually create usdc_vault account and build transaction
      const initializeIx = await program.methods
        .initialize(
          phases,
          TOTAL_TOKENS,
          MAX_PER_TX,
          MAX_PER_WALLET,
          MIN_TIME_BETWEEN,
          launchTimestamp,
          VESTING_LAUNCH_PCT,
          VESTING_MONTHLY_PCT
        )
        .accounts({
          authority: authority.publicKey,
          tokenMint: tokenMint,
          usdcMint: usdcMint,
          treasury: treasury,
        } as any)
        .instruction();
      
      // Manually create usdc_vault account (PDA with presale_config as authority)
      // This must be done in the same transaction, with the program signing for the PDA
      const usdcVaultRent = await getMinimumBalanceForRentExemptAccount(provider.connection);
      const createAccountIx = web3.SystemProgram.createAccount({
        fromPubkey: authority.publicKey,
        newAccountPubkey: usdcVault,
        lamports: usdcVaultRent,
        space: 165,
        programId: TOKEN_PROGRAM_ID,
      });
      
      // Initialize token account - the program will sign for presale_config during initialize
      // We create it with authority temporarily, Initialize will change authority via setAuthority
      const initializeAccountIx = createInitializeAccountInstruction(
        usdcVault,
        usdcMint,
        authority.publicKey, // Temp authority
        TOKEN_PROGRAM_ID
      );
      
      // Build transaction: create account, init token account, then initialize program
      // The initialize instruction will handle setting presale_config as authority
      const transaction = new web3.Transaction().add(
        createAccountIx,
        initializeAccountIx,
        initializeIx
      );
      
      const { blockhash } = await provider.connection.getLatestBlockhash();
      transaction.recentBlockhash = blockhash;
      transaction.feePayer = authority.publicKey;
      
      transaction.sign(authority);
      const tx = await provider.connection.sendRawTransaction(transaction.serialize());
      await provider.connection.confirmTransaction(tx, "confirmed");

      console.log("Initialize transaction signature:", tx);

      // Verify presale config
      const config = await program.account.presaleConfig.fetch(presaleConfig);
      assert.equal(
        config.authority.toString(),
        authority.publicKey.toString()
      );
      assert.equal(config.tokenMint.toString(), tokenMint.toString());
      assert.equal(config.usdcMint.toString(), usdcMint.toString());
      assert.equal(config.treasury.toString(), treasury.toString());
      assert.equal(config.isPaused, false);
      assert.equal(config.totalTokensForSale.toString(), TOTAL_TOKENS.toString());
      assert.equal(config.tokensSold.toString(), "0");
      assert.equal(config.vestingLaunchPercentage, VESTING_LAUNCH_PCT);
      assert.equal(config.vestingMonthlyPercentage, VESTING_MONTHLY_PCT);
    });

    it("Fails to initialize twice", async () => {
      const now = Math.floor(Date.now() / 1000);
      const launchTimestamp = new BN(now + 86400 * 30);

      const phases = Array(10)
        .fill(null)
        .map((_, i) => ({
          priceSol: new BN("100000000"),
          priceUsdc: new BN("50000"),
          startTime: new BN(now + i * 86400 * 7),
          endTime: new BN(now + (i + 1) * 86400 * 7),
          tokensAllocated: new BN(1_000_000).mul(new BN(10 ** 9)),
          tokensSold: new BN(0),
        }));

      try {
        await program.methods
          .initialize(
            phases,
            TOTAL_TOKENS,
            MAX_PER_TX,
            MAX_PER_WALLET,
            MIN_TIME_BETWEEN,
            launchTimestamp,
            VESTING_LAUNCH_PCT,
            VESTING_MONTHLY_PCT
          )
          .accounts({
            authority: authority.publicKey,
            tokenMint: tokenMint,
            usdcMint: usdcMint,
            treasury: treasury,
          } as any)
          .signers([authority])
          .rpc();
        assert.fail("Should have failed");
      } catch (err) {
        assert.include(err.message, "already in use");
      }
    });
  });

  describe("Purchase with SOL", () => {
    it("Allows buyer to purchase tokens with SOL", async () => {
      const purchaseAmount = new BN(1000).mul(new BN(10 ** 9)); // 1000 tokens

      const buyer1BalanceBefore = await provider.connection.getBalance(
        buyer1.publicKey
      );

      const tx = await program.methods
        .purchase(purchaseAmount, { sol: {} })
        .accounts({
          buyer: buyer1.publicKey,
          buyerUsdcAccount: buyer1UsdcAccount,
        } as any)
        .signers([buyer1])
        .rpc();

      console.log("Purchase with SOL transaction:", tx);

      // Verify user purchase account
      const userPurchase = await program.account.userPurchase.fetch(
        buyer1Purchase
      );
      assert.equal(
        userPurchase.totalPurchased.toString(),
        purchaseAmount.toString()
      );
      assert.isTrue(userPurchase.totalSpentSol.toNumber() > 0);

      // Verify SOL was transferred
      const buyer1BalanceAfter = await provider.connection.getBalance(
        buyer1.publicKey
      );
      assert.isTrue(buyer1BalanceAfter < buyer1BalanceBefore);

      // Verify presale config updated
      const config = await program.account.presaleConfig.fetch(presaleConfig);
      assert.equal(config.tokensSold.toString(), purchaseAmount.toString());
    });

    it("Enforces maximum per transaction limit", async () => {
      const tooMuch = MAX_PER_TX.add(new BN(1));

      try {
        await program.methods
          .purchase(tooMuch, { sol: {} })
          .accounts({
            buyer: buyer1.publicKey,
            buyerUsdcAccount: buyer1UsdcAccount,
          } as any)
          .signers([buyer1])
          .rpc();
        assert.fail("Should have failed");
      } catch (err) {
        assert.include(err.message, "ExceedsMaxPerTransaction");
      }
    });

    it("Enforces minimum time between purchases", async () => {
      const purchaseAmount = new BN(100).mul(new BN(10 ** 9));

      try {
        await program.methods
          .purchase(purchaseAmount, { sol: {} })
          .accounts({
            buyer: buyer1.publicKey,
            buyerUsdcAccount: buyer1UsdcAccount,
          } as any)
          .signers([buyer1])
          .rpc();
        assert.fail("Should have failed");
      } catch (err) {
        assert.include(err.message, "TooSoonSinceLastPurchase");
      }
    });
  });

  describe("Purchase with USDC", () => {
    it("Allows buyer to purchase tokens with USDC", async () => {
      // Wait for cooldown
      await new Promise((resolve) => setTimeout(resolve, 61000));

      const purchaseAmount = new BN(500).mul(new BN(10 ** 9)); // 500 tokens

      const buyer2UsdcBefore = await getAccount(
        provider.connection,
        buyer2UsdcAccount
      );

      const tx = await program.methods
        .purchase(purchaseAmount, { usdc: {} })
        .accounts({
          buyer: buyer2.publicKey,
          buyerUsdcAccount: buyer2UsdcAccount,
        } as any)
        .signers([buyer2])
        .rpc();

      console.log("Purchase with USDC transaction:", tx);

      // Verify user purchase account
      const userPurchase = await program.account.userPurchase.fetch(
        buyer2Purchase
      );
      assert.equal(
        userPurchase.totalPurchased.toString(),
        purchaseAmount.toString()
      );
      assert.isTrue(userPurchase.totalSpentUsdc.toNumber() > 0);

      // Verify USDC was transferred
      const buyer2UsdcAfter = await getAccount(
        provider.connection,
        buyer2UsdcAccount
      );
      assert.isTrue(
        Number(buyer2UsdcAfter.amount) < Number(buyer2UsdcBefore.amount)
      );

      // Verify USDC vault received payment
      const usdcVaultAccount = await getAccount(
        provider.connection,
        usdcVault
      );
      assert.isTrue(Number(usdcVaultAccount.amount) > 0);
    });
  });

  describe("Pause/Unpause", () => {
    it("Allows authority to pause presale", async () => {
      const tx = await program.methods
        .pause()
        .accounts({
          authority: authority.publicKey,
        } as any)
        .signers([authority])
        .rpc();

      console.log("Pause transaction:", tx);

      const config = await program.account.presaleConfig.fetch(presaleConfig);
      assert.equal(config.isPaused, true);
    });

    it("Prevents purchases when paused", async () => {
      const purchaseAmount = new BN(100).mul(new BN(10 ** 9));

      try {
        await program.methods
          .purchase(purchaseAmount, { sol: {} })
          .accounts({
            buyer: buyer1.publicKey,
            buyerUsdcAccount: buyer1UsdcAccount,
          } as any)
          .signers([buyer1])
          .rpc();
        assert.fail("Should have failed");
      } catch (err) {
        assert.include(err.message, "PresalePaused");
      }
    });

    it("Allows authority to unpause presale", async () => {
      const tx = await program.methods
        .unpause()
        .accounts({
          authority: authority.publicKey,
        } as any)
        .signers([authority])
        .rpc();

      console.log("Unpause transaction:", tx);

      const config = await program.account.presaleConfig.fetch(presaleConfig);
      assert.equal(config.isPaused, false);
    });

    it("Prevents non-authority from pausing", async () => {
      try {
        await program.methods
          .pause()
          .accounts({
            authority: buyer1.publicKey,
          } as any)
          .signers([buyer1])
          .rpc();
        assert.fail("Should have failed");
      } catch (err) {
        assert.include(err.message, "Unauthorized");
      }
    });
  });

  describe("Update Config", () => {
    it("Allows authority to update configuration", async () => {
      const newMaxPerTx = new BN(20_000).mul(new BN(10 ** 9));
      const newMaxPerWallet = new BN(200_000).mul(new BN(10 ** 9));
      const newMinTime = new BN(30);

      const tx = await program.methods
        .updateConfig(newMaxPerTx, newMaxPerWallet, newMinTime)
        .accounts({
          authority: authority.publicKey,
        } as any)
        .signers([authority])
        .rpc();

      console.log("Update config transaction:", tx);

      const config = await program.account.presaleConfig.fetch(presaleConfig);
      assert.equal(
        config.maxPurchasePerTransaction.toString(),
        newMaxPerTx.toString()
      );
      assert.equal(
        config.maxPurchasePerWallet.toString(),
        newMaxPerWallet.toString()
      );
      assert.equal(
        config.minTimeBetweenPurchases.toString(),
        newMinTime.toString()
      );
    });

    it("Prevents non-authority from updating config", async () => {
      try {
        await program.methods
          .updateConfig(null, null, new BN(10))
          .accounts({
            authority: buyer1.publicKey,
          } as any)
          .signers([buyer1])
          .rpc();
        assert.fail("Should have failed");
      } catch (err) {
        assert.include(err.message, "Unauthorized");
      }
    });
  });

  describe("Get Purchase Status", () => {
    it("Returns purchase status for a user", async () => {
      const tx = await program.methods
        .getPurchaseStatus()
        .accounts({
          userWallet: buyer1.publicKey,
        } as any)
        .rpc();

      console.log("Get purchase status transaction:", tx);

      const userPurchase = await program.account.userPurchase.fetch(
        buyer1Purchase
      );
      console.log("User purchase data:", {
        totalPurchased: userPurchase.totalPurchased.toString(),
        totalSpentSol: userPurchase.totalSpentSol.toString(),
        totalSpentUsdc: userPurchase.totalSpentUsdc.toString(),
        lastPurchaseTime: userPurchase.lastPurchaseTime.toString(),
      });
    });
  });

  describe("Claim Vested", () => {
    it("Prevents claiming before vesting period", async () => {
      try {
        await program.methods
          .claimVested()
          .accounts({
            buyer: buyer1.publicKey,
            tokenMint: tokenMint,
            buyerTokenAccount: buyer1TokenAccount,
            treasury: treasury,
          } as any)
          .signers([buyer1])
          .rpc();
        assert.fail("Should have failed");
      } catch (err) {
        assert.include(err.message, "NoTokensToClaim");
      }
    });

    // Note: Testing actual claiming would require waiting for the vesting period
    // or manipulating time, which is not possible in standard Solana tests
  });

  describe("Summary", () => {
    it("Displays final presale statistics", async () => {
      const config = await program.account.presaleConfig.fetch(presaleConfig);
      const buyer1Data = await program.account.userPurchase.fetch(
        buyer1Purchase
      );
      const buyer2Data = await program.account.userPurchase.fetch(
        buyer2Purchase
      );

      console.log("\n=== Presale Summary ===");
      console.log("Total tokens for sale:", config.totalTokensForSale.toString());
      console.log("Total tokens sold:", config.tokensSold.toString());
      console.log("Presale paused:", config.isPaused);
      console.log("\n=== Buyer 1 ===");
      console.log("Total purchased:", buyer1Data.totalPurchased.toString());
      console.log("Total spent SOL:", buyer1Data.totalSpentSol.toString());
      console.log("Total spent USDC:", buyer1Data.totalSpentUsdc.toString());
      console.log("\n=== Buyer 2 ===");
      console.log("Total purchased:", buyer2Data.totalPurchased.toString());
      console.log("Total spent SOL:", buyer2Data.totalSpentSol.toString());
      console.log("Total spent USDC:", buyer2Data.totalSpentUsdc.toString());
    });
  });
});
