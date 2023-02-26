use std::collections::BTreeSet;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, borsh::try_from_slice_unchecked, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::utils::{AccountInfoHelpers, ResultExt};

pub mod constants {
    use solana_program::declare_id;
    declare_id!("38pfsot7kCZkrttx1THEDXEz4JJXmCCcaDoDieRtVuy5");

    pub const CONFIG_VALIDATION_PHASE: u32 = 373_836_823;
    pub const STORAGE_VALIDATION_PHASE: u32 = 332_049_381;
    pub const NAME_STORAGE_VALIDATION_PHASE: u32 = 938_283_942;

    pub const MAX_PROGRAMS_PER_STORAGE_ACCOUNT: u32 = 625;
    pub const MAX_PROGRAMS_PER_NAME_STORAGE_ACCOUNT: u32 = 1666;

    pub const PROGRAM_DEPLOYMENT_PAYBACK: u64 = 1_000_000_000;

    pub mod team {
        solana_program::declare_id!("Team111111111111111111111111111111111111111");
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Config {
    pub validation_phase: u32,
    pub validator_numeration: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            validation_phase: constants::CONFIG_VALIDATION_PHASE,
            validator_numeration: 0,
        }
    }
}
impl Config {
    pub fn decode(account: &AccountInfo) -> Self {
        account
            .assert_owner(&constants::ID)
            .error_log("Inputed config account is not owned by the program")
            .unwrap();
        let config: Self = try_from_slice_unchecked(&account.data.borrow())
            .error_log("Error at config account try_from_slice")
            .unwrap();
        if config.validation_phase == constants::CONFIG_VALIDATION_PHASE {
            config
        } else {
            panic!("Error: @ config validation phase assertion.");
        }
    }
    pub fn get_space(&self) -> usize {
        8
    }
    pub fn get_storage_numeration(&self) -> u32 {
        self.validator_numeration
            .checked_div(constants::MAX_PROGRAMS_PER_STORAGE_ACCOUNT as u32)
            .unwrap()
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Storage {
    pub validation_phase: u32,
    pub programs: Vec<Pubkey>,
}
impl Default for Storage {
    fn default() -> Self {
        Self {
            validation_phase: constants::STORAGE_VALIDATION_PHASE,
            programs: Vec::new(),
        }
    }
}
impl Storage {
    pub fn decode(account: &AccountInfo) -> Self {
        account
            .assert_owner(&constants::ID)
            .error_log("Inputed storage account is not owned by the program")
            .unwrap();
        let storage: Storage = try_from_slice_unchecked(&account.data.borrow())
            .error_log("Error at storage account try_from_slice")
            .unwrap();
        if storage.validation_phase == constants::STORAGE_VALIDATION_PHASE {
            storage
        } else {
            panic!("Error: @ storage validation phase assertion.");
        }
    }

    pub fn get_space(&self) -> usize {
        4 + 4 + self.programs.len() * 32
    }
    pub fn verify(&self) -> Result<(), ProgramError> {
        if self.validation_phase != constants::STORAGE_VALIDATION_PHASE {
            return Err(ProgramError::InvalidAccountData);
        }
        if self.programs.len() > constants::MAX_PROGRAMS_PER_STORAGE_ACCOUNT as usize {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }
    pub fn add_program(&mut self, program: Pubkey) -> Result<(), ProgramError> {
        self.programs.push(program);
        self.verify()?;
        Ok(())
    }
    pub fn remove_program(&mut self, program: Pubkey) -> Result<(), ProgramError> {
        let index = self
            .programs
            .iter()
            .position(|x| *x == program)
            .ok_or(ProgramError::InvalidAccountData)?;
        self.programs.remove(index);
        self.verify()?;
        Ok(())
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct NameStorage {
    pub validation_phase: u32,
    pub names: BTreeSet<String>,
}

impl Default for NameStorage {
    fn default() -> Self {
        Self {
            validation_phase: constants::NAME_STORAGE_VALIDATION_PHASE,
            names: BTreeSet::new(),
        }
    }
}
impl NameStorage {
    pub fn add_name(&mut self, name: &str) -> Result<(), ProgramError> {
        let mut tmp_name = String::new();
        for char in name.chars() {
            if char.is_ascii_alphanumeric() {
                tmp_name.push(char.to_ascii_lowercase());
            }
        }
        Self::validate_name(&tmp_name)?;
        if self.names.contains(&tmp_name) {
            msg!("Error: @ name is too similar to an existing name.");
            Err(ProgramError::Custom(1))?
        }
        self.names.insert(tmp_name);
        Ok(())
    }

    pub fn validate_name(name: &str) -> Result<(), ProgramError> {
        if name.len() > 12 || name.len() < 1 {
            msg!("Error: @ name length assertion.");
            Err(ProgramError::Custom(2))?
        }
        Ok(())
    }

    pub fn get_space(&self) -> usize {
        let mut str_lens = 0;
        for name in self.names.iter() {
            str_lens += name.len() + 4;
        }
        4 + 4 + str_lens
    }

    pub fn decode(account: &AccountInfo) -> Self {
        account
            .assert_owner(&constants::ID)
            .error_log("Inputed name storage account is not owned by the program")
            .unwrap();
        let name_storage: NameStorage = try_from_slice_unchecked(&account.data.borrow())
            .error_log("Error at name storage account try_from_slice")
            .unwrap();
        if name_storage.validation_phase == constants::NAME_STORAGE_VALIDATION_PHASE {
            name_storage
        } else {
            panic!("Error: @ name storage validation phase assertion.");
        }
    }
}
