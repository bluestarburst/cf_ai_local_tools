#!/bin/bash

# Quick Start Script for CF AI Local Tools
# This script helps you get started with local development

set -e

echo "🚀 CF AI Local Tools - Quick Start"
echo "===================================="
echo ""

# Check prerequisites
echo "📋 Checking prerequisites..."

if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo not found. Please install from https://rustup.rs/"
    exit 1
fi
echo "✅ Rust/Cargo found"

if ! command -v node &> /dev/null; then
    echo "❌ Node.js not found. Please install Node.js v18+"
    exit 1
fi
echo "✅ Node.js found"

if ! command -v pnpm &> /dev/null; then
    echo "❌ pnpm not found"
    exit 1
fi
echo "✅ pnpm found"

echo ""
echo "📦 Installing dependencies..."
echo ""

# Install Cloudflare Worker dependencies
echo "Installing Worker dependencies..."
cd cf-worker
pnpm install
cd ..

# Install Web Viewer dependencies
echo "Installing Web Viewer dependencies..."
cd web-viewer
pnpm install

# Create .env.local if it doesn't exist
if [ ! -f .env.local ]; then
    echo "VITE_WORKER_URL=http://localhost:8787" > .env.local
    echo "✅ Created .env.local"
fi

cd ..

# Build Rust app
echo ""
echo "🔨 Building Rust app..."
cargo build

echo ""
echo "✅ Setup complete!"
echo ""
echo "📖 Next steps:"
echo ""
echo "1️⃣  Start the Cloudflare Worker (Terminal 1):"
echo "   cd cf-worker && npx wrangler dev --port 8787"
echo ""
echo "2️⃣  Start the Rust app (Terminal 2):"
echo "   cargo run"
echo ""
echo "3️⃣  Start the Web Viewer (Terminal 3):"
echo "   cd web-viewer && pnpm run dev"
echo ""
echo "4️⃣  Open http://localhost:3000 in your browser"
echo ""
echo "💡 See README.md for more details"
echo ""
