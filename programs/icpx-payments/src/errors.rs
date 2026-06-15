use solana_program::program_error::ProgramError;

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IcpxError {
    InvalidInstruction = 6000,
    InvalidPda,
    InvalidSigner,
    InvalidStatus,
    InvalidTerms,
    InvalidGpuTerms,
    InvalidReceipt,
    JobExpired,
    JobNotExpired,
    MathOverflow,
    EscrowUnderfunded,
    InvalidSystemProgram,
    InvalidTokenProgram,
    InvalidTokenAccount,
    InvalidTokenMint,
    InvalidTokenOwner,
    InvalidEscrowVault,
    InvalidPaymentAsset,
    InvalidProtocolFeeAccount,
}

impl From<IcpxError> for ProgramError {
    fn from(error: IcpxError) -> Self {
        ProgramError::Custom(error as u32)
    }
}

impl IcpxError {
    pub fn code(self) -> u32 {
        self as u32
    }

    pub fn name(self) -> &'static str {
        match self {
            IcpxError::InvalidInstruction => "InvalidInstruction",
            IcpxError::InvalidPda => "InvalidPda",
            IcpxError::InvalidSigner => "InvalidSigner",
            IcpxError::InvalidStatus => "InvalidStatus",
            IcpxError::InvalidTerms => "InvalidTerms",
            IcpxError::InvalidGpuTerms => "InvalidGpuTerms",
            IcpxError::InvalidReceipt => "InvalidReceipt",
            IcpxError::JobExpired => "JobExpired",
            IcpxError::JobNotExpired => "JobNotExpired",
            IcpxError::MathOverflow => "MathOverflow",
            IcpxError::EscrowUnderfunded => "EscrowUnderfunded",
            IcpxError::InvalidSystemProgram => "InvalidSystemProgram",
            IcpxError::InvalidTokenProgram => "InvalidTokenProgram",
            IcpxError::InvalidTokenAccount => "InvalidTokenAccount",
            IcpxError::InvalidTokenMint => "InvalidTokenMint",
            IcpxError::InvalidTokenOwner => "InvalidTokenOwner",
            IcpxError::InvalidEscrowVault => "InvalidEscrowVault",
            IcpxError::InvalidPaymentAsset => "InvalidPaymentAsset",
            IcpxError::InvalidProtocolFeeAccount => "InvalidProtocolFeeAccount",
        }
    }

    pub fn message(self) -> &'static str {
        match self {
            IcpxError::InvalidInstruction => "Instruction data could not be decoded.",
            IcpxError::InvalidPda => "A program-derived address did not match expected seeds.",
            IcpxError::InvalidSigner => "The required authority did not sign or does not match.",
            IcpxError::InvalidStatus => "The job status does not allow this instruction.",
            IcpxError::InvalidTerms => "The job terms are missing or outside allowed bounds.",
            IcpxError::InvalidGpuTerms => "GPU metering terms are missing or invalid.",
            IcpxError::InvalidReceipt => "The stream receipt is stale, replayed, or over budget.",
            IcpxError::JobExpired => "The job is already expired.",
            IcpxError::JobNotExpired => "The job has not reached its expiry slot.",
            IcpxError::MathOverflow => "A checked arithmetic operation overflowed.",
            IcpxError::EscrowUnderfunded => "The escrow balance cannot cover the requested amount.",
            IcpxError::InvalidSystemProgram => "The system program account is not valid.",
            IcpxError::InvalidTokenProgram => "The SPL token program account is not valid.",
            IcpxError::InvalidTokenAccount => "A token account is malformed or not initialized.",
            IcpxError::InvalidTokenMint => "A token account uses an unsupported mint.",
            IcpxError::InvalidTokenOwner => {
                "A token account owner does not match the required party."
            }
            IcpxError::InvalidEscrowVault => "The provided escrow vault is not the funded vault.",
            IcpxError::InvalidPaymentAsset => "The payment asset is unsupported for this path.",
            IcpxError::InvalidProtocolFeeAccount => {
                "The protocol fee destination does not match the hard-coded multisig."
            }
        }
    }
}
