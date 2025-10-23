import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Week1Challenge } from "../target/types/week1_challenge";
import { TransferHook } from "../target/types/transfer_hook";

describe("week1-challenge", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.week1Challenge as Program<Week1Challenge>;
  const transfer_hook_program = anchor.workspace.transferHook as Program<TransferHook>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
