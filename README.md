# Evo Pro

This is a repo for the Evo AI Assistant - a revolutionary local-first AI system that provides intelligent, privacy-preserving assistance across multiple devices and contexts. Unlike traditional cloud-dependent AI solutions, Evo prioritizes on-device processing with selective cloud augmentation, creating a system that learns and adapts to users over time while keeping their data secure and private.

## Platform Overview

Evo represents the next generation of AI assistance, built on three foundational principles:

- **Privacy by Design**: Your data stays on your devices by default
- **Adaptive Intelligence**: Learning and evolving with you over time  
- **Seamless Integration**: Working across your digital ecosystem without friction

### Advanced Memory System

Evo incorporates a human-inspired memory model with multiple layers:

1. **Working Memory**: Maintains immediate context awareness during interactions
2. **Episodic Memory**: Records and references specific events and conversations
3. **Semantic Memory**: Builds a knowledge graph of concepts and relationships
4. **Procedural Memory**: Learns how to perform tasks through observation and instruction
5. **Intelligent Memory Management**: Implements priority-based memory retention and contextual decay


## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)



## Build from source

1. Install Node.js
2. Install pnpm
3. Install Rust
4. Run the following commands:

```bash
pnpm install
pnpm tauri build
```

## Development

### Quick Setup

We provide setup scripts to automate the installation of dependencies and configuration of the development environment:

#### On macOS/Linux:

```bash
# Make the script executable
chmod +x scripts/setup.sh

# Run the setup script
./scripts/setup.sh
```

#### On Windows:

```powershell
# Run the setup script in PowerShell
.\scripts\setup.ps1
```

The setup scripts will:
- Check for and install required dependencies (Node.js, pnpm, Rust)
- Install cargo-watch for hot reloading
- Install project dependencies
- Set up git hooks
- Check for SQLite

### Manual Setup

If you prefer to set up manually:

1. Install Node.js
2. Install pnpm
3. Install Rust
4. Run the following commands:

```bash
pnpm install
pnpm tauri dev
```

### Hot Reloading

For a better development experience with automatic reloading when files change:

```bash
# Install cargo-watch if you haven't already
cargo install cargo-watch

# Run with hot reloading for both frontend and backend
pnpm dev:hot
```

This will:
- Automatically reload the frontend when you change React/TypeScript files
- Automatically rebuild and reload the Rust backend when you change Rust files

## Build and Test Processes

Evo Design includes a comprehensive set of scripts for building, testing, and maintaining code quality:

### Testing

```bash
# Run all tests
pnpm test

# Run tests in watch mode
pnpm test:watch

# Run tests with coverage report
pnpm test:coverage

# Run Rust tests
pnpm rust:test
```

### Linting and Formatting

```bash
# Lint TypeScript/JavaScript code
pnpm lint

# Fix linting issues
pnpm lint:fix

# Format code with Prettier
pnpm format

# Check if code is properly formatted
pnpm format:check

# Check Rust code
pnpm rust:check

# Lint Rust code with Clippy
pnpm rust:clippy

# Format Rust code
pnpm rust:format
```

### Code Quality Tools

```bash
# Analyze ESLint rules and check for warnings
pnpm analyze

# Check code complexity in TypeScript files
pnpm complexity

# Run all code quality checks
pnpm quality
```

### Combined Scripts

```bash
# Check all code (lint, format, Rust)
pnpm check

# Fix all code issues (lint, format, Rust)
pnpm fix
```

### Automated Code Formatting

The project uses Husky and lint-staged to automatically format and lint code before commits:

- All JavaScript/TypeScript files are linted with ESLint and formatted with Prettier
- All JSON, CSS, and Markdown files are formatted with Prettier
- All Rust files are formatted with rustfmt

This ensures consistent code style throughout the project. The pre-commit hooks are installed automatically when you run `pnpm install`.

To manually set up the git hooks after cloning the repository:

```bash
pnpm prepare
```

## Core Architecture

### Local-First Processing Engine

Evo utilizes a sophisticated processing architecture that prioritizes on-device computation:

- Near-instant response times with minimal latency
- Operation during internet outages or low-connectivity environments
- Protection of sensitive information through local data processing
- Reduced cloud computing costs and carbon footprint

### Hybrid Cognitive Architecture

The system intelligently balances processing between:

- **Local Models**: Lightweight, efficient neural networks running entirely on-device
- **Edge Computing**: Distributing complex tasks across your personal devices
- **Cloud Augmentation**: Selectively leveraging cloud resources for specialized tasks

### Core Agent Ecosystem

Evo deploys specialized agents working in concert:

- **Contacts Agent**: Maintains relationships and interaction history
- **Task Agent**: Manages to-dos and priorities
- **Calendar Agent**: Coordinates scheduling and prevents conflicts
- **Documents Agent**: Organizes and retrieves content
- **Communication Agent**: Handles messages with context awareness
- **Learning Agent**: Continuously improves understanding of user preferences

## Enterprise Features

### Flexcare Healthcare Staffing

Evo's first enterprise deployment is tailored for healthcare staffing with specialized features:

- **Recruiter Productivity Suite**: Candidate relationship management and automated follow-ups
- **Account Management**: Facility relationship dashboard and staffing analytics
- **Compliance Platform**: Credential lifecycle management and regulatory monitoring
- **Clinician Experience**: Personalized assignment matching and credential portfolio management


## Contributing

Please refer to our [Developer Onboarding Guide](docs/developer-onboarding.md) for detailed information about:

- Development workflow and branch strategy
- Code style guidelines
- Testing procedures
- Deployment processes

## Documentation

- [High-Level Overview](docs/evo-high-level-overview.md)
- [Product Overview](docs/product-overview.md)
- [Requirements](docs/requirements.md)
- [Developer Onboarding](docs/developer-onboarding.md)
- [Architecture Documentation](docs/architecture/)

## License

This project is proprietary software. All rights reserved.
