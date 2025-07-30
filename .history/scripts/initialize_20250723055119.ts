import * as anchor from "@coral-xyz/anchor";

const main = async () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    const program = anchor.workspace.SpelltroumTornament as anchor.Program;

    const tx = await program.methods.initialize().rpc();
    console.log("✅ Tx Signature:", tx);
};

main().catch((err) => {
    console.error(err);
});