#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

echo "🚀 Starting Axiom OS Full Build..."
echo "===================================="

echo "🔍 Checking Prerequisites..."

# Check npm
if ! command -v npm &> /dev/null; then
    echo "❌ npm could not be found. Please install Node.js (https://nodejs.org) to build the CCP Frontend."
    exit 1
fi
echo "✅ npm is installed."

# Check rustup/cargo
if ! command -v cargo &> /dev/null; then
    echo "⚠️ cargo could not be found. Installing rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "✅ cargo is installed."
fi

# Check wasm32-unknown-unknown target
echo "🔍 Checking for wasm32-unknown-unknown target..."
if ! rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
    echo "⚠️ wasm32-unknown-unknown target not found. Installing..."
    rustup target add wasm32-unknown-unknown
else
    echo "✅ wasm32-unknown-unknown target is installed."
fi

echo "===================================="


# Step 1: Build and Install the Toolchain (axiom-cli)
# Build Order: 1. It operates independently and is the developer's main entry point for managing Kernels.
echo ""
echo "🛠️  [1/4] Building Axiom CLI (ax toolchain)..."
cd axiom-cli
cargo build --release
cargo install --path .
cd ..
echo "✅ Axiom CLI installed globally."

# Install axiom-sdk to ~/.axiom/sdk so kernels outside this repo can find it
echo ""
echo "📦 [1.5/4] Installing Axiom SDK to ~/.axiom/sdk..."
mkdir -p "$HOME/.axiom/sdk"
rm -rf "$HOME/.axiom/sdk/axiom-sdk"
rm -rf "$HOME/.axiom/sdk/axiom-macros"
cp -r axiom-sdk "$HOME/.axiom/sdk/axiom-sdk"
cp -r axiom-sdk/axiom-macros "$HOME/.axiom/sdk/axiom-macros"
echo "✅ Axiom SDK installed to ~/.axiom/sdk/"

# Step 2: Build the Wasm Runtime (axiom-shell)
# Build Order: 2. The core execution engine. It doesn't strictly depend on CCP to run, but CCP dictates its state.
echo ""
echo "🐚 [2/4] Building Axiom Shell..."
cd axiom-shell
cargo build --release
cd ..
echo "✅ Axiom Shell compiled."

# Step 3: Build the CCP Backend (Rust/Axum)
# Build Order: 3. The nervous system that connects the CLI and the Shell across environments.
echo ""
echo "📡 [3/4] Building Axiom CCP Backend..."
cd axiom-ccp/axiom-ccp-backend
cargo build --release
cd ../..
echo "✅ CCP Backend compiled."

# Step 4: Build the CCP Frontend (React/Vite)
# Build Order: 4. The visual dashboard that relies on the Backend's APIs.
echo ""
echo "💻 [4/4] Building Axiom CCP Frontend..."
cd axiom-ccp/axiom-ccp-frontend
npm install
npm run build
cd ../..
echo "✅ CCP Frontend compiled."

echo ""
echo "===================================="
echo "🎉 Axiom OS fully built successfully!"
echo "Next steps:"
echo "1. Run Axiom CCP: cd axiom-ccp && ./dev.sh"
echo "2. Run Axiom Shell: cd axiom-shell && cargo run"
echo "3. Use the CLI to spin up a kernel: mkdir my-app && cd my-app && ax init"
