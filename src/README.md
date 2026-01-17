# Enhanced Directory Structure

This directory demonstrates the new modular architecture where each agent and tool has its own dedicated directory with:

- `mod.rs` - Module definition and exports
- `agent.rs`/`tool_name.rs` - Main implementation
- `config.toml` - Configuration file
- `tests.rs` - Unit and integration tests
- Additional files as needed (prompts, utilities, etc.)

## Adding New Agents

1. Create new directory in `src/agents/builtin/your_agent/`
2. Follow the Desktop Automation Agent pattern:
   - `mod.rs` - Exports and metadata
   - `agent.rs` - Agent implementation
   - `config.toml` - Agent configuration
   - `tests.rs` - Comprehensive tests
   - `prompt.txt` - System prompt template

## Adding New Tools

1. Create new directory in `src/tools/builtin/category/tool_name/`
2. Follow the mouse tool pattern:
   - `mod.rs` - Module exports
   - `tool_name.rs` - Tool implementation
   - `tests.rs` - Tool tests
   - `config.toml` - Tool parameters and metadata

## Benefits

✅ **Clear Organization** - Each component is self-contained
✅ **Easy Testing** - Dedicated test files for each component
✅ **Modular Design** - Components can be developed independently
✅ **Configuration Management** - TOML configs for easy customization
✅ **Extensibility** - New components follow established patterns
✅ **Maintainability** - Related code is grouped together

## Next Steps

1. Review the example structure
2. Adapt to your specific needs
3. Implement core traits and interfaces
4. Set up the component loader
5. Create build system to auto-discover components