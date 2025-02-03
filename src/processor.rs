//! Program processor.

//! Program entrypoint.

use {
    crate::{
        instruction::EscrowInstruction,
        state::{get_escrow_pda, get_escrow_signer_seeds, Escrow},
    },
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        clock::Clock,
        entrypoint::ProgramResult,
        msg,
        program::{invoke, invoke_signed},
        program_error::ProgramError,
        pubkey::Pubkey,
        sysvar::Sysvar,
    },
    spl_token::instruction::transfer_checked,
};
use crate::state::EscrowPda;

fn process_escrow(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    expiration_slot: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let sender = next_account_info(accounts_iter)?;
    let sender_token_account = next_account_info(accounts_iter)?;
    let escrow = next_account_info(accounts_iter)?;
    let escrow_token_account = next_account_info(accounts_iter)?;
    let mint_account = next_account_info(accounts_iter)?;

    if !sender.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let EscrowPda {
        pubkey,
        bump: _,
    } = get_escrow_pda(program_id, mint_account.key, sender.key);

    if escrow.key != &pubkey {
        return Err(ProgramError::InvalidSeeds);
    }

    // Token transfer...

    let instruction = transfer_checked(
        &spl_token::id(),
        sender_token_account.key,
        mint_account.key,
        escrow_token_account.key,
        sender.key,
        &[sender.key],
        amount,
        /* decimals */ 0,
    )?;

    invoke(
        &instruction,
        &[
            sender_token_account.clone(),
            mint_account.clone(),
            escrow_token_account.clone(),
            sender.clone(),
        ],
    )?;

    escrow.try_borrow_mut_data()?[..].copy_from_slice(
        &Escrow {
            amount,
            creator: *sender.key,
            expiration_slot,
        }
        .pack(),
    );

    Ok(())
}

fn process_withdraw(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let sender = next_account_info(accounts_iter)?;
    let sender_token_account = next_account_info(accounts_iter)?;
    let escrow = next_account_info(accounts_iter)?;
    let escrow_token_account = next_account_info(accounts_iter)?;
    let mint_account = next_account_info(accounts_iter)?;

    if !sender.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let EscrowPda {
        pubkey,
        bump,
    } = get_escrow_pda(program_id, mint_account.key, sender.key);

    if escrow.key != &pubkey {
        return Err(ProgramError::InvalidSeeds);
    }

    let clock = Clock::get()?;

    let data = escrow.try_borrow_data()?;
    let escrow_state = Escrow::unpack(&data)?;

    if clock.slot < escrow_state.expiration_slot {
        return Err(ProgramError::InvalidArgument);
    }

    // Token transfers...

    let instruction = transfer_checked(
        &spl_token::id(),
        escrow_token_account.key,
        mint_account.key,
        sender_token_account.key,
        escrow.key,
        &[escrow.key],
        escrow_state.amount,
        /* decimals */ 0,
    )?;

    let bump_seed = &[bump];
    let signer_seeds = get_escrow_signer_seeds(mint_account.key, sender.key, bump_seed);

    invoke_signed(
        &instruction,
        &[
            escrow_token_account.clone(),
            mint_account.clone(),
            sender_token_account.clone(),
            escrow.clone(),
        ],
        &[&signer_seeds],
    )?;

    Ok(())
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
    match EscrowInstruction::unpack(input)? {
        EscrowInstruction::EscrowTokens {
            amount,
            expiration_slot,
        } => {
            msg!("Instruction: EscrowTokens");
            process_escrow(program_id, accounts, amount, expiration_slot)
        }
        EscrowInstruction::WithdrawTokens => {
            msg!("Instruction: WithdrawTokens");
            process_withdraw(program_id, accounts)
        }
    }
}
