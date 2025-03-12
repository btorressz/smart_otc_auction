//test file that was orgianlly exported from solana playground
//sill editing it
import * as anchor from "@coral-xyz/anchor";
import BN from "bn.js";
import assert from "assert";
import * as web3 from "@solana/web3.js";
import type { SmartOtcAuction } from "../target/types/smart_otc_auction";

describe("smart-otc-auction", () => {
  // Configure the client to use the local cluster
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.SmartOtcAuction as anchor.Program<SmartOtcAuction>;
  
  it("initializes the state", async () => {
    // Create a new keypair for the state account.
    const stateAccount = new web3.Keypair();

    // Create dummy parameters for the initialize function.
    // For testing,  use a new keypair's public key as the reward mint.
    const rewardMint = new web3.Keypair().publicKey;
    // Use the wallet's public key as the governance account.
    const governance = program.provider.publicKey;
    // Define auction parameters.
    const minBidIncrement = new BN(10);
    const slippageTolerance = new BN(100); // 1% tolerance (100 basis points)
    const highValueThreshold = new BN(1000);
    const minStake = new BN(50);
    const rewardVestingPeriod = new BN(3600); // 1 hour vesting period

    // Call the initialize method.
    const txHash = await program.methods.initialize(
      rewardMint,
      governance,
      minBidIncrement,
      slippageTolerance,
      highValueThreshold,
      minStake,
      rewardVestingPeriod
    )
      .accounts({
        admin: program.provider.publicKey,
        state: stateAccount.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([stateAccount])
      .rpc();

    console.log(`Initialized state. Transaction: ${txHash}`);

    // Confirm the transaction.
    await program.provider.connection.confirmTransaction(txHash);

    // Fetch the on-chain state account.
    const state = await program.account.state.fetch(stateAccount.publicKey);
    console.log("On-chain state:", state);

    // Assert that the state was initialized with the expected values.
    assert.ok(state.auctionCount.eq(new BN(0)));
    assert.ok(state.rewardMint.equals(rewardMint));
    assert.ok(state.admin.equals(program.provider.publicKey));
    assert.ok(state.minBidIncrement.eq(minBidIncrement));
    assert.ok(state.slippageTolerance.eq(slippageTolerance));
    assert.ok(state.highValueThreshold.eq(highValueThreshold));
    assert.ok(state.minStake.eq(minStake));
    assert.ok(state.rewardVestingPeriod.eq(rewardVestingPeriod));
  });
});
