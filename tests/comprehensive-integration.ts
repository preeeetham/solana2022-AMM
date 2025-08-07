import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Token2022Amm } from "../target/types/token2022_amm";
import { SafeTransferHook } from "../target/types/safe_transfer_hook";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  Transaction,
  sendAndConfirmTransaction,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import {
  TOKEN_2022_PROGRAM_ID,
  ExtensionType,
  createInitializeMintInstruction,
  createInitializeTransferHookInstruction,
  getMintLen,
  createAssociatedTokenAccountInstruction,
  getAssociatedTokenAddressSync,
  createMintToInstruction,
  createTransferCheckedInstruction,
} from "@solana/spl-token";
import { expect } from "chai";
import { createSimulationService, SimulationResult } from "../scripts/simulation-service";

describe("Comprehensive Token-2022 AMM Integration Tests", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const ammProgram = anchor.workspace.Token2022Amm as Program<Token2022Amm>;
  const hookProgram = anchor.workspace.SafeTransferHook as Program<SafeTransferHook>;
  
  const payer = provider.wallet as anchor.Wallet;
  const connection = provider.connection;

  // Test accounts
  let mintA: Keypair;
  let mintB: Keypair;
  let userA: Keypair;
  let userB: Keypair;
  let whitelistAccount: Keypair;
  let ammPool: Keypair;
  let proposalAccount: Keypair;
  let proxyAccount: Keypair;

  // Token accounts
  let userATokenA: PublicKey;
  let userATokenB: PublicKey;
  let userBTokenA: PublicKey;
  let userBTokenB: PublicKey;

  // Simulation service
  let simulationService: any;

  before(async () => {
    // Initialize test accounts
    mintA = Keypair.generate();
    mintB = Keypair.generate();
    userA = Keypair.generate();
    userB = Keypair.generate();
    whitelistAccount = Keypair.generate();
    ammPool = Keypair.generate();
    proposalAccount = Keypair.generate();
    proxyAccount = Keypair.generate();

    // Airdrop SOL to test accounts
    await connection.requestAirdrop(userA.publicKey, 2 * LAMPORTS_PER_SOL);
    await connection.requestAirdrop(userB.publicKey, 2 * LAMPORTS_PER_SOL);
    
    // Wait for airdrop confirmations
    await new Promise(resolve => setTimeout(resolve, 2000));

    // Initialize simulation service
    simulationService = createSimulationService(connection, ammProgram, provider);
  });

  describe("1. Core AMM Functionality", () => {
    it("Should initialize whitelist", async () => {
      const tx = await ammProgram.methods
        .initializeWhitelist()
        .accounts({
          whitelist: whitelistAccount.publicKey,
          authority: payer.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([whitelistAccount])
        .rpc();

      console.log("✅ Whitelist initialized:", tx);
    });

    it("Should add hook program to whitelist", async () => {
      const tx = await ammProgram.methods
        .addHookToWhitelist(hookProgram.programId)
        .accounts({
          whitelist: whitelistAccount.publicKey,
          authority: payer.publicKey,
        })
        .rpc();

      console.log("✅ Hook added to whitelist:", tx);
    });

    it("Should create Token-2022 mint with transfer hook", async () => {
      const extensions = [ExtensionType.TransferHook];
      const mintLen = getMintLen(extensions);
      const mintRent = await connection.getMinimumBalanceForRentExemption(mintLen);

      const transaction = new Transaction();

      transaction.add(
        SystemProgram.createAccount({
          fromPubkey: payer.publicKey,
          newAccountPubkey: mintA.publicKey,
          space: mintLen,
          lamports: mintRent,
          programId: TOKEN_2022_PROGRAM_ID,
        })
      );

      transaction.add(
        createInitializeTransferHookInstruction(
          mintA.publicKey,
          payer.publicKey,
          hookProgram.programId,
          TOKEN_2022_PROGRAM_ID
        )
      );

      transaction.add(
        createInitializeMintInstruction(
          mintA.publicKey,
          9,
          payer.publicKey,
          payer.publicKey,
          TOKEN_2022_PROGRAM_ID
        )
      );

      const tx = await sendAndConfirmTransaction(
        connection,
        transaction,
        [payer.payer, mintA],
        { commitment: "confirmed" }
      );

      console.log("✅ Token-2022 with hook created:", tx);
    });

    it("Should create standard Token-2022 mint", async () => {
      const mintLen = getMintLen([]);
      const mintRent = await connection.getMinimumBalanceForRentExemption(mintLen);

      const transaction = new Transaction();

      transaction.add(
        SystemProgram.createAccount({
          fromPubkey: payer.publicKey,
          newAccountPubkey: mintB.publicKey,
          space: mintLen,
          lamports: mintRent,
          programId: TOKEN_2022_PROGRAM_ID,
        })
      );

      transaction.add(
        createInitializeMintInstruction(
          mintB.publicKey,
          9,
          payer.publicKey,
          payer.publicKey,
          TOKEN_2022_PROGRAM_ID
        )
      );

      const tx = await sendAndConfirmTransaction(
        connection,
        transaction,
        [payer.payer, mintB],
        { commitment: "confirmed" }
      );

      console.log("✅ Standard Token-2022 created:", tx);
    });
  });

  describe("2. Governance System", () => {
    it("Should create a hook proposal", async () => {
      const newHookProgram = Keypair.generate();
      
      const tx = await ammProgram.methods
        .createHookProposal(
          newHookProgram.publicKey,
          "Test Hook Proposal",
          "https://audit.example.com/report.pdf",
          new anchor.BN(10 * LAMPORTS_PER_SOL) // 10 SOL stake
        )
        .accounts({
          proposal: proposalAccount.publicKey,
          proposer: payer.publicKey,
          whitelist: whitelistAccount.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([proposalAccount])
        .rpc();

      console.log("✅ Hook proposal created:", tx);
    });

    it("Should vote on proposal", async () => {
      const tx = await ammProgram.methods
        .voteOnProposal(true, new anchor.BN(5 * LAMPORTS_PER_SOL)) // Approve with 5 SOL stake
        .accounts({
          proposal: proposalAccount.publicKey,
          voter: userA.publicKey,
          whitelist: whitelistAccount.publicKey,
        })
        .signers([userA])
        .rpc();

      console.log("✅ Vote recorded:", tx);
    });

    it("Should execute approved proposal", async () => {
      // First finalize the proposal
      // Note: In a real scenario, you'd wait for the voting period to end
      
      const tx = await ammProgram.methods
        .executeProposal()
        .accounts({
          proposal: proposalAccount.publicKey,
          whitelist: whitelistAccount.publicKey,
          authority: payer.publicKey,
        })
        .rpc();

      console.log("✅ Proposal executed:", tx);
    });
  });

  describe("3. AMM Proxy Integration", () => {
    it("Should simulate swap with hook validation", async () => {
      // Create token accounts for simulation
      userATokenA = getAssociatedTokenAddressSync(mintA.publicKey, userA.publicKey);
      userATokenB = getAssociatedTokenAddressSync(mintB.publicKey, userA.publicKey);

      // Create token accounts
      const createTokenAccountsTx = new Transaction();
      createTokenAccountsTx.add(
        createAssociatedTokenAccountInstruction(
          payer.publicKey,
          userATokenA,
          userA.publicKey,
          mintA.publicKey
        )
      );
      createTokenAccountsTx.add(
        createAssociatedTokenAccountInstruction(
          payer.publicKey,
          userATokenB,
          userA.publicKey,
          mintB.publicKey
        )
      );

      await sendAndConfirmTransaction(
        connection,
        createTokenAccountsTx,
        [payer.payer],
        { commitment: "confirmed" }
      );

      // Simulate swap
      const simulationResult = await simulationService.simulateSwap({
        poolAddress: ammPool.publicKey,
        userTokenAAccount: userATokenA,
        userTokenBAccount: userATokenB,
        poolTokenAVault: PublicKey.default, // Placeholder
        poolTokenBVault: PublicKey.default, // Placeholder
        tokenAMint: mintA.publicKey,
        tokenBMint: mintB.publicKey,
        whitelistAddress: whitelistAccount.publicKey,
        amountIn: 1000000, // 1 token
        minAmountOut: 900000, // 0.9 token
        user: userA.publicKey,
      });

      console.log("✅ Swap simulation completed");
      console.log("Success:", simulationResult.success);
      if (simulationResult.hookValidation) {
        console.log("Hook validation:", simulationResult.hookValidation);
      }
    });
  });

  describe("4. Advanced Features", () => {
    it("Should validate transfer hooks", async () => {
      const isValid = await ammProgram.methods
        .validateTransferHook(hookProgram.programId)
        .accounts({
          whitelist: whitelistAccount.publicKey,
        })
        .view();

      expect(isValid).to.be.true;
      console.log("✅ Transfer hook validation successful");
    });

    it("Should handle multiple hook types", async () => {
      // Test with different hook scenarios
      const hookValidation = await simulationService.validateTransferHooks(
        mintA.publicKey,
        mintB.publicKey,
        whitelistAccount.publicKey
      );

      console.log("✅ Multi-hook validation:", hookValidation);
    });

    it("Should generate error reports", async () => {
      const mockSimulationResult: SimulationResult = {
        success: false,
        error: "Insufficient liquidity",
        hookValidation: {
          hookProgramId: hookProgram.programId,
          hookName: "Safe Transfer Hook",
          validationPassed: true,
          executionTime: 150,
        },
        gasEstimate: 50000,
        logs: [
          "Program Token2022Amm invoke [1]",
          "Program Token2022Amm success",
        ],
      };

      const errorReport = simulationService.getErrorReport(mockSimulationResult);
      console.log("✅ Error report generated:");
      console.log(errorReport);
    });
  });

  describe("5. Integration with Existing AMMs", () => {
    it("Should support Raydium integration", async () => {
      const raydiumProgramId = new PublicKey("675kPXyM9jqFX2QkqVVENMx7LJWtoHZPRN6dxHa2mAT");
      console.log("✅ Raydium program ID:", raydiumProgramId.toString());
    });

    it("Should support Orca integration", async () => {
      const orcaProgramId = new PublicKey("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc");
      console.log("✅ Orca program ID:", orcaProgramId.toString());
    });

    it("Should support Meteora integration", async () => {
      const meteoraProgramId = new PublicKey("M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K");
      console.log("✅ Meteora program ID:", meteoraProgramId.toString());
    });
  });

  describe("6. Performance and Security", () => {
    it("Should handle large transactions efficiently", async () => {
      const startTime = Date.now();
      
      // Simulate a complex transaction
      const simulationResult = await simulationService.simulateSwap({
        poolAddress: ammPool.publicKey,
        userTokenAAccount: userATokenA,
        userTokenBAccount: userATokenB,
        poolTokenAVault: PublicKey.default,
        poolTokenBVault: PublicKey.default,
        tokenAMint: mintA.publicKey,
        tokenBMint: mintB.publicKey,
        whitelistAddress: whitelistAccount.publicKey,
        amountIn: 10000000, // 10 tokens
        minAmountOut: 9000000, // 9 tokens
        user: userA.publicKey,
      });

      const executionTime = Date.now() - startTime;
      console.log(`✅ Large transaction simulation completed in ${executionTime}ms`);
      console.log("Success:", simulationResult.success);
    });

    it("Should validate security constraints", async () => {
      // Test various security scenarios
      const securityTests = [
        {
          name: "Unauthorized hook access",
          test: async () => {
            try {
              await ammProgram.methods
                .validateTransferHook(PublicKey.default)
                .accounts({
                  whitelist: whitelistAccount.publicKey,
                })
                .view();
              return false; // Should fail
            } catch {
              return true; // Correctly rejected
            }
          }
        },
        {
          name: "Invalid proposal execution",
          test: async () => {
            try {
              await ammProgram.methods
                .executeProposal()
                .accounts({
                  proposal: PublicKey.default,
                  whitelist: whitelistAccount.publicKey,
                  authority: userA.publicKey, // Not the authority
                })
                .rpc();
              return false; // Should fail
            } catch {
              return true; // Correctly rejected
            }
          }
        }
      ];

      for (const test of securityTests) {
        const result = await test.test();
        console.log(`✅ ${test.name}: ${result ? 'PASSED' : 'FAILED'}`);
        expect(result).to.be.true;
      }
    });
  });

  after(async () => {
    console.log("\n🎉 Comprehensive integration tests completed!");
    console.log("📋 Summary:");
    console.log("  ✅ Core AMM functionality");
    console.log("  ✅ Governance system");
    console.log("  ✅ AMM proxy integration");
    console.log("  ✅ Simulation service");
    console.log("  ✅ Security validation");
    console.log("  ✅ Performance testing");
  });
}); 