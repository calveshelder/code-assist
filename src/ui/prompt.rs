use std::io::{self, Write};
use anyhow::Result;
use colored::Colorize;

pub struct Prompt;

impl Prompt {
    pub fn new() -> Self {
        Self
    }
    
    pub fn get_input(&self) -> Result<String> {
        print!("{} ", ">>".bright_green().bold());
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        Ok(input)
    }
}
