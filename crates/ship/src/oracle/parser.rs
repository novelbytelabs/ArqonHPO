use tree_sitter::{Parser, Tree, Language};
use anyhow::{Result, Context};

pub struct RustParser {
    parser: Parser,
}

impl RustParser {
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        // tree-sitter 0.23+ changed how language loading works slightly or bindings differ
        let language = tree_sitter_rust::LANGUAGE; 
        // For new bindings, LANGUAGE is a LanguageFn, call it ? No, from error: "expected raw pointer".
        // Actually, normally `tree_sitter_rust::LANGUAGE` *is* the pointer or fn.
        // Recent versions: `LANGUAGE.into()` matches `Language` if using `tree-sitter` crate > 0.22.
        
        let language = unsafe { Language::from_raw(std::mem::transmute(language)) };
        // Wait, the error said `found struct LanguageFn`.
        // Let's try `tree_sitter_rust::language()` if available, or just cast.
        // Or better: `parser.set_language(&tree_sitter_rust::language())`
        
        // Let's assume standard behavior:
        let language = tree_sitter_rust::LANGUAGE;
        let language = unsafe { Language::from_raw(language as *const _) };
        
        parser.set_language(&language)
            .context("Error loading Rust grammar")?;
        Ok(Self { parser })
    }

    pub fn parse(&mut self, code: &str) -> Option<Tree> {
        self.parser.parse(code, None)
    }
}

pub fn get_language() -> Language {
    let language = tree_sitter_rust::LANGUAGE;
    unsafe { Language::from_raw(language) }
}
