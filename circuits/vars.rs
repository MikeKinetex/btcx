
use crate::consts::HEADER_BYTES_LENGTH;

use plonky2x::prelude::{
    U256Variable, U32Variable, Bytes32Variable, BytesVariable, CircuitBuilder, CircuitVariable, Variable, PlonkParameters, RichField
};

pub type HeaderBytesVariable = BytesVariable<HEADER_BYTES_LENGTH>;

pub type BlockHashVariable = Bytes32Variable;
pub type ThresholdVariable = U256Variable;
pub type WorkVariable = U256Variable;

#[derive(Debug, Clone, CircuitVariable)]
#[value_name(BitcoinHeaderType)]
pub struct BitcoinHeaderVariable {
    pub hash: BlockHashVariable,
    pub parent_hash: BlockHashVariable,
    pub merkle_root: Bytes32Variable,
    pub timestamp: U32Variable,
    pub threshold: ThresholdVariable,
}