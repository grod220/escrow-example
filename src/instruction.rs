//! Program instructions.

use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
};

/// Program instructions.
pub enum EscrowInstruction {
    /// Lock up some tokens with an escrow account. Allows user to configure a
    /// lockup period.
    ///
    /// Account expected by this instruction:
    ///
    /// 0. `[s]` Sender
    /// 1. `[w]` Sender token account
    /// 2. `[w]` Escrow account
    /// 3. `[w]` Escrow token account
    /// 4. `[ ]` Mint account
    /// 5. `[ ]` Token program
    EscrowTokens { amount: u64, expiration_slot: u64 },
    /// Withdraw tokens from escrow, provided the lockup period has ended.
    ///
    /// Account expected by this instruction:
    ///
    /// 0. `[s]` Sender
    /// 1. `[w]` Sender token account
    /// 2. `[w]` Escrow account
    /// 3. `[w]` Escrow token account
    /// 4. `[ ]` Mint account
    /// 5. `[ ]` Token program
    WithdrawTokens,
}

impl EscrowInstruction {
    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        match self {
            EscrowInstruction::EscrowTokens {
                amount,
                expiration_slot,
            } => {
                buf.push(0);
                buf.extend_from_slice(&amount.to_le_bytes());
                buf.extend_from_slice(&expiration_slot.to_le_bytes());
            }
            EscrowInstruction::WithdrawTokens => {
                buf.push(1);
            }
        }
        buf
    }

    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        match input.split_first() {
            Some((&0, rest)) if rest.len() == 16 => {
                let amount = u64::from_le_bytes(rest[0..8].try_into().unwrap());
                let expiration_slot = u64::from_le_bytes(rest[8..16].try_into().unwrap());
                Ok(EscrowInstruction::EscrowTokens {
                    amount,
                    expiration_slot,
                })
            }
            Some((&1, _)) => Ok(EscrowInstruction::WithdrawTokens),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}

pub fn escrow_tokens(
    program_id: &Pubkey,
    sender_address: &Pubkey,
    sender_token_account_address: &Pubkey,
    escrow_address: &Pubkey,
    escrow_token_account_address: &Pubkey,
    mint_address: &Pubkey,
    token_program_id: &Pubkey,
    amount: u64,
    expiration_slot: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*sender_address, true),
        AccountMeta::new(*sender_token_account_address, false),
        AccountMeta::new(*escrow_address, false),
        AccountMeta::new(*escrow_token_account_address, false),
        AccountMeta::new_readonly(*mint_address, false),
        AccountMeta::new_readonly(*token_program_id, false),
    ];
    let data = EscrowInstruction::EscrowTokens {
        amount,
        expiration_slot,
    }
    .pack();
    Instruction::new_with_bytes(*program_id, &data, accounts)
}

pub fn withdraw_tokens(
    program_id: &Pubkey,
    sender_address: &Pubkey,
    sender_token_account_address: &Pubkey,
    escrow_address: &Pubkey,
    escrow_token_account_address: &Pubkey,
    mint_address: &Pubkey,
    token_program_id: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*sender_address, true),
        AccountMeta::new(*sender_token_account_address, false),
        AccountMeta::new(*escrow_address, false),
        AccountMeta::new(*escrow_token_account_address, false),
        AccountMeta::new_readonly(*mint_address, false),
        AccountMeta::new_readonly(*token_program_id, false),
    ];
    let data = EscrowInstruction::WithdrawTokens.pack();
    Instruction::new_with_bytes(*program_id, &data, accounts)
}
