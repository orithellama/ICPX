use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    errors::IcpxError,
    instructions::{
        process_accept_job, process_cancel_expired_job, process_complete_job,
        process_create_gpu_job, process_fund_job, process_open_dispute, process_settle_stream,
        IcpxInstruction,
    },
};

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = IcpxInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::from(IcpxError::InvalidInstruction))?;

    match instruction {
        IcpxInstruction::CreateGpuJob(args) => process_create_gpu_job(program_id, accounts, args),
        IcpxInstruction::FundJob => process_fund_job(program_id, accounts),
        IcpxInstruction::AcceptJob => process_accept_job(program_id, accounts),
        IcpxInstruction::SettleStream(receipt) => {
            process_settle_stream(program_id, accounts, receipt)
        }
        IcpxInstruction::CompleteJob(receipt) => {
            process_complete_job(program_id, accounts, receipt)
        }
        IcpxInstruction::CancelExpiredJob => process_cancel_expired_job(program_id, accounts),
        IcpxInstruction::OpenDispute => process_open_dispute(program_id, accounts),
    }
}
