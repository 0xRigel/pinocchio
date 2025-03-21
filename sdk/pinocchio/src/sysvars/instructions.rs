use crate::{
    account_info::AccountInfo,
    instruction::AccountMeta,
    program_error::ProgramError,
    sanitize_error::SanitizeError,
    pubkey::Pubkey
};

use core::mem::size_of;

/// Sysvar1nstructions1111111111111111111111111
const INSTRUCTIONS_ID: Pubkey = [
    0x06, 0xa7, 0xd5, 0x17, 0x18, 0x7b, 0xd1, 0x66, 
    0x35, 0xda, 0xd4, 0x04, 0x55, 0xfd, 0xc2, 0xc0, 
    0xc1, 0x24, 0xc6, 0x8f, 0x21, 0x56, 0x75, 0xa5, 
    0xdb, 0xba, 0xcb, 0x5f, 0x08, 0x00, 0x00, 0x00, 
];

pub struct Instructions();

/// Load the current `Instruction`'s index in the currently executing
/// `Transaction`.
///
/// `data` is the instructions sysvar account data.
///
/// Unsafe because the sysvar accounts address is not checked; only used
/// internally after such a check.
#[inline(always)]
fn load_current_index(data: &[u8]) -> u16 {
    let len = data.len();
    unsafe {
        u16::from_le(*(data.as_ptr().add(len - 2) as *const u16))
    }
}

/// Load the current `Instruction`'s index in the currently executing
/// `Transaction`.
///
/// # Errors
///
/// Returns [`ProgramError::UnsupportedSysvar`] if the given account's ID is not equal to [`ID`].
#[inline(always)]
pub fn load_current_index_checked(
    instruction_sysvar_account_info: &AccountInfo,
) -> Result<u16, ProgramError> {
    if instruction_sysvar_account_info.key() != &INSTRUCTIONS_ID {
        return Err(ProgramError::UnsupportedSysvar);
    }

    let instruction_sysvar = instruction_sysvar_account_info.try_borrow_data()?;
    let index = load_current_index(&instruction_sysvar);
    Ok(index)
}

/// Store the current `Instruction`'s index in the instructions sysvar data.
#[inline(always)]
pub fn store_current_index(data: &mut [u8], instruction_index: u16) {
    let last_index = data.len() - 2;
    unsafe {
        *(data.as_mut_ptr().add(last_index) as *mut u16) = instruction_index.to_le();
    }
}

/// Load an `Instruction` in the currently executing `Transaction` at the
/// specified index.
///
/// `data` is the instructions sysvar account data.
///
/// Unsafe because the sysvar accounts address is not checked; only used
/// internally after such a check.
#[inline(always)]
fn load_instruction_at(index: usize, data: &[u8]) -> &IntrospectedInstruction {
    unsafe {
        &*(data.as_ptr().add(size_of::<u16>() + index * size_of::<u16>()) as *const IntrospectedInstruction)
    }
}

#[inline(always)]
fn load_instruction_at_checked(index: usize, data: &[u8]) -> Result<&IntrospectedInstruction, SanitizeError> {
    unsafe {
        let num_instructions = u16::from_le(*(data.as_ptr() as *const u16));
        if index >= num_instructions as usize {
            return Err(SanitizeError::IndexOutOfBounds);
        }

        Ok(load_instruction_at(index, data))
    }
}

/// Returns the `Instruction` relative to the current `Instruction` in the
/// currently executing `Transaction`.
///
/// # Errors
///
/// Returns [`ProgramError::UnsupportedSysvar`] if the given account's ID is not equal to [`ID`].
pub fn get_instruction_relative(
    index_relative_to_current: i64,
    instruction_sysvar_account_info: &AccountInfo,
) -> Result<IntrospectedInstruction, ProgramError> {
    if instruction_sysvar_account_info.key() != &INSTRUCTIONS_ID {
        return Err(ProgramError::UnsupportedSysvar);
    }

    let instruction_sysvar = instruction_sysvar_account_info.try_borrow_data()?;
    let current_index = load_current_index(&instruction_sysvar) as i64;
    let index = current_index.saturating_add(index_relative_to_current);
    if index < 0 {
        return Err(ProgramError::InvalidArgument);
    }

    load_instruction_at_checked(
        index as usize,
        &instruction_sysvar,
    ).map(|instr| instr.clone())
     .map_err(|err| match err {
        SanitizeError::IndexOutOfBounds => ProgramError::InvalidArgument,
        _ => ProgramError::InvalidInstructionData,
    })
}


