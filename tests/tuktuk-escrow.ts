import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TuktukEscrow } from "../target/types/tuktuk_escrow";
import { PublicKey } from "@solana/web3.js";
import { init, taskKey, taskQueueAuthorityKey } from "@helium/tuktuk-sdk";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createMint,
  getAccount,
  getAssociatedTokenAddressSync,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { expect } from "chai";

describe("tuktuk-escrow", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const isDevnet = provider.connection.rpcEndpoint.includes("devnet");

  const program = anchor.workspace.tuktukEscrow as Program<TuktukEscrow>;
  const connection = provider.connection;
  const providerWallet = provider.wallet as anchor.Wallet;
  const payer = providerWallet.payer;

  let mintA: PublicKey;
  let mintB: PublicKey;
  let makerAtaA: PublicKey;
  let makerAtaB: PublicKey;
  let makerAtaStartBalance: bigint;

  const seed = new anchor.BN(Math.floor(Math.random() * 1000000));
  const seedBuf = Buffer.alloc(8);
  seedBuf.writeBigUInt64LE(BigInt(seed.toString()));
  let escrowPda: PublicKey;
  let vault: PublicKey;
  const deposit = new anchor.BN(1_000);
  const receive = new anchor.BN(10_000_000);

  const taskQueue = new anchor.web3.PublicKey("84ndxd9T3mrJnEKXCUT9SMTNRbP6ioEAP4qjax9dVjcf");
  const queueAuthority = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("queue_authority")], program.programId)[0];
  const taskQueueAuthority = taskQueueAuthorityKey(taskQueue, queueAuthority)[0];

  before(async () => {
    mintA = await createMint(connection, payer, payer.publicKey, null, 6, undefined, undefined, TOKEN_PROGRAM_ID);
    mintB = await createMint(connection, payer, payer.publicKey, null, 6, undefined, undefined, TOKEN_PROGRAM_ID);

    [escrowPda] = PublicKey.findProgramAddressSync([Buffer.from("escrow"), payer.publicKey.toBuffer(), seedBuf], program.programId);
    vault = getAssociatedTokenAddressSync(mintA, escrowPda, true, TOKEN_PROGRAM_ID);

    makerAtaA = getAssociatedTokenAddressSync(mintA, payer.publicKey, false, TOKEN_PROGRAM_ID);
    makerAtaB = getAssociatedTokenAddressSync(mintB, payer.publicKey, false, TOKEN_PROGRAM_ID);

    await getOrCreateAssociatedTokenAccount(connection, payer, mintA, payer.publicKey, false, undefined, undefined, TOKEN_PROGRAM_ID);
    await getOrCreateAssociatedTokenAccount(connection, payer, mintB, payer.publicKey, false, undefined, undefined, TOKEN_PROGRAM_ID);

    await mintTo(connection, payer, mintA, makerAtaA, payer.publicKey, 1_000_000_000, [], undefined, TOKEN_PROGRAM_ID);
    makerAtaStartBalance = (await getAccount(connection, makerAtaA, undefined, TOKEN_PROGRAM_ID)).amount;
  });

  it("Make locks maker funds in vault", async () => {
    await program.methods.make(seed, deposit, receive).accountsPartial({
      maker: payer.publicKey,
      mintA,
      mintB,
      makerAtaA,
      escrow: escrowPda,
      vault,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: anchor.web3.SystemProgram.programId,
    }).rpc();

    const escrowAccount = await program.account.escrowOffer.fetch(escrowPda);
    expect(escrowAccount.maker.toBase58()).to.eq(payer.publicKey.toBase58());
    expect(escrowAccount.mintA.toBase58()).to.eq(mintA.toBase58());
    expect(escrowAccount.mintB.toBase58()).to.eq(mintB.toBase58());
    expect(escrowAccount.receive.toString()).to.eq(receive.toString());

    const makerAtaAfterMake = await getAccount(connection, makerAtaA, undefined, TOKEN_PROGRAM_ID);
    const vaultAfterMake = await getAccount(connection, vault, undefined, TOKEN_PROGRAM_ID);

    expect(vaultAfterMake.amount.toString()).to.eq(deposit.toString());
    expect(makerAtaAfterMake.amount.toString()).to.eq((makerAtaStartBalance - BigInt(deposit.toString())).toString());
  });

  it("Refund returns funds and closes escrow", async () => {
    await program.methods.refund().accountsPartial({
      maker: payer.publicKey,
      mintA,
      makerAtaA,
      escrow: escrowPda,
      vault,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: anchor.web3.SystemProgram.programId,
    }).rpc();

    const escrowInfo = await connection.getAccountInfo(escrowPda);
    const vaultInfo = await connection.getAccountInfo(vault);
    const makerAtaAfterRefund = await getAccount(connection, makerAtaA, undefined, TOKEN_PROGRAM_ID);

    expect(escrowInfo).to.eq(null);
    expect(vaultInfo).to.eq(null);
    expect(makerAtaAfterRefund.amount.toString()).to.eq(makerAtaStartBalance.toString());
  });

  it("Schedule (devnet integration)", async () => {
    if (!isDevnet) {
      console.log("Skipping devnet-only schedule integration on non-devnet cluster");
      return;
    }

    const tuktukProgram = await init(provider);
    await tuktukProgram.methods
      .addQueueAuthorityV0()
      .accounts({
        payer: payer.publicKey,
        taskQueue,
        queueAuthority,
      })
      .rpc();

    const taskId = 10;
    const [taskPda] = taskKey(taskQueue, taskId);

    await program.methods.schedule(taskId).accountsPartial({
      maker: payer.publicKey,
      mintA,
      makerAtaA,
      escrow: escrowPda,
      vault,
      task: taskPda,
      taskQueue,
      queueAuthority,
      systemProgram: anchor.web3.SystemProgram.programId,
      taskQueueAuthority,
      tuktukProgram: tuktukProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
    }).rpc();
  });
});