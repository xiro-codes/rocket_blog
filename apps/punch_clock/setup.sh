#!/bin/bash

# Setup script for Punch Clock App
echo "🕐 Setting up Punch Clock App..."

# Build the application
echo "Building punch clock application..."
cd "$(dirname "$0")"
cargo build -p punch_clock --release

if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    echo ""
    echo "🚀 To run the Punch Clock app:"
    echo "   cd apps/punch_clock"
    echo "   cargo run"
    echo ""
    echo "📱 Then visit: http://localhost:8001/punch-clock"
    echo ""
    echo "📋 Features available:"
    echo "   • Clock in/out for different work roles"
    echo "   • Track time and automatic earnings calculation"
    echo "   • View work history and statistics"
    echo "   • Admin role management"
    echo ""
    echo "⚠️  Note: You'll need to create work roles first before clocking in."
    echo "   Visit /punch-clock/roles to manage roles (admin required)."
else
    echo "❌ Build failed. Please check the error messages above."
    exit 1
fi