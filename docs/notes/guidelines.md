# Evo Apps Development Guidelines

This document provides essential information for developers working on the Evo Apps monorepo.

## Build/Configuration Instructions

### Monorepo Structure

The Evo Apps monorepo contains multiple applications and shared packages:

- **Apps**:
  - `evo-desktop`: Desktop app built with Tauri
  - `evo-mobile`: Mobile app built with React Native
  - `evo-pro`: Pro app built with React/TypeScript
  - `evo-web`: Web app built with React/TypeScript
  - `evo-server`: Server app built with Node/Express

- **Packages**:
  - `api-interfaces`: API interface definitions
  - `common`: Shared utilities and functions
  - `core`: Core functionality
  - `db`: Database-related code (using Prisma)
  - `mastra`: Custom library
  - `types`: TypeScript type definitions
  - `ui`: Shared UI components

### Prerequisites

- `pnpm`: Package manager (https://pnpm.io/)
- `rust`: For Tauri apps (https://www.rust-lang.org/)
- `cargo`: Rust package manager (https://doc.rust-lang.org/cargo/)
- `tauri`: For desktop apps (https://tauri.app/)
- `expo`: For mobile apps (https://docs.expo.dev/)

### Installation

```bash
pnpm install
```

### Development

To start development servers for all apps:

```bash
pnpm dev
```

To start development for specific apps:

```bash
# Desktop app
pnpm dev:desktop
# or
pnpm tauri:dev

# Mobile app
pnpm dev:mobile
# or
pnpm expo:start

# Server
pnpm dev:server

# Pro app
pnpm dev:evopro
# or
pnpm tauri:dev:evopro
```

### Building

To build all apps:

```bash
pnpm build
```

To build specific apps:

```bash
# Desktop app
pnpm tauri:build

# Pro app
pnpm tauri:build:evopro
```

## Testing Information

### Testing Framework

The project uses Jest for testing. Test files are typically located in `__test__` directories or named with the `.test.ts` or `.test.tsx` extension.

### Running Tests

To run all tests in the monorepo:

```bash
pnpm test
```

### Writing Tests

Tests should be placed in a `__test__` directory adjacent to the code being tested or named with the `.test.ts` extension. Here's an example of a simple test:

```typescript
// Example test for a utility function
import { myFunction } from '../utils';

test('myFunction returns expected result', () => {
  const input = 'test';
  const expected = 'TEST';
  expect(myFunction(input)).toBe(expected);
});
```

### Test Example

Here's a simple test that demonstrates testing a utility function:

```typescript
// utils.ts
export function capitalize(str: string): string {
  return str.charAt(0).toUpperCase() + str.slice(1);
}

// utils.test.ts
import { capitalize } from './utils';

test('capitalize function capitalizes the first letter', () => {
  expect(capitalize('hello')).toBe('Hello');
  expect(capitalize('world')).toBe('World');
  expect(capitalize('')).toBe('');
});
```

## Code Style and Development Practices

### Code Style

The project uses ESLint and Prettier for code linting and formatting. The configuration extends Airbnb's style guide with some customizations:

- Single quotes for strings
- ES5 trailing commas
- Strict import sorting
- TypeScript-specific rules

### Linting and Formatting

To lint all code:

```bash
pnpm lint
```

To format all code:

```bash
pnpm format
```

### Key ESLint Rules

- Imports are sorted using `simple-import-sort`
- Unused imports are warned about using `unused-imports`
- React Hooks rules are enforced
- TypeScript consistent type imports are required

### Monorepo Workflow

- Use `pnpm` for all package management operations
- Workspace packages are referenced using the `workspace:*` syntax in package.json
- The monorepo uses TypeScript project references for better type checking across packages

### Debugging

- For Tauri apps, logs are available through the Tauri logger
- For React apps, use the React Developer Tools browser extension
- For server apps, standard Node.js debugging techniques apply

### Performance Considerations

- Be mindful of bundle sizes, especially for mobile and web apps
- Use React's performance optimization techniques (memoization, virtualization, etc.)
- Consider code splitting for larger applications