use tree_sitter::{Parser, Tree, Language};
use anyhow::{Result, Context};

pub struct PythonParser {
    parser: Parser,
}

impl PythonParser {
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        let language = tree_sitter_python::LANGUAGE;
        let language = unsafe { Language::from_raw(language) };
        parser.set_language(&language)
            .context("Error loading Python grammar")?;
        Ok(Self { parser })
    }

    pub fn parse(&mut self, code: &str) -> Option<Tree> {
        self.parser.parse(code, None)
    }
}
