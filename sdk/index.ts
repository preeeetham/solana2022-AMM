import { Connection, PublicKey, Transaction, sendAndConfirmTransaction } from '@solana/web3.js';
import { Program, AnchorProvider, web3, BN } from '@coral-xyz/anchor';
import { TOKEN_PROGRAM_ID, TOKEN_2022_PROGRAM_ID } from '@solana/spl-token';

/**
 * Token-2022 AMM SDK
 * Provides TypeScript utilities for interacting with the Token-2022 AMM protocol
 */
export class Token2022AmmSDK {
  private connection: Connection;
  private program: Program;
  private hookProgram: Program;
  private provider: AnchorProvider;

  constructor(
    connection: Connection,
    program: Program,
    hookProgram: Program,
    provider: AnchorProvider
  ) {
    this.connection = connection;
    this.program = program;
    this.hookProgram = hookProgram;
    this.provider = provider;
  }

  /**
   * Add a transfer hook program to the whitelist
   */
  async addHookToWhitelist(
    whitelistAddress: PublicKey,
    hookProgramId: PublicKey,
    authority: web3.Keypair
  ): Promise<string> {
    const tx = await this.program.methods
      .addHookToWhitelist(hookProgramId)
      .accounts({
        whitelist: whitelistAddress,
        authority: authority.publicKey,
      })
      .signers([authority])
      .rpc();

    return tx;
  }

  /**
   * Remove a transfer hook program from the whitelist
   */
  async removeHookFromWhitelist(
    whitelistAddress: PublicKey,
    hookProgramId: PublicKey,
    authority: web3.Keypair
  ): Promise<string> {
    const tx = await this.program.methods
      .removeHookFromWhitelist(hookProgramId)
      .accounts({
        whitelist: whitelistAddress,
        authority: authority.publicKey,
      })
      .signers([authority])
      .rpc();

    return tx;
  }

  /**
   * Execute a swap
   */
  async swap(
    poolAddress: PublicKey,
    amountIn: number,
    minAmountOut: number,
    userTokenAAccount: PublicKey,
    userTokenBAccount: PublicKey,
    poolTokenAVault: PublicKey,
    poolTokenBVault: PublicKey,
    tokenAMint: PublicKey,
    tokenBMint: PublicKey,
    whitelistAddress: PublicKey,
    user: web3.Keypair
  ): Promise<string> {
    const tx = await this.program.methods
      .swap(new BN(amountIn), new BN(minAmountOut))
      .accounts({
        pool: poolAddress,
        user: user.publicKey,
        userTokenA: userTokenAAccount,
        userTokenB: userTokenBAccount,
        poolTokenAVault: poolTokenAVault,
        poolTokenBVault: poolTokenBVault,
        tokenAMint: tokenAMint,
        tokenBMint: tokenBMint,
        whitelist: whitelistAddress,
        tokenProgram: TOKEN_PROGRAM_ID,
        token2022Program: TOKEN_2022_PROGRAM_ID,
      })
      .signers([user])
      .rpc();

    return tx;
  }

  /**
   * Add liquidity to a pool
   */
  async addLiquidity(
    poolAddress: PublicKey,
    amountA: number,
    amountB: number,
    minLpTokens: number,
    userTokenAAccount: PublicKey,
    userTokenBAccount: PublicKey,
    userLpTokenAccount: PublicKey,
    poolTokenAVault: PublicKey,
    poolTokenBVault: PublicKey,
    lpMint: PublicKey,
    tokenAMint: PublicKey,
    tokenBMint: PublicKey,
    whitelistAddress: PublicKey,
    user: web3.Keypair
  ): Promise<string> {
    const tx = await this.program.methods
      .addLiquidity(
        new BN(amountA),
        new BN(amountB),
        new BN(minLpTokens)
      )
      .accounts({
        pool: poolAddress,
        user: user.publicKey,
        userTokenA: userTokenAAccount,
        userTokenB: userTokenBAccount,
        userLpToken: userLpTokenAccount,
        poolTokenAVault: poolTokenAVault,
        poolTokenBVault: poolTokenBVault,
        lpMint: lpMint,
        tokenAMint: tokenAMint,
        tokenBMint: tokenBMint,
        whitelist: whitelistAddress,
        tokenProgram: TOKEN_PROGRAM_ID,
        token2022Program: TOKEN_2022_PROGRAM_ID,
      })
      .signers([user])
      .rpc();

    return tx;
  }

  /**
   * Remove liquidity from a pool
   */
  async removeLiquidity(
    poolAddress: PublicKey,
    lpTokensToBurn: number,
    minTokenA: number,
    minTokenB: number,
    userTokenAAccount: PublicKey,
    userTokenBAccount: PublicKey,
    userLpTokenAccount: PublicKey,
    poolTokenAVault: PublicKey,
    poolTokenBVault: PublicKey,
    lpMint: PublicKey,
    tokenAMint: PublicKey,
    tokenBMint: PublicKey,
    whitelistAddress: PublicKey,
    user: web3.Keypair
  ): Promise<string> {
    const tx = await this.program.methods
      .removeLiquidity(
        new BN(lpTokensToBurn),
        new BN(minTokenA),
        new BN(minTokenB)
      )
      .accounts({
        pool: poolAddress,
        user: user.publicKey,
        userTokenA: userTokenAAccount,
        userTokenB: userTokenBAccount,
        userLpToken: userLpTokenAccount,
        poolTokenAVault: poolTokenAVault,
        poolTokenBVault: poolTokenBVault,
        lpMint: lpMint,
        tokenAMint: tokenAMint,
        tokenBMint: tokenBMint,
        whitelist: whitelistAddress,
        tokenProgram: TOKEN_PROGRAM_ID,
        token2022Program: TOKEN_2022_PROGRAM_ID,
      })
      .signers([user])
      .rpc();

    return tx;
  }

  /**
   * Validate a transfer hook
   */
  async validateTransferHook(
    whitelistAddress: PublicKey,
    hookProgramId: PublicKey
  ): Promise<boolean> {
    try {
      const result = await this.program.methods
        .validateTransferHook(hookProgramId)
        .accounts({
          whitelist: whitelistAddress,
        })
        .view();

      return result;
    } catch (error) {
      console.error('Error validating transfer hook:', error);
      return false;
    }
  }
}

/**
 * Helper function to create SDK instance
 */
export function createToken2022AmmSDK(
  connection: Connection,
  program: Program,
  hookProgram: Program,
  provider: AnchorProvider
): Token2022AmmSDK {
  return new Token2022AmmSDK(connection, program, hookProgram, provider);
} 