import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram, Keypair } from "@solana/web3.js";
// Import the generated IDL type for your program
import { SpelltroumTournament } from "../target/types/spelltroum_tournament";

describe("spelltroum_tournament", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.SpelltroumTournament as Program<SpelltroumTournament>;

  const connection = anchor.getProvider().connection;

  const createPlayer = async (amount: number) => {
    const player = Keypair.generate();
    const lamports = await connection.getMinimumBalanceForRentExemption(8);
    const [pda, bump] = await PublicKey.findProgramAddress(
      [Buffer.from("player"), player.publicKey.toBuffer()],
      program.programId
    );

    await connection.requestAirdrop(player.publicKey, 2e9);
    await new Promise(resolve => setTimeout(resolve, 500));

    // Create PDA account manually (simulate initialized PDA)
    await program.provider.sendAndConfirm(new anchor.web3.Transaction().add(
      SystemProgram.createAccount({
        fromPubkey: player.publicKey,
        newAccountPubkey: pda,
        lamports,
        space: 100, // enough space for PlayerBalance
        programId: program.programId,
      })
    ), [player]);

    return { player, pda, bump };
  };

  it("Deposits tokens", async () => {
    const { player, pda, bump } = await createPlayer(1000);

    await program.methods.deposit(new anchor.BN(1000))
      .accounts({
        owner: player.publicKey,
        playerBalance: pda,
      })
      .signers([player])
      .rpc();

    const balance = await program.account.playerBalance.fetch(pda);
    console.log("Balance after deposit:", balance.amount.toString());
  });

  it("Settles a match", async () => {
    const matchAuthority = anchor.web3.Keypair.generate();
    const matchHistory = anchor.web3.Keypair.generate();

    // Create 2 players: Alice (winner), Bob (loser)
    const alice = await createPlayer(100);
    const bob = await createPlayer(100);

    const matchId = "match-001";

    // Initialize match history
    await program.provider.sendAndConfirm(new anchor.web3.Transaction().add(
      SystemProgram.createAccount({
        fromPubkey: matchAuthority.publicKey,
        newAccountPubkey: matchHistory.publicKey,
        lamports: await connection.getMinimumBalanceForRentExemption(1000),
        space: 1000,
        programId: program.programId,
      })
    ), [matchAuthority, matchHistory]);

    await program.methods.settleMatch(matchId, [bob.player.publicKey, alice.player.publicKey], [alice.player.publicKey], new anchor.BN(100))
      .accounts({
        matchHistory: matchHistory.publicKey,
        authority: matchAuthority.publicKey,
      })
      .remainingAccounts([
        { pubkey: bob.pda, isWritable: true, isSigner: false },
        { pubkey: alice.pda, isWritable: true, isSigner: false },
        { pubkey: alice.pda, isWritable: true, isSigner: false },
      ])
      .signers([matchAuthority])
      .rpc();

    const aliceBalance = await program.account.playerBalance.fetch(alice.pda);
    console.log("Alice's balance after winning:", aliceBalance.amount.toString());
  });

  it("Withdraws tokens", async () => {
    const { player, pda, bump } = await createPlayer(500);

    await program.methods.deposit(new anchor.BN(500))
      .accounts({
        owner: player.publicKey,
        playerBalance: pda,
      })
      .signers([player])
      .rpc();

    await program.methods.withdraw()
      .accounts({
        owner: player.publicKey,
        playerBalance: pda,
      })
      .signers([player])
      .rpc();

    const balance = await program.account.playerBalance.fetch(pda);
    console.log("Balance after withdrawal:", balance.amount.toString()); // should be 0
  });
});