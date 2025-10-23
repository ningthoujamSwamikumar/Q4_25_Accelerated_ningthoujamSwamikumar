
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

    console.log("delegated state before random update: ", userAccountData.data.toNumber());

    let clientSeed = Math.random() * 10;
    const tx = await ephemeralProgram.methods.updateRandomDelegated(clientSeed).accounts({
      user: anchor.Wallet.local().publicKey,
      userAccount: userAccount,
    }).rpc({ commitment: "confirmed" });
    console.log("\nState Updated with Ranom value: ", tx);
    const lateshBlockhash = await providerEphemeralRollup.connection.getLatestBlockhash();
    await providerEphemeralRollup.connection.confirmTransaction({
      blockhash: lateshBlockhash.blockhash,
      lastValidBlockHeight: lateshBlockhash.lastValidBlockHeight,
      signature: tx
    });

    await new Promise(resolve => setTimeout(resolve, 3000));

    const userAccountStatePostUpdate = await providerEphemeralRollup.connection.getAccountInfo(userAccount);
    const userAccountDataPostUpdate = ephemeralProgram.coder.accounts.decode("userAccount", userAccountStatePostUpdate.data) as anchor.IdlAccounts<ErStateAccount>["userAccount"];

    console.log("delegated state after random update: ", userAccountDataPostUpdate.data.toNumber());
    assert(!userAccountData.data.eq(userAccountDataPostUpdate.data), "Account state likely is not updated!");
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

/*
User Account initialized:  4cHcZRD1i4mPxbkg1KQLJdGrzHux3E6bAyHzvMXzyKc6vbeBRTsTQD1KmyGyp8a1L2VWHkFcHZSr9fviwyGLfXUx
    ✔ Is initialized! (3309ms)

User Account State Updated:  3Qwm9ZmgYXRgdFzVEZgGQELetHmFeDhXcoxLXZ3swumQXdj38Qoxt6HFtD65tgvPZYZdtjxBSoX6S6oJ2FtD8JNo
    ✔ Update State! (3882ms)

State Updated with Ranom value:  BuSEuGmRUSGQe7M288dvY1XCfrs2RBkHdd4gTZP6QrHdjnud2bquFAZQ93mqQU76uR35N4pHvcHpL3HHmMKWtWg
    ✔ Update State with Random value! (7730ms)

User Account Delegated to Ephemeral Rollup:  2188KhPgXq6d1gH58YZ27HA82Nz1bi4rrQHkGAV7knVRc85Uhvbff2YfWMvpJQ6zA6WTE6FRytauVwvy8QPnS63o
    ✔ Delegate to Ephemeral Rollup! (3431ms)

User Account State Updated:  4dYGpAWz6C4oSYdWfaFn5YieFUcaQTgDrFh2N3RdV2DkiH21kG9wuquHQzBgMGq2LATB4oiZHzzLamvbGJH7XtPF
    ✔ Update State and Commit to Base Layer! (6012ms)
delegated state before random update:  43

State Updated with Random value:  gXon2PwvsFGbXbtYjLNEixCGDqgwnGrMoHXmj1Xneqgmb18EsngWAZNxWj1FBGyPnhSBtUExaa9WrH4QzDjbSwK
delegated state after random update:  43
    1) Update Delegated State with Random value!
User Account Info:  {
  data: <Buffer d3 21 88 10 ba 6e f2 7f 8f 10 53 da f9 bc 02 f9 ff 2a 8d 99 ef 4f 11 4c b8 c9 32 0e 85 54 a2 82 7e cf 6e 2b b6 37 9e 5b 2b 00 00 00 00 00 00 00 fe>,
  executable: false,
  lamports: 1231920,
  owner: PublicKey [PublicKey(AZdi44i7DALnmupX5MZPoKCzvJ8s9CAUz15emtBm1aZ5)] {
    _bn: <BN: 8e15461cd8db1da72a8b606cd8f790987f24c518087d8139c308be8e891fa1a8>
  },
  rentEpoch: 18446744073709552000,
  space: 49
}
User account 7j7GRSm9R12hky3j8NXbqKkkKYG5RAQc57JSrDyV1CYr

User Account Undelegated:  5B9T7qrk8bbRrrvGgonTNXWgKQXAxReCUvyAoKmjUewToBeWEi6BA3LvHBqXhUZJuWRMNJ3ASEq8ix2TdsGQSPHT
    ✔ Commit and undelegate from Ephemeral Rollup! (6215ms)

User Account State Updated:  3M4C5ssDyar1sjgxPPW27Px4Qmw2gfXboPUgpeU8K8WQ1c2QSEWFF6L9aX7rFHgC3DekzbKfnzB87Y2PwQkcGuaN
    ✔ Update State! (3575ms)

User Account Closed:  4pnRpSiMyiSYcvtkGzowRR7HMnZfhDiERk9QjcjuxzsUwP7ZdEonU6QqqJGscNCqpYJbN53bHNHJA5zBy66NCb4r
    ✔ Close Account! (4260ms)
*/
