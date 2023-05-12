use cranelift_codegen::verifier::VerifierErrors;
use cranelift_module::ModuleError;

pub mod block_translator;
pub mod instruction_translator;

use std::{error::Error, fmt::Display};

use crate::disasm::armv4t::Instruction;

#[derive(Debug)]
pub enum TranslationError {
    Unimplemented(Instruction),
    Invalid(Instruction),
    CraneliftModuleError(ModuleError),
    CraneliftVerifierError(VerifierErrors),
}

impl Display for TranslationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for TranslationError {}

impl From<ModuleError> for TranslationError {
    fn from(err: ModuleError) -> Self {
        Self::CraneliftModuleError(err)
    }
}
impl From<VerifierErrors> for TranslationError {
    fn from(err: VerifierErrors) -> Self {
        Self::CraneliftVerifierError(err)
    }
}
