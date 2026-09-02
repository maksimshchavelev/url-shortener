use crate::domain;
use crate::domain::ShortCode;
use nanoid::nanoid;

/// Generates short code with length 8 without certain 
/// characters that resemble one another
pub struct CodeGenerator {
    /// Alphabet
    alphabet: Vec<char>,
}

impl CodeGenerator {
    /// Create new `CodeGenerator`
    pub fn new() -> Self {
        Self {
            alphabet: "23456789ABCDEFGHKLMNPQRSTVWXYZabcdefghikmnopqrstvwxyz"
                .chars()
                .collect(),
        }
    }
}

impl domain::CodeGenerator for CodeGenerator {
    fn generate(&self) -> ShortCode {
        ShortCode(nanoid!(8, &self.alphabet))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::CodeGenerator as _;

    #[test]
    fn generates_code_with_length_8() {
        let generator = CodeGenerator::new();
        assert_eq!(generator.generate().0.chars().count(), 8);
    }

    #[test]
    fn generates_different_codes() {
        let generator = CodeGenerator::new();

        let first = generator.generate();
        let second = generator.generate();

        assert_ne!(first, second);
    }
}
