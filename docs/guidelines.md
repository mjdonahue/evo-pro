# Development Guidelines for evo-pro

This document provides guidelines and instructions for developing and maintaining the evo-pro project.

## Build/Configuration Instructions

### Prerequisites

- [Node.js](https://nodejs.org/) (LTS version recommended)
- [PNPM](https://pnpm.io/) for package management
- [Rust](https://www.rust-lang.org/) and [Cargo](https://doc.rust-lang.org/cargo/) for the Tauri backend
- [SQLite](https://www.sqlite.org/) for database operations

### Setup

1. Clone the repository
2. Install dependencies:
   ```bash
   pnpm install
   ```
3. Copy the environment file and configure it:
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

### Development

To start the development server:

```bash
pnpm dev
```

This will start both the Vite development server for the frontend and the Tauri development process.

### Building for Production

To build the application for production:

```bash
pnpm build
```

This will:
1. Compile TypeScript
2. Build the Vite frontend
3. Build the Tauri application

### Tauri-specific Commands

To run Tauri-specific commands:

```bash
pnpm tauri [command]
```

Common commands include:
- `pnpm tauri dev` - Start the development environment
- `pnpm tauri build` - Build the application for production

## Testing Information

### Frontend Testing (JavaScript/TypeScript)

The project uses Vitest for testing the frontend code.

#### Running Tests

To run all tests once:

```bash
pnpm test
```

To run tests in watch mode during development:

```bash
pnpm test:watch
```

#### Writing Tests

Tests are located in the `src/__tests__` directory. The project uses:
- Vitest as the test runner
- React Testing Library for testing React components
- Jest DOM for DOM-specific assertions

Example of a simple test:

```typescript
// src/__tests__/example.test.ts
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import MyComponent from '../components/MyComponent';

describe('MyComponent', () => {
  it('renders correctly', () => {
    render(<MyComponent />);
    expect(screen.getByText('Hello World')).toBeInTheDocument();
  });
});
```

### Backend Testing (Rust)

The Rust backend uses Rust's built-in testing framework with Tokio for async testing.

#### Running Tests

To run the Rust tests:

```bash
cd src-tauri
cargo test
```

For running specific tests:

```bash
cargo test test_name
```

#### Writing Tests

Rust tests are located in the `src-tauri/src/tests` directory. The project uses:
- Rust's built-in `#[test]` attribute for synchronous tests
- `#[tokio::test]` attribute for asynchronous tests

Example of a simple Rust test:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_function() {
        assert_eq!(add(1, 2), 3);
    }

    #[tokio::test]
    async fn test_async_function() -> Result<()> {
        let result = async_function().await?;
        assert_eq!(result, expected_value);
        Ok(())
    }
}
```

## Code Style and Development Practices

### TypeScript/JavaScript

- The project uses TypeScript for type safety
- ESLint is configured for code linting with the following extensions:
  - eslint:recommended
  - @typescript-eslint/recommended
  - plugin:react-hooks/recommended
- Use functional components with hooks for React components
- Follow the React hooks rules (enforced by ESLint)

### Rust

- Follow the standard Rust code style (enforced by rustfmt)
- Use the actor model pattern with Kameo for concurrent operations
- Implement proper error handling using Result types
- Use async/await for asynchronous operations with Tokio

### Database

- The project uses SQLx with SQLite for database operations
- Database migrations are located in the `src-tauri/migrations` directory
- Seed data for development is in the `src-tauri/seeders` directory

### Architecture

- The application follows a client-server architecture within a single desktop application:
  - Frontend: React, TypeScript, Vite
  - Backend: Rust, Tauri
- Communication between frontend and backend is handled through Tauri's API
- The application uses a peer-to-peer networking model with libp2p

## Additional Development Information

### Debugging

- For frontend debugging, use the browser's developer tools
- For backend debugging, use logging with the `tracing` crate
- Tauri logs can be accessed through the Tauri developer tools

### Performance Considerations

- The application is optimized for desktop use with a focus on local-first operations
- Heavy operations should be offloaded to the Rust backend
- Use React's performance optimization techniques (memoization, virtualization) for large lists or complex UI

### Security

- All user data is stored locally and encrypted
- Communication between peers is encrypted using libp2p's security protocols
- Follow the principle of least privilege when implementing new features