// tests/spelltroum_tournament.ts

import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import assert from "assert";
import { SpelltroumTournament } from "../target/types/spelltroum_tournament";

anchor.setProvider(anchor.AnchorProvider.env());
const provider = anchor.getProvider() as anchor.AnchorProvider;
const program = anchor.workspace
  .SpelltroumTournament as Program<SpelltroumTournament>;

/** 
 * Интерфейс для аргумента .accounts() в deposit 
 */
interface DepositAccounts {
  owner: PublicKey;
  playerBalance: PublicKey;
  systemProgram: PublicKey;
}

/** 
 * Интерфейс для аргумента .accounts() в withdraw 
 */
interface WithdrawAccounts {
  owner: PublicKey;
  playerBalance: PublicKey;
}

/** 
 * Находит PDA для аккаунта PlayerBalance по ключу игрока 
 */
async function findPlayerPda(owner: PublicKey): Promise<[PublicKey, number]> {
  return PublicKey.findProgramAddress(
    [Buffer.from("player"), owner.toBuffer()],
    program.programId
  );
}

describe("spelltroum_tournament", () => {
  it("deposit должен инициализировать и накапливать баланс", async () => {
    // 1) Создаём нового игрока
    const player = Keypair.generate();

    // 2) Аирдропим немного SOL, чтобы можно было платить за создание PDA
    const sig = await provider.connection.requestAirdrop(
      player.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(sig);
    await new Promise((r) => setTimeout(r, 500)); // ждём подтверждения

    // 3) Получаем PDA-адрес
    const [pda] = await findPlayerPda(player.publicKey);

    // 4) Первый депозит: 500
    await program.methods
      .deposit(new anchor.BN(500))
      .accounts<DepositAccounts>({
        owner: player.publicKey,
        playerBalance: pda,
        systemProgram: SystemProgram.programId,
      })
      .signers([player])
      .rpc();

    // 5) Проверяем: balance == 500
    let acct = await program.account.playerBalance.fetch(pda);
    assert.ok(
      acct.amount.eq(new anchor.BN(500)),
      `После первого депозита должно быть 500, а получилось ${acct.amount}`
    );

    // 6) Второй депозит: +300 → всего должно стать 800
    await program.methods
      .deposit(new anchor.BN(300))
      .accounts<DepositAccounts>({
        owner: player.publicKey,
        playerBalance: pda,
        systemProgram: SystemProgram.programId,
      })
      .signers([player])
      .rpc();

    // 7) Проверяем: balance == 800
    acct = await program.account.playerBalance.fetch(pda);
    assert.ok(
      acct.amount.eq(new anchor.BN(800)),
      `После второго депозита должно быть 800, а получилось ${acct.amount}`
    );
  });

  it("withdraw должен сбрасывать баланс в 0", async () => {
    // 1) Новый игрок и аирдроп
    const player = Keypair.generate();
    const sig = await provider.connection.requestAirdrop(
      player.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(sig);
    await new Promise((r) => setTimeout(r, 500));

    // 2) Находим PDA и сразу делаем депозит 1000
    const [pda] = await findPlayerPda(player.publicKey);
    await program.methods
      .deposit(new anchor.BN(1000))
      .accounts<DepositAccounts>({
        owner: player.publicKey,
        playerBalance: pda,
        systemProgram: SystemProgram.programId,
      })
      .signers([player])
      .rpc();

    // 3) Вызываем withdraw
    await program.methods
      .withdraw()
      .accounts<WithdrawAccounts>({
        owner: player.publicKey,
        playerBalance: pda,
      })
      .signers([player])
      .rpc();

    // 4) Проверяем: balance == 0
    const acct = await program.account.playerBalance.fetch(pda);
    assert.ok(
      acct.amount.eq(new anchor.BN(0)),
      `После withdraw баланс должен быть 0, а получилось ${acct.amount}`
    );
  });
});