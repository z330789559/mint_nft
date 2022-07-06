use borsh::{BorshDeserialize, BorshSerialize};

#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum GameInstruction {
    Mint,
}
