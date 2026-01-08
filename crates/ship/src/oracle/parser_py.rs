use tree_sitter::{Parser, Tree, Language};
use anyhow::{Result, Context};

pub struct PythonParser {
    parser: Parser,
}

impl PythonParser {
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        // tree-sitter 0.22+ API: LanguageFn into_raw() -> Language
        let language: Language = tree_sitter_python::LANGUAGE.into();
        parser.set_language(&language).context("Error loading Python grammar")?;
        Ok(Self { parser })
    }

    pub fn parse(&mut self, code: &str) -> Option<Tree> {
        self.parser.parse(code, None)
    }
}
