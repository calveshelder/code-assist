# CodeAssist

An agentic terminal coding assistant that understands your codebase and helps you code faster.

## Features

- Edit files and fix bugs across your codebase
- Answer questions about your code's architecture and logic
- Execute and fix tests, linting, and other commands
- Search through git history, resolve merge conflicts, and create commits and PRs

## Usage

Run interactively:
```
code-assist
```

Execute a one-off command:
```
code-assist exec "fix the bug in auth.rs where users can't reset passwords"
```

Configure:
```
code-assist config --api_url="http://localhost:8000/v1" --model="gpt-3.5-turbo"
```

## Building

```
cargo build --release
```
