
import * as anchor from "@coral-xyz/anchor";
import { web3 } from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { GetCommitmentSignature } from "@magicblock-labs/ephemeral-rollups-sdk";
import { ErStateAccount } from "../target/types/er_state_account";
import { assert } from "chai";

describe.only("er-state-account", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const providerEphemeralRollup = new anchor.AnchorProvider(
    new anchor.web3.Connection(process.env.EPHEMERAL_PROVIDER_ENDPOINT || "https://devnet.magicblock.app/", { wsEndpoint: process.env.EPHEMERAL_WS_ENDPOINT || "wss://devnet.magicblock.app/" }
    ),
    anchor.Wallet.local()
  );

  console.log("Base Layer Connection: ", provider.connection.rpcEndpoint);
  console.log("Ephemeral Rollup Connection: ", providerEphemeralRollup.connection.rpcEndpoint);
  console.log(`Current SOL Public Key: ${anchor.Wallet.local().publicKey}`)

  before(async function () {
    const balance = await provider.connection.getBalance(anchor.Wallet.local().publicKey)
    console.log('Current balance is', balance / web3.LAMPORTS_PER_SOL, ' SOL', '\n')
  })

  const program = anchor.workspace.erStateAccount as Program<ErStateAccount>;
  console.log("programId: ", program.programId);

  const ephemeralProgram = (new Program(program.idl, providerEphemeralRollup)) as Program<ErStateAccount>;

  const userAccount = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("user"), anchor.Wallet.local().publicKey.toBuffer()],
    program.programId
  )[0];

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().accountsPartial({
      user: anchor.Wallet.local().publicKey,
      userAccount: userAccount,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
      .rpc();
    console.log("User Account initialized: ", tx);
  });

  it("Update State!", async () => {
    const tx = await program.methods.update(new anchor.BN(42)).accountsPartial({
      user: anchor.Wallet.local().publicKey,
      userAccount: userAccount,
    })
      .rpc();
    console.log("\nUser Account State Updated: ", tx);
  });

  it("Update State with Random value!", async () => {
    let userAccountState = await program.account.userAccount.fetch(userAccount);
    let clientSeed = Math.random() * 10;
    const tx = await program.methods.updateRandom(clientSeed).accounts({
      user: anchor.Wallet.local().publicKey,
      userAccount
    }).rpc({ commitment: "confirmed" });
    console.log("\nState Updated with Ranom value: ", tx);
    const lateshBlockhash = await provider.connection.getLatestBlockhash();
    await provider.connection.confirmTransaction({
      blockhash: lateshBlockhash.blockhash,
      lastValidBlockHeight: lateshBlockhash.lastValidBlockHeight,
      signature: tx
    });
    const userAccountStatePostUpdate = await program.account.userAccount.fetch(userAccount);
    assert(userAccountState.data !== userAccountStatePostUpdate.data, "Account state likely is not updated!");
  })

  it("Delegate to Ephemeral Rollup!", async () => {

    let tx = await program.methods.delegate().accountsPartial({
      user: anchor.Wallet.local().publicKey,
      userAccount: userAccount,
      validator: new web3.PublicKey("MAS1Dt9qreoRMQ14YQuhg8UTZMMzDdKhmkZMECCzk57"),
      systemProgram: anchor.web3.SystemProgram.programId,
    }).rpc({ skipPreflight: true });

    console.log("\nUser Account Delegated to Ephemeral Rollup: ", tx);
  });

  it("Update State and Commit to Base Layer!", async () => {
    let tx = await program.methods.updateCommit(new anchor.BN(43)).accounts({
      user: providerEphemeralRollup.wallet.publicKey,
      //userAccount: userAccount,
    })
      .transaction();

    tx.feePayer = providerEphemeralRollup.wallet.publicKey;

    tx.recentBlockhash = (await providerEphemeralRollup.connection.getLatestBlockhash()).blockhash;
    tx = await providerEphemeralRollup.wallet.signTransaction(tx);
    const txHash = await providerEphemeralRollup.sendAndConfirm(tx, [], { skipPreflight: false });
    const txCommitSgn = await GetCommitmentSignature(
      txHash,
      providerEphemeralRollup.connection
    );

    console.log("\nUser Account State Updated: ", txHash);
  });

  it("Update Delegated State with Random value!", async () => {
    let userAccountState = await providerEphemeralRollup.connection.getAccountInfo(userAccount);
    const userAccountData = program.coder.accounts.decode("userAccount", userAccountState.data) as anchor.IdlAccounts<ErStateAccount>["userAccount"];

    console.log("delegated state before random update: ", userAccountData);

    let clientSeed = Math.random() * 10;
    const tx = await ephemeralProgram.methods.updateRandomDelegated(clientSeed).accounts({
      user: anchor.Wallet.local().publicKey,
      userAccount: userAccount,
    }).rpc({ commitment: "confirmed" });
    console.log("\nState Updated with Ranom value: ", tx);
    const lateshBlockhash = await providerEphemeralRollup.connection.getLatestBlockhash();
    await provider.connection.confirmTransaction({
      blockhash: lateshBlockhash.blockhash,
      lastValidBlockHeight: lateshBlockhash.lastValidBlockHeight,
      signature: tx
    });

    const userAccountStatePostUpdate = await providerEphemeralRollup.connection.getAccountInfo(userAccount);
    const userAccountDataPostUpdate = program.coder.accounts.decode("userAccount", userAccountStatePostUpdate.data) as anchor.IdlAccounts<ErStateAccount>["userAccount"];

    console.log("delegated state after random update: ", userAccountDataPostUpdate);

    assert(userAccountData.data.eq(userAccountDataPostUpdate.data), "Account state likely is not updated!");
  })

  it("Commit and undelegate from Ephemeral Rollup!", async () => {
    let info = await providerEphemeralRollup.connection.getAccountInfo(userAccount);

    console.log("User Account Info: ", info);

    console.log("User account", userAccount.toBase58());

    let tx = await program.methods.undelegate().accounts({
      user: providerEphemeralRollup.wallet.publicKey,
    })
      .transaction();

    tx.feePayer = providerEphemeralRollup.wallet.publicKey;

    tx.recentBlockhash = (await providerEphemeralRollup.connection.getLatestBlockhash()).blockhash;
    tx = await providerEphemeralRollup.wallet.signTransaction(tx);
    const txHash = await providerEphemeralRollup.sendAndConfirm(tx, [], { skipPreflight: false });
    const txCommitSgn = await GetCommitmentSignature(
      txHash,
      providerEphemeralRollup.connection
    );

    console.log("\nUser Account Undelegated: ", txHash);
  });

  it("Update State!", async () => {
    let tx = await program.methods.update(new anchor.BN(45)).accountsPartial({
      user: anchor.Wallet.local().publicKey,
      userAccount: userAccount,
    })
      .rpc();

    console.log("\nUser Account State Updated: ", tx);
  });

  it("Close Account!", async () => {
    const tx = await program.methods.close().accountsPartial({
      user: anchor.Wallet.local().publicKey,
      userAccount: userAccount,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
      .rpc();
    console.log("\nUser Account Closed: ", tx);
  });
});
