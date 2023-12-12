
use crate::consts::HEADER_BYTES_LENGTH;

use plonky2x::prelude::{
    Bytes32Variable, BytesVariable, U256Variable,
    CircuitVariable, Variable, CircuitBuilder, PlonkParameters, RichField
};

pub type HeaderBytesVariable = BytesVariable<HEADER_BYTES_LENGTH>;

pub type BlockHashVariable = Bytes32Variable;
pub type WorkVariable = U256Variable;

#[derive(Debug, Clone, CircuitVariable)]
#[value_name(BitcoinHeaderType)]
pub struct BitcoinHeaderVariable {
    pub hash: BlockHashVariable,
    pub work: WorkVariable,
}