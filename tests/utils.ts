import * as anchor from "@coral-xyz/anchor";
import BN from "bn.js";
import { Connection, PublicKey, LAMPORTS_PER_SOL } from "@solana/web3.js";

/**
 * Helper function to wait for a specified number of seconds
 */
export async function sleep(seconds: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, seconds * 1000));
}

/**
 * Convert tokens with decimals to base units
 */
export function toTokenAmount(amount: number, decimals: number = 9): BN {
  return new BN(amount).mul(new BN(10 ** decimals));
}

/**
 * Convert base units to tokens with decimals
 */
export function fromTokenAmount(amount: BN, decimals: number = 9): number {
  return amount.div(new BN(10 ** decimals)).toNumber();
}

/**
 * Get current Unix timestamp
 */
export function getCurrentTimestamp(): number {
  return Math.floor(Date.now() / 1000);
}

/**
 * Create a future timestamp (days from now)
 */
export function getFutureTimestamp(days: number): BN {
  return new BN(getCurrentTimestamp() + days * 86400);
}

/**
 * Create presale phases for testing
 */
export function createTestPhases(
  startTime: number,
  phaseCount: number = 10,
  phaseDurationDays: number = 7
): any[] {
  return Array(phaseCount)
    .fill(null)
    .map((_, i) => ({
      priceSol: new BN((100_000_000 + i * 10_000_000).toString()), // 0.1 SOL + 0.01 per phase
      priceUsdc: new BN((50_000 + i * 5_000).toString()), // $0.05 + $0.005 per phase
      startTime: new BN(startTime + i * 86400 * phaseDurationDays),
      endTime: new BN(startTime + (i + 1) * 86400 * phaseDurationDays),
      tokensAllocated: toTokenAmount(1_000_000), // 1M tokens per phase
      tokensSold: new BN(0),
    }));
}

/**
 * Format lamports to SOL
 */
export function lamportsToSol(lamports: number): number {
  return lamports / LAMPORTS_PER_SOL;
}

/**
 * Format SOL to lamports
 */
export function solToLamports(sol: number): number {
  return sol * LAMPORTS_PER_SOL;
}

/**
 * Airdrop SOL to an account with retry logic
 */
export async function airdropSol(
  connection: Connection,
  publicKey: PublicKey,
  amount: number
): Promise<void> {
  const signature = await connection.requestAirdrop(
    publicKey,
    amount * LAMPORTS_PER_SOL
  );
  await connection.confirmTransaction(signature);
}

/**
 * Get account balance in SOL
 */
export async function getBalance(
  connection: Connection,
  publicKey: PublicKey
): Promise<number> {
  const balance = await connection.getBalance(publicKey);
  return lamportsToSol(balance);
}

/**
 * Log transaction details
 */
export function logTransaction(name: string, signature: string): void {
  console.log(`\n✅ ${name}`);
  console.log(`   Signature: ${signature}`);
}

/**
 * Log account info
 */
export function logAccountInfo(name: string, data: any): void {
  console.log(`\n📊 ${name}`);
  Object.entries(data).forEach(([key, value]) => {
    if (value instanceof BN) {
      console.log(`   ${key}: ${value.toString()}`);
    } else if (typeof value === "object" && value !== null) {
      console.log(`   ${key}: ${JSON.stringify(value, null, 2)}`);
    } else {
      console.log(`   ${key}: ${value}`);
    }
  });
}

/**
 * Assert BN equality with better error messages
 */
export function assertBNEqual(actual: BN, expected: BN, message?: string): void {
  if (!actual.eq(expected)) {
    throw new Error(
      `${message || "BN values not equal"}\nExpected: ${expected.toString()}\nActual: ${actual.toString()}`
    );
  }
}

/**
 * Assert BN greater than
 */
export function assertBNGreaterThan(
  actual: BN,
  expected: BN,
  message?: string
): void {
  if (!actual.gt(expected)) {
    throw new Error(
      `${message || "BN not greater than"}\nExpected > ${expected.toString()}\nActual: ${actual.toString()}`
    );
  }
}

/**
 * Assert BN less than
 */
export function assertBNLessThan(
  actual: BN,
  expected: BN,
  message?: string
): void {
  if (!actual.lt(expected)) {
    throw new Error(
      `${message || "BN not less than"}\nExpected < ${expected.toString()}\nActual: ${actual.toString()}`
    );
  }
}

