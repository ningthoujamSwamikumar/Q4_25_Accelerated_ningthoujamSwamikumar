import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  TOKEN_2022_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  createTransferCheckedWithTransferHookInstruction,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountInstruction,
  createMintToInstruction,
} from "@solana/spl-token";
import { SendTransactionError, SystemProgram, Transaction, sendAndConfirmTransaction } from '@solana/web3.js';
import { WhitelistTransferHook } from "../target/types/whitelist_transfer_hook";

describe("whitelist-transfer-hook", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const wallet = provider.wallet as anchor.Wallet;

  const program = anchor.workspace.whitelistTransferHook as Program<WhitelistTransferHook>;

  const mint2022 = anchor.web3.Keypair.generate();

  // Sender token account address
  const sourceTokenAccount = getAssociatedTokenAddressSync(
    mint2022.publicKey,
    wallet.publicKey,
    false,
    TOKEN_2022_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
  );

  // Recipient token account address
  const recipient = anchor.web3.Keypair.generate();
  const destinationTokenAccount = getAssociatedTokenAddressSync(
    mint2022.publicKey,
    recipient.publicKey,
    false,
    TOKEN_2022_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
  );

  // ExtraAccountMetaList address
  // Store extra accounts required by the custom transfer hook instruction
  const [extraAccountMetaListPDA] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from('extra-account-metas'), mint2022.publicKey.toBuffer()],
    program.programId,
  );

  const whitelist_pda = anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from("whitelist"),
      provider.publicKey.toBuffer()
    ],
    program.programId
  )[0];

  it("Add user to whitelist", async () => {
    const tx = await program.methods.addToWhitelist(provider.publicKey)
      .accountsPartial({
        admin: provider.publicKey,
        whitelist: whitelist_pda,
      })
      .rpc();

    console.log("\nUser added to whitelist:", provider.publicKey.toBase58());
    console.log("Transaction signature:", tx);
  });

  it("Remove user to whitelist", async () => {
    const tx = await program.methods.removeFromWhitelist(provider.publicKey)
      .accountsPartial({
        admin: provider.publicKey,
        whitelist: whitelist_pda,
      })
      .rpc();

    console.log("\nUser removed from whitelist:", provider.publicKey.toBase58());
    console.log("Transaction signature:", tx);
  });

  it('Create Mint Account with Transfer Hook Extension', async () => {
    const txSig = await program.methods.initializeMintWithTransferHook()
      .accounts({
        mint: mint2022.publicKey,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        user: provider.publicKey,
      })
      .signers([wallet.payer, mint2022])
      .rpc({ commitment: "confirmed" });

    console.log("\nTransaction Signature: ", txSig);
  });

  it('Create Token Accounts and Mint Tokens', async () => {
    // 100 tokens
    const amount = 100 * 10 ** 9;

    const transaction = new Transaction().add(
      createAssociatedTokenAccountInstruction(
        wallet.publicKey,
        sourceTokenAccount,
        wallet.publicKey,
        mint2022.publicKey,
        TOKEN_2022_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID,
      ),
      createAssociatedTokenAccountInstruction(
        wallet.publicKey,
        destinationTokenAccount,
        recipient.publicKey,
        mint2022.publicKey,
        TOKEN_2022_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID,
      ),
      createMintToInstruction(mint2022.publicKey, sourceTokenAccount, wallet.publicKey, amount, [], TOKEN_2022_PROGRAM_ID),
    );

    const txSig = await sendAndConfirmTransaction(provider.connection, transaction, [wallet.payer], { skipPreflight: true });

    console.log("\nTransaction Signature: ", txSig);
  });

  // Account to store extra accounts required by the transfer hook instruction
  it('Create ExtraAccountMetaList Account', async () => {
    const initializeExtraAccountMetaListInstruction = await program.methods
      .initializeTransferHook()
      .accountsPartial({
        payer: wallet.publicKey,
        mint: mint2022.publicKey,
        extraAccountMetaList: extraAccountMetaListPDA,
        systemProgram: SystemProgram.programId,
      })
      .instruction();
    //.rpc();

    const transaction = new Transaction().add(initializeExtraAccountMetaListInstruction);

    const txSig = await sendAndConfirmTransaction(provider.connection, transaction, [wallet.payer], { skipPreflight: true, commitment: 'confirmed' });
    console.log("\nExtraAccountMetaList Account created:", extraAccountMetaListPDA.toBase58());
    console.log('Transaction Signature:', txSig);
  });

  it('Transfer Hook with Extra Account Meta', async () => {
    // whitelist the user 
    await program.methods.addToWhitelist(provider.publicKey)
      .accountsPartial({
        admin: provider.publicKey,
        whitelist: whitelist_pda,
      })
      .rpc();

    // 1 tokens
    const amount = 1 * 10 ** 9;
    const amountBigInt = BigInt(amount);

    const transferInstructionWithHelper = await createTransferCheckedWithTransferHookInstruction(
      provider.connection,
      sourceTokenAccount,
      mint2022.publicKey,
      destinationTokenAccount,
      wallet.publicKey,
      amountBigInt,
      9,
      [],
      'confirmed',
      TOKEN_2022_PROGRAM_ID,
    );

    const transaction = new Transaction().add(transferInstructionWithHelper);

    try {
      // Send the transaction
      const txSig = await sendAndConfirmTransaction(provider.connection, transaction, [wallet.payer], { skipPreflight: false });
      console.log("\nTransfer Signature:", txSig);
    }
    catch (error) {
      if (error instanceof SendTransactionError) {
        console.error("\nTransaction failed:", error.logs[4]);
      } else {
        console.error("\nUnexpected error:", error);
      }
    }
  });

});
