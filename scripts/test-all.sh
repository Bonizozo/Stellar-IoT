#!/bin/bash

# Run all tests across the monorepo

set -e

echo "🧪 Running all tests..."

# Test backend
echo ""
echo "Testing backend..."
cd apps/api
cargo test
cd ../..

# Test contracts
echo ""
echo "Testing smart contracts..."
cd contracts/iot
cargo test
cd ../..

# Test frontend (if tests exist)
echo ""
echo "Testing frontend..."
cd apps/web
if [ -f "package.json" ] && grep -q "\"test\"" package.json; then
  npm test
else
  echo "⚠️  No frontend tests configured yet"
fi
cd ../..

echo ""
echo "✅ All tests passed!"
