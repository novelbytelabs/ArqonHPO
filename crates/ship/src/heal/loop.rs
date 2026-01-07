use anyhow::Result;
use crate::heal::parser_rust::{RustLogParser, TestFailure};
use crate::heal::context::{ContextBuilder, HealContext};
use crate::heal::llm::{LlmClient, CandleLlm};
use crate::heal::prompts::PromptTemplates;
use crate::heal::apply::apply_fix;
use crate::heal::verify::VerificationGate;
use crate::oracle::OracleStore;
use std::path::PathBuf;

pub struct HealingLoop {
    context_builder: ContextBuilder,
    llm: CandleLlm,
    max_attempts: u32,
    root: PathBuf,
}

#[derive(Debug)]
pub enum HealOutcome {
    Success,
    CompileFailed,
    TestFailed,
    NoFixGenerated,
    MaxAttemptsExceeded,
}

impl HealingLoop {
    pub fn new(store: OracleStore, root: PathBuf, max_attempts: u32) -> Result<Self> {
        let context_builder = ContextBuilder::new(store, root.clone());
        let llm = CandleLlm::new()?;
        
        Ok(Self {
            context_builder,
            llm,
            max_attempts,
            root,
        })
    }
    
    pub fn run(&mut self, failure: &TestFailure) -> Result<HealOutcome> {
        for attempt in 0..self.max_attempts {
            println!("Healing attempt {}/{}", attempt + 1, self.max_attempts);
            
            // 1. Build context
            let ctx = self.context_builder.build_context(failure)?;
            
            // 2. Generate prompt
            let prompt = if failure.file_path.ends_with(".rs") {
                PromptTemplates::rust_repair(&ctx)
            } else {
                PromptTemplates::python_repair(&ctx)
            };
            
            // 3. Generate fix from LLM
            let fix = self.llm.generate_fix(&prompt)?;
            if fix.is_empty() {
                println!("LLM generated no fix");
                return Ok(HealOutcome::NoFixGenerated);
            }
            
            // 4. Apply fix
            let file_path = self.root.join(&failure.file_path);
            apply_fix(&file_path, &fix)?;
            
            // 5. Verify
            let gate = VerificationGate::new(self.root.clone());
            if gate.check_compile()? && gate.check_lint()? && gate.check_tests()? {
                return Ok(HealOutcome::Success);
            }
        }
        
        Ok(HealOutcome::MaxAttemptsExceeded)
    }
}
