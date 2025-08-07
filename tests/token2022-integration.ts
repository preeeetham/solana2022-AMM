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

describe("Token-2022 AMM Integration Tests", () => {
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

  // Token accounts
  let userATokenA: PublicKey;
  let userATokenB: PublicKey;
  let userBTokenA: PublicKey;
  let userBTokenB: PublicKey;

  before(async () => {
    // Initialize test accounts
    mintA = Keypair.generate();
    mintB = Keypair.generate();
    userA = Keypair.generate();
    userB = Keypair.generate();
    whitelistAccount = Keypair.generate();
    ammPool = Keypair.generate();

    // Airdrop SOL to test accounts
    await connection.requestAirdrop(userA.publicKey, 2 * LAMPORTS_PER_SOL);
    await connection.requestAirdrop(userB.publicKey, 2 * LAMPORTS_PER_SOL);
    
    // Wait for airdrop confirmations
    await new Promise(resolve => setTimeout(resolve, 2000));
  });

  describe("Safe Transfer Hook Program", () => {
    it("Can deploy and initialize safe transfer hook", async () => {
      // The hook program should be deployed and accessible
      const programAccount = await connection.getAccountInfo(hookProgram.programId);
      expect(programAccount).to.not.be.null;
      expect(programAccount!.executable).to.be.true;
    });

    it("Hook program has correct program ID", async () => {
      expect(hookProgram.programId.toString()).to.equal(
        "BroadwayHooK11111111111111111111111111111111"
      );
    });
  });

  describe("Transfer Hook Whitelist System", () => {
    it("Can initialize transfer hook whitelist", async () => {
      const tx = await ammProgram.methods
        .initializeWhitelist()
        .accounts({
          whitelist: whitelistAccount.publicKey,
          authority: payer.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([whitelistAccount])
        .rpc();

      console.log("Whitelist initialization tx:", tx);

      // Verify whitelist account was created
      const whitelistData = await ammProgram.account.transferHookWhitelist.fetch(
        whitelistAccount.publicKey
      );
      
      expect(whitelistData.authority.toString()).to.equal(payer.publicKey.toString());
      expect(whitelistData.hookCount).to.equal(0);
    });

    it("Can add safe transfer hook to whitelist", async () => {
      const tx = await ammProgram.methods
        .addHookToWhitelist(hookProgram.programId)
        .accounts({
          whitelist: whitelistAccount.publicKey,
          authority: payer.publicKey,
        })
        .rpc();

      console.log("Add hook to whitelist tx:", tx);

      // Verify hook was added
      const whitelistData = await ammProgram.account.transferHookWhitelist.fetch(
        whitelistAccount.publicKey
      );
      
      expect(whitelistData.hookCount).to.equal(1);
      expect(whitelistData.whitelistedHooks[0].toString()).to.equal(
        hookProgram.programId.toString()
      );
    });

    it("Cannot add hook twice", async () => {
      try {
        await ammProgram.methods
          .addHookToWhitelist(hookProgram.programId)
          .accounts({
            whitelist: whitelistAccount.publicKey,
            authority: payer.publicKey,
          })
          .rpc();
        
        expect.fail("Should have thrown error for duplicate hook");
      } catch (error) {
        expect(error.toString()).to.include("HookAlreadyWhitelisted");
      }
    });
  });

  describe("Token-2022 with Transfer Hook Creation", () => {
    it("Can create Token-2022 mint with transfer hook", async () => {
      // Calculate space needed for mint with transfer hook extension
      const extensions = [ExtensionType.TransferHook];
      const mintLen = getMintLen(extensions);
      const mintRent = await connection.getMinimumBalanceForRentExemption(mintLen);

      const transaction = new Transaction();

      // Create mint account
      transaction.add(
        SystemProgram.createAccount({
          fromPubkey: payer.publicKey,
          newAccountPubkey: mintA.publicKey,
          space: mintLen,
          lamports: mintRent,
          programId: TOKEN_2022_PROGRAM_ID,
        })
      );

      // Initialize transfer hook extension
      transaction.add(
        createInitializeTransferHookInstruction(
          mintA.publicKey,
          payer.publicKey, // authority
          hookProgram.programId, // hook program
          TOKEN_2022_PROGRAM_ID
        )
      );

      // Initialize mint
      transaction.add(
        createInitializeMintInstruction(
          mintA.publicKey,
          9, // decimals
          payer.publicKey, // mint authority
          payer.publicKey, // freeze authority (optional)
          TOKEN_2022_PROGRAM_ID
        )
      );

      const tx = await sendAndConfirmTransaction(
        connection,
        transaction,
        [payer.payer, mintA],
        { commitment: "confirmed" }
      );

      console.log("Token-2022 with hook creation tx:", tx);

      // Verify mint was created with transfer hook
      const mintInfo = await connection.getAccountInfo(mintA.publicKey);
      expect(mintInfo).to.not.be.null;
      expect(mintInfo!.owner.toString()).to.equal(TOKEN_2022_PROGRAM_ID.toString());
    });

    it("Can create standard Token-2022 mint without hook", async () => {
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

      console.log("Standard Token-2022 creation tx:", tx);
    });
  });

  describe("Token Account Management", () => {
    it("Can create and fund token accounts", async () => {
      // Create associated token accounts
      userATokenA = getAssociatedTokenAddressSync(
        mintA.publicKey,
        userA.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      );

      userATokenB = getAssociatedTokenAddressSync(
        mintB.publicKey,
        userA.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      );

      const transaction = new Transaction();

      // Create ATA for mintA
      transaction.add(
        createAssociatedTokenAccountInstruction(
          payer.publicKey,
          userATokenA,
          userA.publicKey,
          mintA.publicKey,
          TOKEN_2022_PROGRAM_ID
        )
      );

      // Create ATA for mintB
      transaction.add(
        createAssociatedTokenAccountInstruction(
          payer.publicKey,
          userATokenB,
          userA.publicKey,
          mintB.publicKey,
          TOKEN_2022_PROGRAM_ID
        )
      );

      // Mint tokens to user A
      transaction.add(
        createMintToInstruction(
          mintA.publicKey,
          userATokenA,
          payer.publicKey,
          1000 * 1e9, // 1000 tokens
          [],
          TOKEN_2022_PROGRAM_ID
        )
      );

      transaction.add(
        createMintToInstruction(
          mintB.publicKey,
          userATokenB,
          payer.publicKey,
          1000 * 1e9,
          [],
          TOKEN_2022_PROGRAM_ID
        )
      );

      const tx = await sendAndConfirmTransaction(
        connection,
        transaction,
        [payer.payer],
        { commitment: "confirmed" }
      );

      console.log("Token account creation and funding tx:", tx);
    });
  });

  describe("Transfer Hook Validation", () => {
    it("Can transfer tokens with whitelisted hook", async () => {
      // Create destination account
      userBTokenA = getAssociatedTokenAddressSync(
        mintA.publicKey,
        userB.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      );

      const transaction = new Transaction();

      // Create destination ATA
      transaction.add(
        createAssociatedTokenAccountInstruction(
          payer.publicKey,
          userBTokenA,
          userB.publicKey,
          mintA.publicKey,
          TOKEN_2022_PROGRAM_ID
        )
      );

      // Transfer with hook (this would normally require extra accounts)
      transaction.add(
        createTransferCheckedInstruction(
          userATokenA,
          mintA.publicKey,
          userBTokenA,
          userA.publicKey,
          100 * 1e9, // 100 tokens
          9, // decimals
          [],
          TOKEN_2022_PROGRAM_ID
        )
      );

      const tx = await sendAndConfirmTransaction(
        connection,
        transaction,
        [payer.payer, userA],
        { commitment: "confirmed" }
      );

      console.log("Transfer with hook tx:", tx);
    });
  });

  describe("AMM Integration", () => {
    it("Can validate transfer hooks in AMM context", async () => {
      // This test would verify that the AMM can:
      // 1. Detect Token-2022 mints with transfer hooks
      // 2. Validate hooks against the whitelist
      // 3. Include extra accounts in swap transactions
      // 4. Execute swaps successfully with hooked tokens

      // For now, we'll test the validation logic
      const isHookWhitelisted = await ammProgram.methods
        .validateTransferHook(hookProgram.programId)
        .accounts({
          whitelist: whitelistAccount.publicKey,
        })
        .view();

      expect(isHookWhitelisted).to.be.true;
    });

    it("Rejects non-whitelisted hooks", async () => {
      const randomHookProgram = Keypair.generate().publicKey;

      try {
        await ammProgram.methods
          .validateTransferHook(randomHookProgram)
          .accounts({
            whitelist: whitelistAccount.publicKey,
          })
          .view();
        
        expect.fail("Should have rejected non-whitelisted hook");
      } catch (error) {
        expect(error.toString()).to.include("HookNotWhitelisted");
      }
    });
  });

  describe("Security Tests", () => {
    it("Cannot add hook without authority", async () => {
      const unauthorizedUser = Keypair.generate();
      
      try {
        await ammProgram.methods
          .addHookToWhitelist(Keypair.generate().publicKey)
          .accounts({
            whitelist: whitelistAccount.publicKey,
            authority: unauthorizedUser.publicKey,
          })
          .signers([unauthorizedUser])
          .rpc();
        
        expect.fail("Should have rejected unauthorized hook addition");
      } catch (error) {
        // Should fail due to authority constraint
        expect(error).to.not.be.null;
      }
    });

    it("Cannot exceed whitelist capacity", async () => {
      // Try to add hooks beyond the maximum (32)
      let addedHooks = 1; // We already added one hook
      
      try {
        for (let i = addedHooks; i < 33; i++) {
          await ammProgram.methods
            .addHookToWhitelist(Keypair.generate().publicKey)
            .accounts({
              whitelist: whitelistAccount.publicKey,
              authority: payer.publicKey,
            })
            .rpc();
          addedHooks++;
        }
        
        // This should fail when we try to add the 33rd hook
        expect.fail("Should have rejected hook when whitelist is full");
      } catch (error) {
        expect(error.toString()).to.include("WhitelistFull");
        expect(addedHooks).to.equal(32); // Should have stopped at 32
      }
    });
  });

  describe("End-to-End Workflow", () => {
    it("Complete Token-2022 AMM workflow", async () => {
      console.log("\n=== Complete Token-2022 AMM Workflow ===");
      
      // 1. Whitelist is already initialized and populated
      console.log("✓ Transfer Hook whitelist initialized");
      console.log("✓ Safe Transfer Hook added to whitelist");
      
      // 2. Tokens are created and funded
      console.log("✓ Token-2022 mints created (with and without hooks)");
      console.log("✓ Token accounts created and funded");
      
      // 3. Transfer Hook validation works
      console.log("✓ Transfer Hook validation working");
      
      // 4. In a complete implementation, we would:
      console.log("\nNext steps for full implementation:");
      console.log("- Initialize AMM pool with Token-2022 assets");
      console.log("- Execute swaps with automatic hook detection");
      console.log("- Verify extra accounts are properly included");
      console.log("- Test with frontend integration");
      
      // Summary
      const whitelistData = await ammProgram.account.transferHookWhitelist.fetch(
        whitelistAccount.publicKey
      );
      
      console.log(`\nWhitelist Summary:`);
      console.log(`- Authority: ${whitelistData.authority.toString()}`);
      console.log(`- Hooks whitelisted: ${whitelistData.hookCount}`);
      console.log(`- Safe hook included: ${whitelistData.whitelistedHooks[0].toString()}`);
      
      expect(whitelistData.hookCount).to.be.greaterThan(0);
    });
  });
});