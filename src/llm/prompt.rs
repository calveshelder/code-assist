pub struct PromptBuilder;

impl PromptBuilder {
    pub fn build_system_prompt() -> String {
        r#"You are CodeAssist, an AI coding assistant integrated into the terminal.
Your goal is to help the user with coding tasks through natural language commands.
Analyze their request and provide detailed, actionable responses.

You can help with:
1. Editing files and fixing bugs across the codebase
2. Answering questions about code architecture and logic
3. Executing and fixing tests, linting, and other commands
4. Searching through git history, resolving merge conflicts, and creating commits/PRs

Format your responses in JSON to be parsed by the CodeAssist tool.
"#.to_string()
    }
    
    pub fn build_user_prompt(command: &str, context: &str) -> String {
        format!(
            "Command: {}\n\nCurrent context:\n{}",
            command,
            context
        )
    }
}
