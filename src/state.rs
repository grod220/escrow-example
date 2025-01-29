//! Program state types (accounts).

use solana_program::{program_error::ProgramError, pubkey::Pubkey};

pub fn get_escrow_pda(program_id: &Pubkey, mint: &Pubkey, creator: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"escrow", mint.as_ref(), creator.as_ref()], program_id)
}

pub fn get_escrow_signer_seeds<'a>(
    mint: &'a Pubkey,
    creator: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 4] {
    [b"escrow", mint.as_ref(), creator.as_ref(), bump_seed]
}

/// Escrow account.
pub struct Escrow {
    /// Amount of tokens escrowed.
    pub amount: u64,
    /// Expiration slot for lockup.
    pub expiration_slot: u64,
    /// Creator pubkey.
    pub creator: Pubkey,
}

impl Escrow {
    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(8 + 8 + 32);
        buf.extend_from_slice(&self.amount.to_le_bytes());
        buf.extend_from_slice(&self.expiration_slot.to_le_bytes());
        buf.extend_from_slice(self.creator.as_ref());
        buf
    }

    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        if input.len() != 8 + 8 + 32 {
            return Err(ProgramError::InvalidAccountData);
        }
        let amount = u64::from_le_bytes(input[0..8].try_into().unwrap());
        let expiration_slot = u64::from_le_bytes(input[8..16].try_into().unwrap());
        let creator = Pubkey::new_from_array(input[16..48].try_into().unwrap());
        Ok(Escrow {
            amount,
            expiration_slot,
            creator,
        })
    }
}
