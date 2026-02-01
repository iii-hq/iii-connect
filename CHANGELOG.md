# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-02-01

### Added

- Initial release of iii-mcp
- Full MCP protocol implementation (initialize, tools/list, tools/call, resources/list, resources/read)
- CLI with `--engine-url` and `--debug` flags
- State operations tools: `state_set`, `state_get`, `state_delete`, `state_update`, `state_list`
- Event emission tool: `emit`
- Engine introspection tools: `engine_functions_list`, `engine_workers_list`, `engine_triggers_list`
- Dynamic tool registration for custom worker functions
- Resources: `iii://functions`, `iii://workers`, `iii://triggers`
- Structured logging to stderr
- Claude Desktop and Cursor configuration examples
- Architecture documentation

### Compatibility

| iii-mcp | iii-engine |
|---------|------------|
| 0.1.x   | 0.3.x      |
