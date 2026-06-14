use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    instruction::{AccountMeta, Instruction},
    program::{invoke, invoke_signed},
    pubkey::Pubkey,
};

use crate::{
    constants::{spl_token_program_id, JOB_SEED},
    errors::IcpxError,
    state::JobState,
};

const TOKEN_ACCOUNT_LEN: usize = 165;
const TOKEN_ACCOUNT_MINT_OFFSET: usize = 0;
const TOKEN_ACCOUNT_OWNER_OFFSET: usize = 32;
const TOKEN_ACCOUNT_AMOUNT_OFFSET: usize = 64;
const TOKEN_ACCOUNT_STATE_OFFSET: usize = 108;
const TOKEN_ACCOUNT_INITIALIZED: u8 = 1;
const SPL_TOKEN_TRANSFER_INSTRUCTION: u8 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TokenAccountSnapshot {
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub amount: u64,
}

pub fn load_job(program_id: &Pubkey, job_account: &AccountInfo) -> Result<JobState, ProgramError> {
    if job_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let job = JobState::try_from_slice(&job_account.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;
    let nonce_bytes = job.client_nonce.to_le_bytes();
    let seeds = [
        JOB_SEED,
        job.requester.as_ref(),
        job.provider.as_ref(),
        nonce_bytes.as_ref(),
    ];
    let (expected_job, bump) = Pubkey::find_program_address(&seeds, program_id);
    if expected_job != *job_account.key || bump != job.bump {
        return Err(IcpxError::InvalidPda.into());
    }

    Ok(job)
}

pub fn save_job(job_account: &AccountInfo, job: &JobState) -> ProgramResult {
    job.serialize(&mut &mut job_account.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)
}

pub fn require_signer(account: &AccountInfo) -> ProgramResult {
    if !account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    Ok(())
}

pub fn require_key(actual: &Pubkey, expected: &Pubkey, error: IcpxError) -> ProgramResult {
    if actual != expected {
        return Err(error.into());
    }
    Ok(())
}

pub fn require_nonzero_pubkey(pubkey: &Pubkey) -> ProgramResult {
    if *pubkey == Pubkey::default() {
        return Err(IcpxError::InvalidTerms.into());
    }
    Ok(())
}

pub fn require_nonzero_hash(hash: &[u8; 32]) -> ProgramResult {
    if *hash == [0; 32] {
        return Err(IcpxError::InvalidGpuTerms.into());
    }
    Ok(())
}

pub fn require_system_program(system_program: &AccountInfo) -> ProgramResult {
    require_key(
        system_program.key,
        &solana_program::system_program::id(),
        IcpxError::InvalidSystemProgram,
    )
}

pub fn require_spl_token_program(token_program: &AccountInfo) -> ProgramResult {
    require_key(
        token_program.key,
        &spl_token_program_id(),
        IcpxError::InvalidTokenProgram,
    )
}

pub fn load_token_account(
    token_account: &AccountInfo,
) -> Result<TokenAccountSnapshot, ProgramError> {
    if *token_account.owner != spl_token_program_id() {
        return Err(IcpxError::InvalidTokenAccount.into());
    }

    let data = token_account.data.borrow();
    if data.len() < TOKEN_ACCOUNT_LEN {
        return Err(IcpxError::InvalidTokenAccount.into());
    }

    if data[TOKEN_ACCOUNT_STATE_OFFSET] != TOKEN_ACCOUNT_INITIALIZED {
        return Err(IcpxError::InvalidTokenAccount.into());
    }

    Ok(TokenAccountSnapshot {
        mint: read_pubkey(&data, TOKEN_ACCOUNT_MINT_OFFSET)?,
        owner: read_pubkey(&data, TOKEN_ACCOUNT_OWNER_OFFSET)?,
        amount: read_u64(&data, TOKEN_ACCOUNT_AMOUNT_OFFSET)?,
    })
}

pub fn require_token_account(
    token_account: &AccountInfo,
    expected_mint: &Pubkey,
    expected_owner: &Pubkey,
) -> Result<TokenAccountSnapshot, ProgramError> {
    let snapshot = load_token_account(token_account)?;
    if snapshot.mint != *expected_mint {
        return Err(IcpxError::InvalidTokenMint.into());
    }
    if snapshot.owner != *expected_owner {
        return Err(IcpxError::InvalidTokenOwner.into());
    }
    Ok(snapshot)
}

pub fn transfer_lamports(from: &AccountInfo, to: &AccountInfo, amount: u64) -> ProgramResult {
    if amount == 0 {
        return Ok(());
    }

    let from_balance = **from.lamports.borrow();
    if from_balance < amount {
        return Err(IcpxError::EscrowUnderfunded.into());
    }

    **from.try_borrow_mut_lamports()? = from_balance
        .checked_sub(amount)
        .ok_or(IcpxError::MathOverflow)?;
    **to.try_borrow_mut_lamports()? = to
        .lamports()
        .checked_add(amount)
        .ok_or(IcpxError::MathOverflow)?;
    Ok(())
}

pub fn transfer_spl_tokens(
    token_program: &AccountInfo,
    source: &AccountInfo,
    destination: &AccountInfo,
    authority: &AccountInfo,
    signer_seeds: Option<&[&[u8]]>,
    amount: u64,
) -> ProgramResult {
    if amount == 0 {
        return Ok(());
    }

    require_spl_token_program(token_program)?;

    let mut data = Vec::with_capacity(9);
    data.push(SPL_TOKEN_TRANSFER_INSTRUCTION);
    data.extend_from_slice(&amount.to_le_bytes());

    let instruction = Instruction {
        program_id: *token_program.key,
        accounts: vec![
            AccountMeta::new(*source.key, false),
            AccountMeta::new(*destination.key, false),
            AccountMeta::new_readonly(*authority.key, true),
        ],
        data,
    };

    let account_infos = [
        source.clone(),
        destination.clone(),
        authority.clone(),
        token_program.clone(),
    ];

    if let Some(seeds) = signer_seeds {
        invoke_signed(&instruction, &account_infos, &[seeds])
    } else {
        invoke(&instruction, &account_infos)
    }
}

fn read_pubkey(data: &[u8], start: usize) -> Result<Pubkey, ProgramError> {
    let end = start
        .checked_add(32)
        .ok_or(IcpxError::InvalidTokenAccount)?;
    let bytes = data
        .get(start..end)
        .ok_or(IcpxError::InvalidTokenAccount)?;
    let mut pubkey_bytes = [0; 32];
    pubkey_bytes.copy_from_slice(bytes);
    Ok(Pubkey::new_from_array(pubkey_bytes))
}

fn read_u64(data: &[u8], start: usize) -> Result<u64, ProgramError> {
    let end = start
        .checked_add(8)
        .ok_or(IcpxError::InvalidTokenAccount)?;
    let bytes = data
        .get(start..end)
        .ok_or(IcpxError::InvalidTokenAccount)?;
    let mut amount_bytes = [0; 8];
    amount_bytes.copy_from_slice(bytes);
    Ok(u64::from_le_bytes(amount_bytes))
}
