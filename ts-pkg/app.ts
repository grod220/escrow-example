import { get_escrow_pda, Pubkey } from "./wasm";

const programId = new Pubkey("TwRapQCDhWkZRrDaHfZGuHxkZ91gHDRkyuzNqeU5MgR")
const mint = new Pubkey("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb")
const creator = new Pubkey("11111116djSnXB2wXVGT4xDLsfTnkp1p4cCxHAfRq")

const { pubkey, bump } = get_escrow_pda(programId, mint, creator)

console.log(`EscrowPda: ${pubkey.toString()}`);
console.log(`Bump: ${bump}`);

// EscrowPda: 4tU5BP1aCE5nYPzmtbfNWZrqZMmgSNWSqt2NF9Y4Yk9q
// Bump: 252
