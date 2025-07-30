import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import assert from "assert";
import { SpelltroumTournament } from "../target/types/spelltroum_tournament";

describe("spelltroum_tournament", () => {
  // Настраиваем провайдера и загружаем программу
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider() as anchor.AnchorProvider;
  const program = anchor.workspace
    .SpelltroumTournament as Program<SpelltroumTournament>;

  // Утилита для получения PDA игрока
  async function findPlayerPda(
    owner: PublicKey
  ): Promise<[PublicKey, number]> {
    return PublicKey.findProgramAddress(
      [Buffer.from("player"), owner.toBuffer()],
      program.programId
    );
  }

  it("deposit должен инициализировать и накапливать баланс", async () => {
    // Генерируем кошелёк игрока и даём ему SOL
    const player = Keypair.generate();
    const sig = await provider.connection.requestAirdrop(
      player.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(sig);
    // Немного ждём, чтобы аирдроп пришёл
    await new Promise((r) => setTimeout(r, 500));

    // Находим PDA
    const [pda] = await findPlayerPda(player.publicKey);

    // Первый депозит: 500
    await program.methods
      .deposit(new anchor.BN(500))
      .accounts({
        owner: player.publicKey,
        playerBalance: pda,
        systemProgram: SystemProgram.programId,
      })
      .signers([player])
      .rpc();

    // Проверяем баланс
    let acct = await program.account.playerBalance.fetch(pda);
    assert.ok(
      acct.amount.eq(new anchor.BN(500)),
      "После первого депозита должно быть 500"
    );

    // Второй депозит: +300 → всего 800
    await program.methods
      .deposit(new anchor.BN(300))
      .accounts({
        owner: player.publicKey,
        playerBalance: pda,
        systemProgram: SystemProgram.programId,
      })
      .signers([player])
      .rpc();

    acct = await program.account.playerBalance.fetch(pda);
    assert.ok(
      acct.amount.eq(new anchor.BN(800)),
      "После второго депозита должно быть 800"
    );
  });

  it("withdraw должен сбрасывать баланс в 0", async () => {
    // Новый игрок
    const player = Keypair.generate();
    const sig = await provider.connection.requestAirdrop(
      player.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(sig);
    await new Promise((r) => setTimeout(r, 500));

    const [pda] = await findPlayerPda(player.publicKey);

    // Сначала депозим 1000
    await program.methods
      .deposit(new anchor.BN(1000))
      .accounts({
        owner: player.publicKey,

        playerBalance: pda,
        systemProgram: SystemProgram.programId,
      })
      .signers([player])
      .rpc();

    // А теперь withdraw
    await program.methods
      .withdraw()
      .accounts({
        owner: player.publicKey,
        playerBalance: pda,
      })
      .signers([player])
      .rpc();

    const acct = await program.account.playerBalance.fetch(pda);
    assert.ok(acct.amount.eq(new anchor.BN(0)), "Баланс после withdraw = 0");
  });
});