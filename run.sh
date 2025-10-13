#!/bin/bash
# Quick run script for DBI Backend

echo "🦀 Starting DBI Backend (Rust Edition)..."
echo ""
echo "Make sure:"
echo "1. Switch is connected via USB"
echo "2. DBI is running on Switch"
echo "3. You've selected 'Install title from DBIbackend' on Switch"
echo ""

./target/release/dbi-backend-rust
