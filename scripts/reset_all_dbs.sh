#!/bin/bash
set -e

# Script to reset all databases (Auth, Driver, Rider, Admin)
# This uses cargo make tasks defined in Makefile.toml

echo "=========================================="
echo "⚠️  WARNING: This will DROP and RECREATE all databases!"
echo "   - movve_auth"
echo "   - movve_driver"
echo "   - movve_rider"
echo "   - movve_admin"
echo "=========================================="
echo ""
read -p "Are you sure you want to continue? (y/N) " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted."
    exit 1
fi

echo ""
echo "🔄 Resetting Auth Service DB..."
cargo make diesel-auth-reset

echo ""
echo "🔄 Resetting Driver Service DB..."
cargo make diesel-driver-reset

echo ""
echo "🔄 Resetting Rider Service DB..."
cargo make diesel-rider-reset

echo ""
echo "🔄 Resetting Admin Service DB..."
cargo make diesel-admin-reset

echo ""
echo "✅ All databases have been reset successfully!"
