# Hot Reloading for Frontend and Backend

This document explains how hot reloading works in the evo-pro project for both frontend and backend components.

## Overview

Hot reloading allows developers to see changes in real-time without manually restarting the application. The evo-pro project supports hot reloading for both:

1. **Frontend (React/TypeScript)**: Using Vite's built-in Hot Module Replacement (HMR)
2. **Backend (Rust/Tauri)**: Using cargo-watch to monitor file changes and rebuild automatically

## How It Works

### Frontend Hot Reloading

Vite provides built-in hot module replacement for React components. When you modify a React component or any frontend code, Vite automatically updates the browser without a full page reload, preserving the application state.

### Backend Hot Reloading

For the Rust backend, we use `cargo-watch` to monitor changes in the `src-tauri/src` directory. When a file changes, cargo-watch automatically rebuilds and restarts the Tauri application with the `dev` feature enabled.

## Usage

Hot reloading is enabled by default when running the development server:

```bash
pnpm dev
```

This command runs both frontend and backend hot reloading in parallel.

### Alternative Commands

- `pnpm dev:legacy`: Runs only the frontend with hot reloading (old behavior)
- `pnpm dev:hot`: Alias for the current default behavior
- `pnpm dev:hot:frontend`: Runs only the frontend with hot reloading
- `pnpm dev:hot:backend`: Runs only the backend with hot reloading

## Requirements

To use backend hot reloading, you need to have `cargo-watch` installed:

```bash
cargo install cargo-watch
```

## Troubleshooting

If hot reloading isn't working as expected:

1. Make sure `cargo-watch` is installed for backend hot reloading
2. Check that the `dev` feature is properly defined in `src-tauri/Cargo.toml`
3. Ensure your changes are being saved to disk
4. For backend changes, check the terminal for any compilation errors