// #[repr(C)]
// #[derive(Clone, PartialEq, Eq)]
// pub struct Instructions<'a> {
//     pub(crate) account_info: &'a AccountInfo,
// }


// impl<'a> From<&'a AccountInfo> for Instructions<'a> {
//     fn from(account_info: &'a AccountInfo) -> Self {
//         Self { account_info }
//     }
// }

// impl<'a> Instructions<'a> {
//     pub fn new(account_info: &'a AccountInfo) -> Result<Self, ProgramError> {
//         let sysvar = Self::new_unchecked(account_info);
//         if !sysvar.check_id() {
//             return Err(ProgramError::Custom(
//                 InstructionSysvarError::InvalidAccountId as u32,
//             ));
//         }
//         Ok(sysvar)
//     }

//     pub fn new_unchecked(account_info: &'a AccountInfo) -> Self {
//         Self { account_info }
//     }

//     pub fn check_id(&self) -> bool {
//         self.account_info.key() == &INSTRUCTIONS_ID
//     }

//     pub fn get_instruction_count(&self) -> Result<usize, ProgramError> {
//         let mut current = 0;
//         let data = self.account_info.try_borrow_data()?;
//         let num_instructions = read_u16(&mut current, &data)
//             .map_err(|_| ProgramError::Custom(InstructionSysvarError::InvalidAccountData as u32))?;
//         Ok(num_instructions as usize)
//     }
//     pub fn load_instruction_at_checked(
//         self,
//         index: usize,
//     ) -> Result<IntrospectedInstruction, ProgramError> {
//         // We need to make calculations based on the data, but we don't need to keep
//         // the Ref alive after this function returns
//         unsafe {
//             let data_ref = self.account_info.try_borrow_data()?;
//             let data_ptr = data_ref.as_ptr();

//             let mut current = 0;

//             // Get number of instructions
//             let num_instructions = read_u16(&mut current, &data_ref).map_err(|_| {
//                 ProgramError::Custom(InstructionSysvarError::InvalidAccountData as u32)
//             })?;

//             if index >= num_instructions as usize {
//                 return Err(ProgramError::Custom(
//                     InstructionSysvarError::InvalidAccountData as u32,
//                 ));
//             }

//             // Calculate offset to this instruction's location
//             current += index * 2;
//             let instruction_start = read_u16(&mut current, &data_ref).map_err(|_| {
//                 ProgramError::Custom(InstructionSysvarError::InvalidAccountData as u32)
//             })?;

//             // Move to the start of the instruction
//             current = instruction_start as usize;

//             // Read the number of accounts
//             let num_accounts = read_u16(&mut current, &data_ref).map_err(|_| {
//                 ProgramError::Custom(InstructionSysvarError::InvalidAccountData as u32)
//             })?;

//             // Calculate important offsets
//             let program_id_offset = current + (num_accounts as usize * 33);
//             let ix_data_offset = program_id_offset + core::mem::size_of::<Pubkey>();

//             // Read instruction data length
//             let mut data_len_pos = ix_data_offset;
//             let ix_data_len = read_u16(&mut data_len_pos, &data_ref).map_err(|_| {
//                 ProgramError::Custom(InstructionSysvarError::InvalidAccountData as u32)
//             })?;

//             // Calculate total instruction length
//             let total_len = ix_data_offset + 2 + ix_data_len as usize;

//             // Create the IntrospectedInstruction with raw pointer and metadata
//             Ok(IntrospectedInstruction {
//                 data_ptr: data_ptr.add(instruction_start as usize),
//                 data_len: total_len - instruction_start as usize,
//                 num_accounts,
//                 // Offset is relative to the start of the instruction
//                 program_id_offset: program_id_offset - instruction_start as usize,
//                 // Offset is relative to the start of the instruction
//                 ix_data_offset: ix_data_offset + 2 - instruction_start as usize, // +2 to skip the length field
//                 ix_data_len,
//             })
//         }
//     }
// }

