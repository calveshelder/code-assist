# Project Memory for CodeAssist

## Project Overview
CodeAssist is a Rust-based CLI application that uses LLMs to help developers understand and work with codebases. It analyzes project structure, identifies file types, and provides context-aware assistance.

Key features:
- Code structure analysis for multiple languages (Rust, Python, JavaScript/TypeScript, PHP, Go)
- Project type detection (Rust, Python, JavaScript, TypeScript, PHP, Go, Drupal, Angular, React)
- Contextual search with language-aware relevance scoring
- Memory system to store project-specific information
- Language-specific framework detection and analysis

## Frequently Used Commands
```
# Build the project
cargo build

# Run the application
cargo run

# Run tests
cargo test

# Run linting
cargo clippy
```

## Code Conventions
- Error handling with anyhow::Result
- Modular design with separate components for different concerns
- Rust naming conventions (snake_case for functions, CamelCase for types)
- Struct-based implementations with clear separation of responsibilities

## Architecture
The application follows a modular architecture:

1. **App Core**
   - `app.rs`: Main application logic and coordination
   - `main.rs`: Entry point

2. **Analysis Module**
   - `parser.rs`: Multi-language code parsing (Rust, Python, JS/TS, PHP, Go)
   - `structure.rs`: Project type detection and language-specific structure analysis

3. **File System Operations**
   - `fs/search.rs`: Smart file search with language-aware relevance scoring
   - `fs/edit.rs`: File editing capabilities

4. **LLM Integration**
   - `llm/client.rs`: LLM API client with support for local and remote models
   - `llm/context.rs`: Context management with language-specific optimizations
   - `llm/prompt.rs`: LLM prompt engineering

5. **Command Execution**
   - `commands/executor.rs`: Executes commands interpreted by the LLM

6. **Memory System**
   - `memory/mod.rs`: Project memory persistence

7. **User Interface**
   - `ui/display.rs`: Output formatting
   - `ui/prompt.rs`: Input prompting

## Language Support
The application supports these languages with varying capabilities:

1. **Rust**
   - Detects modules, structs, and functions
   - Full parsing support
   - Cargo package analysis
   - Crate dependency tracking

2. **Python**
   - Detects classes and functions
   - Framework detection (Django, Flask, FastAPI)
   - Virtual environment support
   - Module hierarchy analysis

3. **JavaScript/TypeScript**
   - Detects classes, functions, components, and hooks
   - Framework detection (React, Angular, Next.js)
   - Component and service analysis
   - State management support (Redux, NgRx)

4. **PHP**
   - Detects classes, interfaces, and functions
   - Special support for Drupal modules
   - Identifies Drupal hooks and module structures
   - Namespace and annotation analysis

5. **Go** (newly added)
   - Detects packages, structs, interfaces, and functions
   - Go module dependency analysis
   - Method receiver detection
   - Package organization support

## Project Type Detection
The application can detect these project types:

1. **Rust Projects**
   - Identified by Cargo.toml files
   - Analysis of dependencies and features
   - Detection of libraries vs binaries

2. **Python Projects**
   - Identified by pyproject.toml, setup.py, or requirements.txt
   - Detection of Django, Flask, and FastAPI frameworks
   - Virtual environment analysis

3. **JavaScript/TypeScript Projects**
   - Identified by package.json, tsconfig.json files
   - Detection of frontend frameworks and libraries

4. **React Applications**
   - Identified by React dependencies in package.json
   - JSX/TSX file analysis
   - Component and hook detection
   - Redux state management support
   - Next.js framework detection

5. **Angular Applications**
   - Identified by angular.json and Angular dependencies
   - Component, service, and module analysis
   - NgRx state management support

6. **PHP Projects**
   - Identified by composer.json or .php files
   - Namespace and class hierarchy analysis

7. **Drupal Modules/Sites**
   - Identified by .info.yml files with "type: module"
   - PHP files with Drupal-specific patterns (hooks, namespaces)
   - Enhanced search relevance for Drupal keywords

8. **Go Projects**
   - Identified by go.mod files
   - Package structure analysis
   - Module dependency tracking

9. **Generic Projects**
   - Default for unrecognized project types

## Important Notes
- When adding new language support, follow the pattern in `parser.rs` by adding a dedicated analyzer function
- Project type detection in `structure.rs` now uses the ProjectFeatures struct for efficient detection
- Search relevance in `search.rs` uses language-specific signatures for more accurate results
- Context generation in `context.rs` includes project type information with specific helper methods
- The project now has a strong focus on framework detection and specialized analysis

## Performance Improvements
- Search algorithm now stores relevance scores during initial scan to avoid repeated file reading
- Project detection uses a two-phase approach with feature detection first, then specific analysis
- Code parser uses more efficient line-by-line analysis with lookahead for nested structures
- Context gathering is now separated by language to avoid unnecessary processing

## Search Relevance Enhancements
- Language signature detection for more accurate file type identification 
- Keyword boosting based on file content and search context
- Framework-specific relevance scoring (React, Angular, Drupal, etc.)
- Multi-pass search algorithm with incremental relevance adjustment

## Using the File Editor
For file editing operations, use one of these formats:

1. **Complete file replacement:**
```json
{
  "action": "edit_file",
  "details": {
    "file_path": "/path/to/file.ext",
    "content": "Entire new content of the file"
  }
}
```

2. **Append to file:**
```json
{
  "action": "edit_file",
  "details": {
    "file_path": "/path/to/file.ext",
    "append": "Content to add at the end of the file"
  }
}
```

3. **Replace specific lines:**
```json
{
  "action": "edit_file",
  "details": {
    "file_path": "/path/to/file.ext",
    "edit_type": "replace",
    "start_line": 10,
    "end_line": 15,
    "new_text": "Text to replace the specified lines"
  }
}
```

4. **Insert at specific line:**
```json
{
  "action": "edit_file",
  "details": {
    "file_path": "/path/to/file.ext",
    "edit_type": "insert",
    "line": 10,
    "text": "Text to insert at the specified line"
  }
}
```

5. **Delete specific lines:**
```json
{
  "action": "edit_file",
  "details": {
    "file_path": "/path/to/file.ext",
    "edit_type": "delete",
    "start_line": 10,
    "end_line": 15
  }
}
```

6. **Simple content replacement (fallback):**
```json
{
  "action": "edit_file",
  "details": {
    "file_path": "/path/to/file.ext",
    "text": "Content for the file"
  }
}
```