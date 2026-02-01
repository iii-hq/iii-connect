# Contributing to iii-mcp

Thank you for your interest in contributing to iii-mcp! This document provides guidelines to help you contribute effectively.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Architecture Overview](#architecture-overview)
- [Development Workflow](#development-workflow)
- [Testing](#testing)
- [Pull Request Process](#pull-request-process)
- [Reporting Issues](#reporting-issues)

## Code of Conduct

We are committed to providing a welcoming and inclusive experience for everyone. Please be respectful and constructive in all interactions.

## Getting Started

### Prerequisites

- **Rust 1.75+** - Install via [rustup](https://rustup.rs/)
- **iii-engine** - A running iii-engine instance for testing

### Clone the Repository

```bash
git clone https://github.com/MotiaDev/iii-mcp.git
cd iii-mcp
```

## Development Setup

### Install Dependencies

```bash
# Install Rust toolchain components
rustup component add rustfmt clippy
```

### Build the Project

```bash
# Debug build
cargo build

# Release build
cargo build --release
```

### Run iii-mcp

```bash
# Connect to local iii-engine
cargo run -- --engine-url ws://localhost:8080

# With debug logging
cargo run -- --engine-url ws://localhost:8080 --debug
```

## Architecture Overview

```
iii-mcp/
├── src/
│   ├── main.rs             # CLI entry point
│   ├── lib.rs              # Library exports
│   ├── server.rs           # MCP server + request routing
│   ├── json_rpc.rs         # JSON-RPC 2.0 types (self-contained)
│   ├── handlers/
│   │   ├── mod.rs          # Handler exports
│   │   ├── initialize.rs   # MCP initialization & capabilities
│   │   ├── tools.rs        # tools/list, tools/call handlers
│   │   └── resources.rs    # resources/list, resources/read handlers
│   └── transport/
│       ├── mod.rs          # Transport exports
│       └── stdio.rs        # stdio transport for MCP clients
```

### Key Components

| Component | File | Purpose |
|-----------|------|---------|
| JSON-RPC Handler | `json_rpc.rs` | Self-contained JSON-RPC 2.0 message types |
| MCP Server | `server.rs` | Routes MCP requests to handlers |
| Tools Handler | `handlers/tools.rs` | Converts iii-engine functions to MCP tools |
| Resources Handler | `handlers/resources.rs` | Exposes iii-engine data as MCP resources |
| stdio Transport | `transport/stdio.rs` | Reads/writes JSON-RPC over stdin/stdout |

### Adding New Features

#### Adding a New MCP Method

1. Add the handler method in the appropriate handler file
2. Add routing in `server.rs`:
   ```rust
   "your/method" => Ok(self.your_handler.method(request.params).await)
   ```

#### Adding a New Resource

1. Add the resource definition in `handlers/resources.rs`:
   ```rust
   McpResource {
       uri: "iii://your-resource".to_string(),
       name: "Your Resource".to_string(),
       description: Some("Description".to_string()),
       mime_type: Some("application/json".to_string()),
   }
   ```
2. Add the read handler in the `read` method

## Development Workflow

### Creating a Feature Branch

```bash
git checkout main
git pull origin main
git checkout -b feature/your-feature-name
```

### Code Style

We follow standard Rust conventions:

```bash
# Format code
cargo fmt

# Run linter
cargo clippy --all-targets -- -D warnings
```

### Commit Messages

Use clear, descriptive commit messages:

```
feat: add prompts/list handler for MCP prompts
fix: handle connection timeout gracefully
docs: update README with new configuration options
```

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

### Manual Testing with MCP Inspector

```bash
# Build release
cargo build --release

# Test with MCP Inspector
npx @anthropic/mcp-inspector ./target/release/iii-mcp --engine-url ws://localhost:8080
```

### Testing with Claude Desktop

1. Build the release binary
2. Add to Claude Desktop config:
   ```json
   {
     "mcpServers": {
       "iii-test": {
         "command": "/path/to/target/release/iii-mcp",
         "args": ["--engine-url", "ws://localhost:8080", "--debug"]
       }
     }
   }
   ```
3. Restart Claude Desktop and test tool invocations

## Pull Request Process

### Before Submitting

1. **Ensure tests pass**: `cargo test`
2. **Check formatting**: `cargo fmt --check`
3. **Run linter**: `cargo clippy --all-targets -- -D warnings`
4. **Update documentation** if needed

### PR Guidelines

- **One feature per PR** - Keep PRs focused
- **Describe your changes** - Explain what and why
- **Test your changes** - Include test results or screenshots
- **Be responsive** - Address review feedback promptly

## Reporting Issues

### Bug Reports

Include:
- **iii-mcp version**: `iii-mcp --version`
- **iii-engine version**: The engine version you're connecting to
- **Operating system**: e.g., macOS 14, Ubuntu 22.04
- **Steps to reproduce**: Minimal example
- **Expected vs actual behavior**
- **Logs**: Run with `--debug` flag

### Feature Requests

Include:
- **Use case**: What problem does this solve?
- **Proposed solution**: How should it work?
- **MCP specification reference**: Link to relevant MCP docs if applicable

## Questions?

- Open a [GitHub Issue](https://github.com/MotiaDev/iii-mcp/issues)
- Check [MCP Documentation](https://modelcontextprotocol.io/)
- See [iii-engine docs](https://github.com/MotiaDev/iii-engine)

Thank you for contributing to iii-mcp!
