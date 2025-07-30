import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SpelltroumTournament } from "../target/types/spelltroum_tournament";
import { PublicKey, SystemProgram, Keypair, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import assert from "assert";

describe("spelltroum_tournament", () => {
  // Настройка провайдера и программы
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.SpelltroumTournament as Program<SpelltroumTournament>;

  const owner = provider.wallet.publicKey;
  let playerBalancePda: PublicKey;
  let playerBalanceBump: number;

  // Находим PDA для аккаунта баланса игрока
  before(async () => {
    [playerBalancePda, playerBalanceBump] = await PublicKey.findProgramAddress(
      [Buffer.from("player"), owner.toBuffer()],
      program.programId
    );
  });

  it("Initial deposit should create player balance account and set amount", async () => {
    const depositAmount = new anchor.BN(1000);

    // Вызываем метод deposit
    await program.methods
      .deposit(depositAmount)
      .accounts({
        owner,
        playerBalance: playerBalancePda,
        systemProgram: SystemProgram.programId,
      } as any)
      .rpc();

    // Проверяем, что аккаунт создался и amount = depositAmount
    const account = await program.account.playerBalance.fetch(playerBalancePda);
    assert.ok(account.amount.eq(depositAmount), "Amount should match deposit");
    assert.ok(account.owner.equals(owner), "Owner should be set correctly");
  });

  it("Deposit zero should fail with InvalidAmount", async () => {
    // Попытка внести 0 должна упасть с ошибкой CustomError::InvalidAmount
    await assert.rejects(
      () =>
        program.methods
          .deposit(new anchor.BN(0))
          .accounts({
            owner,
            playerBalance: playerBalancePda,
            systemProgram: SystemProgram.programId,
          } as any)
          .rpc(),
      /InvalidAmount/
    );
  });

  it("Withdraw should set amount to zero", async () => {
    // Сначала чуть-чуть депозита, чтобы было что снимать
    await program.methods
      .deposit(new anchor.BN(500))
      .accounts({
        owner,
        playerBalance: playerBalancePda,
        systemProgram: SystemProgram.programId,
      } as any)
      .rpc();

    // Вызываем withdraw
    await program.methods
      .withdraw()
      .accounts({
        owner,
        playerBalance: playerBalancePda,
      })
      .rpc();

    const account = await program.account.playerBalance.fetch(playerBalancePda);
    assert.ok(account.amount.eq(new anchor.BN(0)), "Amount should be reset to zero");
  });

  it("Withdraw with zero balance should fail with NothingToWithdraw", async () => {
    // Снова пытаемся снять, когда balance = 0
    await assert.rejects(
      () =>
        program.methods
          .withdraw()
          .accounts({
            owner,
            playerBalance: playerBalancePda,
          })
          .rpc(),
      /NothingToWithdraw/
    );
  });

  // TODO: написать тесты для settle_match после добавления в контракт инициализации MatchHistory
});