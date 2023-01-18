use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

pub trait PubkeyHelpers {
    fn assert_match(&self, a: &Pubkey) -> ProgramResult;
}

impl PubkeyHelpers for Pubkey {
    fn assert_match(&self, a: &Pubkey) -> ProgramResult {
        if self != a {
            Err(ProgramError::Custom(0))?
        }
        Ok(())
    }
}

pub trait AccountInfoHelpers {
    fn assert_key_match(&self, a: &Pubkey) -> ProgramResult;
    fn assert_owner(&self, a: &Pubkey) -> ProgramResult;
    fn assert_signer(&self) -> ProgramResult;
    fn assert_seed(
        &self,
        program_id: &Pubkey,
        seed: &[&[u8]],
    ) -> Result<(Pubkey, u8), ProgramError>;
}

impl AccountInfoHelpers for AccountInfo<'_> {
    fn assert_key_match(&self, a: &Pubkey) -> ProgramResult {
        self.key.assert_match(a)
    }
    fn assert_owner(&self, a: &Pubkey) -> ProgramResult {
        self.owner
            .assert_match(a)
            .error_log("Error: @ owner assertion.")
    }
    fn assert_signer(&self) -> ProgramResult {
        if !self.is_signer {
            Err(ProgramError::MissingRequiredSignature)?
        }
        Ok(())
    }
    fn assert_seed(
        &self,
        program_id: &Pubkey,
        seed: &[&[u8]],
    ) -> Result<(Pubkey, u8), ProgramError> {
        let (key, bump) = Pubkey::find_program_address(seed, program_id);
        self.assert_key_match(&key)
            .error_log("Error: @ PDA Assertion")?;
        Ok((key, bump))
    }
}

pub trait ResultExt<T, E> {
    fn error_log(self, message: &str) -> Self;
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    ///Logs the error message if the result is an error, then returns the Err.
    /// If the result is an Ok(x), returns the Ok(x).
    fn error_log(self, message: &str) -> Self {
        match self {
            Ok(v) => Ok(v),
            Err(e) => {
                msg!("Error: {:?}", message);
                Err(e)
            }
        }
    }
}

pub trait OptionExt<T> {
    fn error_log(self, message: &str) -> Result<T, ProgramError>;
}

impl<T> OptionExt<T> for Option<T> {
    fn error_log(self, message: &str) -> Result<T, ProgramError> {
        match self {
            Some(v) => Ok(v),
            _ => {
                msg!("OptionUnwrapError: {}", message);
                Err(ProgramError::Custom(1))?
            }
        }
    }
}
