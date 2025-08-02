# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Claude Workbench is a comprehensive desktop GUI application for Claude CLI, built with Tauri (Rust backend) and React TypeScript frontend. It provides an intuitive interface for AI-powered development workflows, project management, session handling, and MCP (Model Context Protocol) server management.

The application supports multiple platforms:
- **Windows** (primary target) - Full native support with Windows-specific optimizations
- **macOS** - Cross-platform compatible with automatic builds via GitHub Actions
- **Linux** - Cross-platform compatible with automatic builds via GitHub Actions

The application focuses on core Claude CLI integration optimized for Windows users, providing a streamlined experience for developers working with AI assistance on Windows platforms.

## Development Commands

### Prerequisites
- **Bun** (recommended) or npm for package management
- **Rust** 2021 edition with Tauri CLI
- **Node.js** 18+ (for development)
- **Platform-specific dependencies**:
  - **Windows**: Microsoft C++ Build Tools, WebView2
  - **macOS**: Xcode and development tools
  - **Linux**: libgtk, webkit2gtk, and other system dependencies

### Frontend Development
```bash
# Start Vite development server
bun run dev

# Build frontend only
bun run build

# Type checking
npx tsc --noEmit

# Preview built frontend
bun run preview
```

### Tauri/Desktop Development
```bash
# Start full Tauri development (includes frontend hot reload)
bun run tauri dev

# Build complete desktop application
bun run tauri build

# Fast development build (optimized for iteration)
bun run tauri:build-fast

# Build Rust backend only
cd src-tauri && cargo build --release
```

### Critical Build Requirements
- **ALWAYS use `bun run tauri build` for production/testing** - Development mode can mask compatibility issues
- **Bun is required for cross-platform compatible builds** - npm builds may cause issues on some systems
- **GitHub Actions for cross-platform builds** - Mac and Linux versions built automatically via CI/CD
- **Release builds use aggressive size optimizations** - opt-level="z", LTO enabled, symbols stripped

## Core Architecture

### Application Structure
The codebase follows a clear separation between frontend UI and backend system integration:

```
Frontend (React/TypeScript)
├── App.tsx - Main view router and application state
├── components/ - UI components organized by feature
│   ├── ClaudeCodeSession.tsx - Core Claude interaction interface
│   ├── FloatingPromptInput.tsx - Universal prompt input with context awareness
│   ├── Settings.tsx - Multi-tab configuration interface
│   └── [50+ specialized components]
├── lib/ - Business logic and API layer
│   ├── api.ts - Type-safe Tauri invoke interface
│   └── contextManager.ts - Automatic context length management
└── hooks/ - Custom React hooks for complex state

Backend (Rust/Tauri)
├── main.rs - Application entry and command registration
├── commands/ - Modular command handlers
│   ├── claude.rs - Claude CLI integration and process management
│   ├── agents.rs - Agent execution with GitHub integration
│   ├── mcp.rs - MCP server lifecycle management
│   ├── storage.rs - SQLite database operations
│   └── [additional specialized modules]
└── process/ - Process registry and lifecycle management
```

### Data Flow Patterns
1. **Frontend → Backend**: Type-safe Tauri `invoke()` calls with comprehensive error handling
2. **Backend → CLI**: Process spawning with streaming output capture and health monitoring
3. **Database**: SQLite with automatic migrations and optimized queries
4. **File System**: Scoped access with security policies (`$HOME/**`, `$TEMP/**`, `$TMP/**`)
5. **Event System**: Tauri events for real-time communication (claude-output, claude-complete, etc.)

### Key Architectural Decisions

#### Context Management System
- **Automatic Detection**: Real-time token usage monitoring across session messages
- **Smart Compression**: Auto-triggers `/compact` command when approaching 200K token limit (configurable)
- **Anti-Loop Protection**: Cooldown periods and error detection prevent infinite compression loops
- **Session Isolation**: Different sessions tracked independently with persistent user preferences

#### Process Management
- **Windows-Optimized**: Uses Windows-specific process creation flags (`CREATE_NO_WINDOW`)
- **Session Isolation**: Each Claude session runs in isolated process with unique session IDs
- **Stream Handling**: Real-time JSONL parsing with proper error boundaries
- **Lifecycle Tracking**: Comprehensive process registry for cleanup and monitoring

#### Provider Management (Core Feature)
- **Silent Switching**: One-click provider switching without popups or interruptions
- **Local Storage**: All configurations stored locally with zero hardcoded credentials
- **Immediate Effect**: Automatic Claude process restart when provider changes
- **Smart Detection**: Auto-detects and displays current active provider configuration

## Component Architecture

### Critical Components

#### ClaudeCodeSession (`src/components/ClaudeCodeSession.tsx`)
- **Purpose**: Main Claude interaction interface with session management
- **Key Features**: Session resumption, checkpoint management, real-time streaming, context awareness
- **Event Handling**: Listens for `claude-output:${sessionId}`, `claude-complete:${sessionId}`, `claude-error:${sessionId}`
- **Context Integration**: Provides conversation context to FloatingPromptInput for enhanced prompts

#### FloatingPromptInput (`src/components/FloatingPromptInput.tsx`)
- **Purpose**: Universal prompt input with advanced features
- **Key Features**: Model selection, thinking modes, file picker (@mention), slash commands, context-aware prompt enhancement
- **Image Support**: Drag-drop, clipboard paste, file references with automatic path handling
- **Context Awareness**: Accepts `getConversationContext` callback for intelligent prompt enhancement

