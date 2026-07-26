//! phi-tools: General-purpose Agent toolset
//!
//! Each tool independently implements the agent_base::Tool trait.
//! Consumers register tools with AgentBuilder on demand.
//!
//! # Example
//!
//! ```ignore
//! use phi_tools::LocalShellTool;
//!
//! builder.register_tool(LocalShellTool::new(30_000));
//! ```

pub mod local_shell;

pub use local_shell::LocalShellTool;
