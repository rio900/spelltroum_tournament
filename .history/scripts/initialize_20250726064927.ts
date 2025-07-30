import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { SpelltroumTournament } from "../target/types/spelltroum_tournament";
// Замените путь на ваш IDL-тайп

async function main() {
    // 1) инициализация провайдера и программы
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    const program = anchor.workspace.SpelltroumTournament as Program<SpelltroumTournament>;
    const payer = provider.wallet.publicKey;

    // === Вспомогательная функция для получения PDA баланса игрока ===
    function getPlayerPDA(playerPubkey: PublicKey): PublicKey {
        const [pda] = PublicKey.findProgramAddressSync(
            [Buffer.from("player"), playerPubkey.toBuffer()],
            program.programId
        );
        return pda;
    }

    // -----------------------------
    // 2) Вызов deposit(amount)
    // -----------------------------
    const depositAmount = 500; // любое число > 0
    const playerPda = getPlayerPDA(payer);
    const txDeposit = await program.methods
        .deposit(new BN(depositAmount))
        .accounts({
            owner: payer,
            playerBalance: playerPda,
            systemProgram: SystemProgram.programId,
        })
        .rpc();
    console.log("✅ deposit:", txDeposit);

    // -----------------------------
    // 3) Вызов withdraw()
    // -----------------------------
    const txWithdraw = await program.methods
        .withdraw()
        .accounts({
            owner: payer,
            playerBalance: playerPda,
        })
        .rpc();
    console.log("✅ withdraw:", txWithdraw);

    // -----------------------------
    // 4) Вызов settle_match(matchId, players, winners, entryFee)
    // -----------------------------
    // 4.1) Параметры матча
    const matchId = "match_123";
    const players: PublicKey[] = [
        // замените на реальные pubkey игроков
        new PublicKey("Player111111111111111111111111111111111"),
        new PublicKey("Player222222222222222222222222222222222"),
    ];
    const winners: PublicKey[] = [
        // subset из players
        players[0],
    ];
    const entryFee = 100; // должна соответствовать BN

    // 4.2) Печатная история матча (предполагается, что аккаунт уже создан!)
    const matchHistoryPubkey = new PublicKey("HistoryAccount1111111111111111111111111111");

    // 4.3) Собираем remainingAccounts: first все игроки (списываем entryFee), потом победители (получают share)
    const remainingAccounts = [
        ...players.map((pk) => ({
            pubkey: getPlayerPDA(pk),
            isWritable: true as const,
            isSigner: false as const,
        })),
        ...winners.map((pk) => ({
            pubkey: getPlayerPDA(pk),
            isWritable: true as const,
            isSigner: false as const,
        })),
    ];

    const txSettle = await program.methods
        .settleMatch(matchId, players, winners, new BN(entryFee))
        .accounts({
            matchHistory: matchHistoryPubkey,
            authority: payer,
        })
        .remainingAccounts(remainingAccounts)
        .rpc();
    console.log("✅ settleMatch:", txSettle);
}

main()
    .then(() => process.exit(0))
    .catch((err) => {
        console.error(err);
        process.exit(1);
    });