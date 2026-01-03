//! WASM compilation utilities.

use serde::{Deserialize, Serialize};

/// Compile WAT (WebAssembly Text format) to WASM binary
pub fn compile_wat(wat: &str) -> Result<Vec<u8>, CompileError> {
    wat::parse_str(wat).map_err(|e| CompileError::ParseError(e.to_string()))
}

/// Validate a WASM binary
pub fn validate_wasm(wasm: &[u8]) -> Result<(), ValidateError> {
    // Use wasmi to validate
    let engine = wasmi::Engine::default();
    wasmi::Module::new(&engine, wasm)
        .map_err(|e| ValidateError::InvalidWasm(e.to_string()))?;
    Ok(())
}

/// Extract metadata from WASM module
pub fn extract_metadata(wasm: &[u8]) -> Result<WasmMetadata, MetadataError> {
    let engine = wasmi::Engine::default();
    let module = wasmi::Module::new(&engine, wasm)
        .map_err(|e| MetadataError::LoadFailed(e.to_string()))?;

    let exports = module
        .exports()
        .map(|e| e.name().to_string())
        .collect();

    let imports = module
        .imports()
        .map(|i| {
            ImportInfo {
                module: i.module().to_string(),
                name: i.name().to_string(),
                ty: format!("{:?}", i.ty()),
            }
        })
        .collect();

    Ok(WasmMetadata {
        exports,
        imports,
    })
}

/// WASM module metadata
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WasmMetadata {
    pub exports: Vec<String>,
    pub imports: Vec<ImportInfo>,
}

/// Import information
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportInfo {
    pub module: String,
    pub name: String,
    pub ty: String,
}

/// Compilation errors
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompileError {
    ParseError(String),
    InvalidWat(String),
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            CompileError::InvalidWat(msg) => write!(f, "Invalid WAT: {}", msg),
        }
    }
}

impl std::error::Error for CompileError {}

/// Validation errors
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValidateError {
    InvalidWasm(String),
    MissingExport(String),
    ForbiddenImport(String),
}

impl std::fmt::Display for ValidateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidateError::InvalidWasm(msg) => write!(f, "Invalid WASM: {}", msg),
            ValidateError::MissingExport(name) => write!(f, "Missing export: {}", name),
            ValidateError::ForbiddenImport(name) => write!(f, "Forbidden import: {}", name),
        }
    }
}

impl std::error::Error for ValidateError {}

/// Metadata errors
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MetadataError {
    LoadFailed(String),
}

impl std::fmt::Display for MetadataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetadataError::LoadFailed(msg) => write!(f, "Load failed: {}", msg),
        }
    }
}

impl std::error::Error for MetadataError {}

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE_WAT: &str = r#"
        (module
            (memory (export "memory") 1)
            (func (export "run") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add
            )
            (func (export "alloc") (param i32) (result i32)
                local.get 0
            )
        )
    "#;

    #[test]
    fn test_compile_wat() {
        let wasm = compile_wat(SIMPLE_WAT).unwrap();
        assert!(!wasm.is_empty());
    }

    #[test]
    fn test_validate_wasm() {
        let wasm = compile_wat(SIMPLE_WAT).unwrap();
        assert!(validate_wasm(&wasm).is_ok());
    }

    #[test]
    fn test_extract_metadata() {
        let wasm = compile_wat(SIMPLE_WAT).unwrap();
        let metadata = extract_metadata(&wasm).unwrap();

        assert!(metadata.exports.contains(&"memory".to_string()));
        assert!(metadata.exports.contains(&"run".to_string()));
        assert!(metadata.exports.contains(&"alloc".to_string()));
    }
}