#[repr(C)]
#[derive(Clone, PartialEq, Eq)]
pub struct IntrospectedInstruction {
    pub raw: *const u8,
}

impl IntrospectedInstruction {
    pub fn get_account_meta_at_unchecked(&self, index: usize) -> &IntrospectedAccountMeta {
        unsafe {
            &*(self.raw.add(size_of::<u16>() + index * size_of::<IntrospectedAccountMeta>()) as *const IntrospectedAccountMeta)
        }
    }

    pub fn get_account_meta_at(
        &self,
        index: usize,
    ) -> Result<&IntrospectedAccountMeta, SanitizeError> {
        if index >=  unsafe { u16::from_le(*(self.raw as *const u16)) } as usize {
            return Err(SanitizeError::IndexOutOfBounds);
        }

        Ok(self.get_account_meta_at_unchecked(index))
    }

    pub fn get_program_id(&self) -> &Pubkey {
        unsafe {
            let num_accounts = u16::from_le(*(self.raw as *const u16));
            &*(self.raw.add(size_of::<u16>() + num_accounts as usize * size_of::<IntrospectedAccountMeta>()) as *const Pubkey)
        }
    }

    pub fn get_data(&self) -> &[u8] {
        unsafe {
            let num_accounts = u16::from_le(*(self.raw as *const u16));
            let data_len = u16::from_le(*(self.raw.add(size_of::<u16>() + num_accounts as usize * size_of::<IntrospectedAccountMeta>() + size_of::<Pubkey>()) as *const u16));
            core::slice::from_raw_parts(self.raw.add(size_of::<u16>() + num_accounts as usize * size_of::<IntrospectedAccountMeta>() + size_of::<Pubkey>() + size_of::<u16>()), data_len as usize)
        }
    }

    // pub fn get_account_metas(&self) -> &[IntrospectedAccountMeta] {
    //     unsafe {
    //         let accounts = core::mem::MaybeUninit::<[IntrospectedAccountMeta; u16::from_le(*(self.accounts as *const u16))]>::uninit();
    //     }
    // }

    // pub fn to_instruction<'s, 'a, 'b>(
    //     &'s self,
    //     account_meta_buffer: &'b mut [AccountMeta<'a>],
    // ) -> Result<Instruction<'a, 'b, 's, 's>, SanitizeError>
    // where
    //     'a: 'b,
    //     's: 'a,
    // {
    //     let metas = self.get_account_metas();
    //     if account_meta_buffer.len() < metas.len() {
    //         return Err(SanitizeError::IndexOutOfBounds);
    //     }

    //     // Fill the buffer with account metas
    //     for (i, meta) in metas.iter().enumerate() {
    //         account_meta_buffer[i] = meta.to_account_meta();
    //     }
    //     Ok(Instruction {
    //         program_id: self.get_program_id(),
    //         accounts: &account_meta_buffer[..metas.len()],
    //         data: self.get_instruction_data(),
    //     })
    // }
}

#[repr(C)]
#[derive(Clone, PartialEq, Eq)]
pub struct IntrospectedAccountMeta {
    pub raw: *const u8,
}

// Define these constants at the top of your file
const IS_SIGNER_BIT: u8 = 0; // Assuming bit 1 for signer flag
const IS_WRITABLE_BIT: u8 = 1; // Assuming bit 0 for writable flag

impl IntrospectedAccountMeta {
    pub fn is_writable(&self) -> bool {
        unsafe {
            (*self.raw & (1 << IS_WRITABLE_BIT)) != 0
        }
    }

    pub fn is_signer(&self) -> bool {
        unsafe {
            (*self.raw & (1 << IS_SIGNER_BIT)) != 0
        }
    }

    pub fn key(&self) -> &Pubkey {
        unsafe {
            &*(self.raw.add(size_of::<u8>()) as *const Pubkey)
        }
    }

    pub fn to_account_meta(&self) -> AccountMeta {
        AccountMeta::new(self.key(), self.is_signer(), self.is_writable())
    }
}