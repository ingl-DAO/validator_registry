use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, borsh::try_from_slice_unchecked, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::utils::{AccountInfoHelpers, ResultExt};

use self::constants::MAX_NAME_LENGTH;

pub mod constants {
    use solana_program::declare_id;
    declare_id!("38pfsot7kCZkrttx1THEDXEz4JJXmCCcaDoDieRtVuy5");

    pub const CONFIG_VALIDATION_PHASE: u32 = 373_836_823;
    pub const STORAGE_VALIDATION_PHASE: u32 = 332_049_381;
    pub const NAME_STORAGE_VALIDATION_PHRASE: u32 = 938_283_942;
    pub const MARKETPLACE_STORAGE_VALIDATION_PHRASE: u32 = 728_721_427;

    pub const MAX_NAME_LENGTH: usize = 12;

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
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Storage {
    pub validation_phase: u32,
    pub num_programs: u32,
    //Programs are stored here like this: [program_id1, program_id2, program_id3, etc...] with each program being 32bytes long.
}
impl Default for Storage {
    fn default() -> Self {
        Self {
            validation_phase: constants::STORAGE_VALIDATION_PHASE,
            num_programs: 0,
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
        Self::get_init_space() + (self.num_programs as usize * 32)
    }
    pub fn get_init_space() -> usize {
        8 // 4 bytes for validation_phase and 4 bytes for num_programs
    }
    pub fn find_program(
        program_id: Pubkey,
        account: &AccountInfo,
        optimistic_index: Option<u32>,
    ) -> Option<u32> {
        let storage = Self::decode(account);
        let ind = optimistic_index.unwrap_or(storage.num_programs - 1) as i32;
        let mut cnt = 0;
        while ind - cnt >= 0 {
            let start_ind = Self::get_init_space() + ((ind - cnt) as usize * 32);
            let end_ind = start_ind + 32;
            let program = Pubkey::new(&account.data.borrow()[start_ind..end_ind]);
            if program == program_id {
                return Some((ind - cnt) as u32);
            }
            cnt += 1;
        }
        None
    }
    pub fn add_program(
        &mut self,
        program: Pubkey,
        account: &AccountInfo,
    ) -> Result<(), ProgramError> {
        if let Some(_) = Self::find_program(constants::ID, account, None) {
            msg!("Error: @ Program already exists in the storage account.");
            Err(ProgramError::Custom(0))?
        }
        let start_ind = Self::get_init_space() + (self.num_programs as usize * 32);
        let end_ind = start_ind + 32;
        account.data.borrow_mut()[start_ind..end_ind].copy_from_slice(&program.to_bytes());
        self.num_programs += 1;
        Ok(())
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct NameStorage {
    pub validation_phrase: u32,
    pub num_names: u32,
    //The names are stored here like this: "name1\0\0...", "name2\0\0...", "name3\0\0...", etc. with each name being a maximum ofMAX_NAME_LENGTH characters long.
}

impl Default for NameStorage {
    fn default() -> Self {
        Self {
            validation_phrase: constants::NAME_STORAGE_VALIDATION_PHRASE,
            num_names: 0,
        }
    }
}
impl NameStorage {
    pub fn add_name(&mut self, name: &str, account: &AccountInfo) -> Result<(), ProgramError> {
        let mut tmp_name = String::new();
        for char in name.chars() {
            if char.is_ascii_alphanumeric() {
                tmp_name.push(char.to_ascii_lowercase());
            }
        }
        if tmp_name.len() > MAX_NAME_LENGTH || tmp_name.len() < 1 {
            msg!("Error: @ name length assertion.");
            Err(ProgramError::Custom(2))?
        }

        tmp_name.push_str(&"_".repeat(12 - tmp_name.len()));

        if let Some(_) = Self::find_name(&tmp_name, account, None) {
            msg!("Error: @ name is too similar to an existing name.");
            Err(ProgramError::Custom(3))?
        }
        let mut account_data = account.data.borrow_mut();
        let name_start = Self::get_init_space() + (12 * self.num_names as usize);
        let name_end = name_start + MAX_NAME_LENGTH;
        account_data[name_start..name_end].copy_from_slice(tmp_name.as_bytes());
        self.num_names += 1;
        account_data[name_end..name_end + MAX_NAME_LENGTH].fill(0);
        Ok(())
    }

    pub fn find_name(
        name: &str,
        account: &AccountInfo,
        optimistic_index: Option<u32>,
    ) -> Option<u32> {
        let account_struct_data = Self::decode(account);
        let account_data = account.data.borrow();

        if account_struct_data.num_names == 0 {
            return None;
        }

        let optimistic_ind = optimistic_index.unwrap_or(account_struct_data.num_names - 1) as i32;

        let mut cnt = 0;
        while optimistic_ind - cnt >= 0 {
            let name_start = Self::get_init_space() + (12 * (optimistic_ind - cnt) as usize);
            let name_end = name_start + MAX_NAME_LENGTH;
            let name_slice = &account_data[name_start..name_end];
            let name_str = std::str::from_utf8(name_slice).unwrap();
            if name_str == name {
                return Some((optimistic_ind - cnt) as u32);
            }
            cnt += 1;
        }

        None
    }

    pub fn get_space(&self) -> usize {
        Self::get_init_space() + (MAX_NAME_LENGTH * self.num_names as usize)
    }
    pub fn get_init_space() -> usize {
        8 // 4 validation phrase + 4 num_names
    }

    pub fn decode(account: &AccountInfo) -> Self {
        account
            .assert_owner(&constants::ID)
            .error_log("Inputed name storage account is not owned by the program")
            .unwrap();
        let name_storage: NameStorage = try_from_slice_unchecked(&account.data.borrow())
            .error_log("Error at name storage account try_from_slice")
            .unwrap();
        if name_storage.validation_phrase == constants::NAME_STORAGE_VALIDATION_PHRASE {
            name_storage
        } else {
            panic!("Error: @ name storage validation phase assertion.");
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct MarketplaceStorage {
    pub validation_phase: u32,
    pub num_programs: u32,
    //Programs are stored here like this: [program_id1, program_id2, program_id3, etc...] with each program being 32bytes long.
}
impl Default for MarketplaceStorage {
    fn default() -> Self {
        Self {
            validation_phase: constants::MARKETPLACE_STORAGE_VALIDATION_PHRASE,
            num_programs: 0,
        }
    }
}
impl MarketplaceStorage {
    pub fn decode(account: &AccountInfo) -> Self {
        account
            .assert_owner(&constants::ID)
            .error_log("Inputed storage account is not owned by the program")
            .unwrap();
        let storage: MarketplaceStorage = try_from_slice_unchecked(&account.data.borrow())
            .error_log("Error at storage account try_from_slice")
            .unwrap();
        if storage.validation_phase == constants::MARKETPLACE_STORAGE_VALIDATION_PHRASE {
            storage
        } else {
            panic!("Error: @ storage validation phase assertion.");
        }
    }
    pub fn get_space(&self) -> usize {
        Self::get_init_space() + (self.num_programs as usize * 32)
    }
    pub fn get_init_space() -> usize {
        8 // 4 bytes for validation_phase and 4 bytes for num_programs
    }
    pub fn find_program(
        program_id: Pubkey,
        account: &AccountInfo,
        optimistic_index: Option<u32>,
    ) -> Option<u32> {
        let storage = Self::decode(account);
        let ind = optimistic_index.unwrap_or(storage.num_programs - 1) as i32;
        let mut cnt = 0;
        while ind - cnt >= 0 {
            let start_ind = Self::get_init_space() + ((ind - cnt) as usize * 32);
            let end_ind = start_ind + 32;
            let program = Pubkey::new(&account.data.borrow()[start_ind..end_ind]);
            if program == program_id {
                return Some((ind - cnt) as u32);
            }
            cnt += 1;
        }
        None
    }
    pub fn add_program(
        &mut self,
        program: Pubkey,
        account: &AccountInfo,
    ) -> Result<(), ProgramError> {
        if let Some(_) = Self::find_program(constants::ID, account, None) {
            msg!("Error: @ Program already exists in the storage account.");
            Err(ProgramError::Custom(0))?
        }
        let start_ind = Self::get_init_space() + (self.num_programs as usize * 32);
        let end_ind = start_ind + 32;
        account.data.borrow_mut()[start_ind..end_ind].copy_from_slice(&program.to_bytes());
        self.num_programs += 1;
        Ok(())
    }
}
