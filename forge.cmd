@echo off
cd /d "%~dp0"
cargo run --quiet --bin forge -- %*
