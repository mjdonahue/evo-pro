# PowerShell script for setting up Evo Design development environment on Windows

# Function to check if a command exists
function Test-Command {
    param (
        [string]$Command
    )
    
    $exists = $null -ne (Get-Command $Command -ErrorAction SilentlyContinue)
    return $exists
}

Write-Host "Setting up Evo Design development environment..." -ForegroundColor Green

# Check if Node.js is installed
if (-not (Test-Command node)) {
    Write-Host "Node.js is not installed. Please install Node.js LTS version." -ForegroundColor Red
    Write-Host "Visit https://nodejs.org/ to download and install." -ForegroundColor Yellow
    exit 1
}

# Check if pnpm is installed
if (-not (Test-Command pnpm)) {
    Write-Host "pnpm is not installed. Installing pnpm..." -ForegroundColor Yellow
    npm install -g pnpm
}

# Check if Rust is installed
if (-not (Test-Command rustc)) {
    Write-Host "Rust is not installed. Installing Rust..." -ForegroundColor Yellow
    Invoke-WebRequest -Uri https://win.rustup.rs/x86_64 -OutFile rustup-init.exe
    .\rustup-init.exe -y
    Remove-Item rustup-init.exe
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
}

# Check if cargo-watch is installed (for hot reloading)
if (-not (Test-Command cargo-watch)) {
    Write-Host "cargo-watch is not installed. Installing cargo-watch..." -ForegroundColor Yellow
    cargo install cargo-watch
}

# Install dependencies
Write-Host "Installing project dependencies..." -ForegroundColor Green
pnpm install

# Set up git hooks
Write-Host "Setting up git hooks..." -ForegroundColor Green
pnpm prepare

# Check if SQLite is installed
if (-not (Test-Command sqlite3)) {
    Write-Host "SQLite is not installed. Some features may not work correctly." -ForegroundColor Yellow
    Write-Host "Please install SQLite manually for Windows." -ForegroundColor Yellow
}

Write-Host "Development environment setup complete!" -ForegroundColor Green
Write-Host "You can now run 'pnpm dev' to start the development server." -ForegroundColor Green