import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { LendingPool } from "../target/types/lending_pool";

describe("Initialize Pool", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.Lending as Program<LendingPool>;



const CHAINLINK_PROGRAM = "HEvSKofvBgfaexv23kMabbYqxasxU3mQ4ibBMEmJWHny";
const CHAINLINK_SOL_USD = "99B2bTijsU6f1GCT73HmdR7HCFFjGMBcPZY6jZ96ynrR";
const CHAINLINK_USDC_USD = "2EmfL3MqL3YHABudGNmajjCpR13NNEn9Y4LWxbDm6SwR";

  it("Initialize Pool", async () => {
    // 代币 Mint 地址
    const mint = new anchor.web3.PublicKey("..."); // 替换为实际的 Mint 地址

    // 计算资金池 PDA
    const [poolPda, poolBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("lending_pool"), mint.toBuffer()],
      program.programId
    );

    // 调用初始化指令
    await program.methods.initializePool(
      mint,          // 代币 Mint 地址
      6,             // 代币精度（如 USDC 是 6）
      10,            // 储备金率（10%）
      75,            // 抵押率（75%）
      500            // 基础利率（5% APR）
    )
      .accounts({
        pool: poolPda,
        mint: mint,
        authority: provider.wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    console.log("Pool Initialized:", poolPda.toBase58());
  });
});