#### Settings (`src/components/Settings.tsx`)
- **Purpose**: Multi-tab configuration interface
- **Tabs**: General, Providers, Hooks, Storage, Advanced
- **Provider Management**: Full CRUD operations for API provider configurations
- **Integration**: Real-time application of settings changes

### Backend Command Modules

#### claude.rs - Core Claude Integration
- **Session Management**: `execute_claude_code`, `continue_claude_code`, `resume_claude_code`
- **Process Lifecycle**: Process spawning, monitoring, cleanup with Windows optimizations
- **Context Enhancement**: `enhance_prompt` with conversation context analysis
- **Checkpoint System**: Automatic and manual checkpoint creation with timeline navigation

#### agents.rs - Agent Execution System
- **GitHub Integration**: Fetch and import agents from GitHub repositories
- **Process Isolation**: Each agent runs in separate tracked process
- **Real-time Monitoring**: Live session output and execution metrics
- **Database Management**: SQLite-based agent storage and run history

#### mcp.rs - MCP Server Management
- **Server Lifecycle**: Start, stop, monitor MCP servers
- **Configuration Management**: Project-specific and global MCP configurations
- **Health Monitoring**: Connection testing and status tracking
- **Integration**: Seamless integration with Claude CLI MCP support

## Development Patterns

### Error Handling Strategy
- **Rust Backend**: Comprehensive `Result<T, String>` pattern with detailed error messages
- **TypeScript Frontend**: Type-safe error handling with user-friendly alerts and recovery mechanisms
- **Process Errors**: Graceful handling of Claude CLI failures with fallback strategies

### State Management
- **Global State**: React Context providers for theme, i18n, and cross-component data
- **Local State**: Component-specific state with custom hooks for complex logic
- **Persistent State**: SQLite database for long-term data storage
- **Real-time State**: Tauri events for live updates between frontend and backend

### Security Considerations
- **File System Access**: Strictly scoped to allowed directories with validation
- **Process Execution**: Sandboxed process spawning with argument sanitization
- **Credential Storage**: Local-only storage with no hardcoded secrets
- **CSP Policy**: Restrictive Content Security Policy for web content

## Debugging and Development

### Logging and Diagnostics
- **Rust Logging**: Comprehensive `log` crate usage with structured messages including session IDs and operation contexts
- **Frontend Debugging**: Console logging with operation tracking and error boundaries
- **Process Monitoring**: Built-in process registry for tracking Claude sessions and agent executions

### Common Development Patterns
- **Type Safety**: Strict TypeScript configuration with comprehensive type definitions
- **Component Communication**: Props-based communication with callback patterns for parent-child interaction
- **Async Operations**: Proper async/await handling with loading states and error boundaries
- **Resource Cleanup**: Automatic cleanup of listeners, processes, and temporary files

### Performance Considerations
- **Memory Management**: Circular buffering for streaming operations to prevent memory leaks
- **Database Optimization**: Indexed queries and prepared statements for frequently accessed data
- **Build Optimization**: Aggressive size optimization for release builds with LTO and symbol stripping
- **UI Performance**: Virtual scrolling for large lists and efficient re-rendering strategies

## Cross-Platform Support

### Platform Compatibility
- **Windows** (Primary): Full native support with platform-specific optimizations
  - Console window hiding via `CREATE_NO_WINDOW` flags
  - Windows path normalization and UNC path support
  - Registry integration capabilities
  - Windows-specific environment variable handling

- **macOS**: Cross-platform compatible via conditional compilation
  - All Windows-specific code wrapped in `#[cfg(target_os = "windows")]`
  - Mac-specific build targets: `.dmg` and `.app` formats
  - Automatic builds via GitHub Actions

- **Linux**: Cross-platform compatible via conditional compilation
  - System dependency requirements handled in CI
  - Linux-specific build optimizations
  - Automatic builds via GitHub Actions

### Build Strategy
- **Local Development**: Primary development on Windows platform
- **Cross-Platform Builds**: Automated via GitHub Actions CI/CD
- **Conditional Compilation**: Platform-specific code isolated with Rust cfg attributes
- **Zero Cross-Platform Impact**: All changes maintain full Windows functionality

### GitHub Actions Workflow
The repository includes automated multi-platform builds that:
1. Build for Windows (`.msi`, `.exe`)
2. Build for macOS (`.dmg`, `.app`)
3. Build for Linux (`.deb`, `.AppImage`)
4. Create draft releases with all platform artifacts
5. Support both release tags and development builds

## Windows-Specific Optimizations

### Process Management
- **Console Window Hiding**: `CREATE_NO_WINDOW` flag for all subprocess operations
- **Path Handling**: Proper Windows path normalization and UNC path support
- **Environment Variables**: Windows-specific environment variable handling for Claude CLI

### File System Integration
- **Scoped Access**: Proper handling of Windows file permissions and directory structures
- **Temporary Files**: Windows-compatible temporary file creation and cleanup
- **Registry Integration**: Potential for Windows registry integration (not currently implemented)

This architecture provides a robust foundation for Claude CLI desktop integration while maintaining type safety, performance, and user experience standards across all supported platforms, with Windows remaining the primary development and optimization target.