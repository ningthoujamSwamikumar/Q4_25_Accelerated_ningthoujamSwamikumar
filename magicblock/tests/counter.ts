import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Counter } from "../target/types/counter";

const SEED_TEST_PDA = Buffer.from("counter");

// Set ER Validator
const ER_VALIDATOR = new anchor.web3.PublicKey(
    "MAS1Dt9qreoRMQ14YQuhg8UTZMMzDdKhmkZMECCzk57"  // Asia ER Validator
    //"mAGicPQYBMvcYveUZA5F5UNNwyHvfYh5xkLS2Fr1mev" // local ER validator
);

describe("counter", () => {
    let provider;
    let providerEphemeralRollup;
    let program;
    let pda;

    before(async () => {
        const magicClusterEndpoint = process.env.EPHEMERAL_PROVIDER_ENDPOINT || "https://devnet-as.magicblock.app/";
        const magicClusterWsEndpoint = process.env.EPHEMERAL_WS_ENDPOINT || "wss://devnet.magicblock.app/";

        console.log("magicClusterEndpoint: ", magicClusterEndpoint);
        console.log("magicClusterWsEndpoint: ", magicClusterWsEndpoint);

        // Set Anchor providers
        provider = anchor.AnchorProvider.env();
        anchor.setProvider(provider);
        providerEphemeralRollup = new anchor.AnchorProvider(
            new anchor.web3.Connection(magicClusterEndpoint, {
                wsEndpoint: magicClusterWsEndpoint,
            }),
            anchor.Wallet.local()
        );

        // Set program and pda
        program = anchor.workspace.Counter as Program<Counter>;
        pda = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from(SEED_TEST_PDA)],
            program.programId
        )[0];
    })

    it("init counter", async () => {
        // Initialize Counter on Base Layer
        let initTx = await program.methods
            .initialize()
            .accounts({
                user: provider.wallet.publicKey
            })
            .transaction();
        initTx.feePayer = provider.wallet.publicKey;
        initTx.recentBlockhash = (
            await provider.connection.getLatestBlockhash()
        ).blockhash;
        initTx = await provider.wallet.signTransaction(initTx);
        const dinitTxHash = await provider.sendAndConfirm(initTx);
        console.log("✅ counter initialize in base layer with sig: ", dinitTxHash);
    })

    it("increment counter in base layer", async () => {
        // increment counter on base layer
        let incrTx = await program.methods.increment()
            .accounts({
                user: provider.wallet.publicKey,
            }).transaction();
        incrTx.feePayer = provider.wallet.publicKey;
        incrTx.recentBlockhash = (await provider.connection.getLatestBlockhash()).blockhash;
        incrTx = await provider.wallet.signTransaction(incrTx);
        let incrTxHash = await provider.sendAndConfirm(incrTx);
        console.log("✅ counter incremented in base layer with sig: ", incrTxHash);
    })

    it("delegate counter account to ER", async () => {
        // Delegate Counter on Base Layer to ER
        let delTx = await program.methods
            .delegate()
            .accounts({
                payer: provider.wallet.publicKey,
                validator: ER_VALIDATOR,
                pda: pda,
            })
            .transaction();
        delTx.feePayer = provider.wallet.publicKey;
        let lateshBlockhash = await provider.connection.getLatestBlockhash()
        delTx.recentBlockhash = lateshBlockhash.blockhash;
        delTx = await provider.wallet.signTransaction(delTx);
        const delTxHash = await provider.sendAndConfirm(delTx, undefined, { commitment: 'finalized', skipPreflight: true });
        await provider.connection.confirmTransaction({
            blockhash: lateshBlockhash.blockhash,
            lastValidBlockHeight: lateshBlockhash.lastValidBlockHeight,
            signature: delTxHash
        });
        console.log("✅ counter pda delegated with sig: ", delTxHash);
    })

    it("increment counter in real time on ER", async () => {
        // Increment Counter in real-time on ER
        let incTx = await program.methods
            .increment()
            .accounts({
                user: providerEphemeralRollup.wallet.publicKey
            })
            .transaction();
        incTx.feePayer = providerEphemeralRollup.wallet.publicKey;
        incTx.recentBlockhash = (
            await providerEphemeralRollup.connection.getLatestBlockhash()
        ).blockhash;
        incTx = await providerEphemeralRollup.wallet.signTransaction(incTx);
        const incTxHash = await providerEphemeralRollup.sendAndConfirm(incTx);
        console.log("✅ incremented counter in real time on ER with sig: ", incTxHash);
    })

    it("decrement counter in real time on ER", async () => {
        // Decrement Counter in real-time on ER
        let decTx = await program.methods
            .decrement()
            .accounts({
                user: providerEphemeralRollup.wallet.publicKey
            })
            .transaction();
        decTx.feePayer = providerEphemeralRollup.wallet.publicKey;
        decTx.recentBlockhash = (
            await providerEphemeralRollup.connection.getLatestBlockhash()
        ).blockhash;
        decTx = await providerEphemeralRollup.wallet.signTransaction(decTx);
        const decTxHash = await providerEphemeralRollup.sendAndConfirm(decTx);
        console.log("✅ decremented counter in real time on ER with sig: ", decTxHash);
    })

    it("undelegate counter account from ER", async () => {
        // Undelegate Counter on Base Layer to ER
        let undelTx = await program.methods
            .undelegate()
            .accounts({
                user: providerEphemeralRollup.wallet.publicKey
            })
            .transaction();
        undelTx.feePayer = provider.wallet.publicKey;
        //lateshBlockhash = await providerEphemeralRollup.connection.getLatestBlockhash();
        undelTx.recentBlockhash = (await providerEphemeralRollup.connection.getLatestBlockhash()).blockhash;
        undelTx = await providerEphemeralRollup.wallet.signTransaction(undelTx);
        const undelTxHash = await providerEphemeralRollup.sendAndConfirm(undelTx, undefined, { commitment: "finalized", skipPreflight: true });
        await provider.connection.confirmTransaction({
            blockhash: (await providerEphemeralRollup.connection.getLatestBlockhash()).blockhash,
            lastValidBlockHeight: (await providerEphemeralRollup.connection.getLatestBlockhash()).lastValidBlockHeight,
            signature: undelTxHash
        });
        console.log("✅ counter pda undelegated with sig: ", undelTxHash);
    })

    it("increment counter in base layer", async () => {
        // increment counter on base layer
        let incrTx = await program.methods.increment()
            .accounts({
                user: provider.wallet.publicKey,
            }).transaction();
        incrTx.feePayer = provider.wallet.publicKey;
        incrTx.recentBlockhash = (await provider.connection.getLatestBlockhash()).blockhash;
        incrTx = await provider.wallet.signTransaction(incrTx);
        const incrTxHash = await provider.sendAndConfirm(incrTx);
        console.log("✅ counter incremented in base layer with sig: ", incrTxHash);
    })

    it("close pda account in base layer", async () => {
        // Close the counter pda, so to not have to restart the validators
        let closeTx = await program.methods.closePda().accounts({ user: provider.wallet.publicKey }).transaction();
        closeTx.feePayer = provider.wallet.publicKey;
        closeTx.recentBlockhash = (
            await provider.connection.getLatestBlockhash()
        ).blockhash;
        closeTx = await provider.wallet.signTransaction(closeTx);
        const closeTxHash = await provider.sendAndConfirm(closeTx);
        console.log("✅ counter pda closed with sig: ", closeTxHash);
    })
})
