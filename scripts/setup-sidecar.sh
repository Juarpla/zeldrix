#!/bin/bash
# Setup script for Zeldrix llama.cpp sidecar
# Run this script to prepare the environment for the sidecar

set -e

DATA_DIR="$HOME/Library/Application Support/zeldrix"
BIN_DIR="$DATA_DIR/bin"
MODELS_DIR="$DATA_DIR/models"

echo "=== Zeldrix Sidecar Setup ==="
echo ""

# Create directories
echo "Creating directories..."
mkdir -p "$BIN_DIR"
mkdir -p "$MODELS_DIR"
echo "  Created: $DATA_DIR"
echo "  Created: $BIN_DIR"
echo "  Created: $MODELS_DIR"
echo ""

# Check if llama-server exists
if [ -f "$BIN_DIR/llama-server" ]; then
    echo "✓ llama-server found at $BIN_DIR/llama-server"
else
    echo "✗ llama-server NOT found"
    echo ""
    echo "To build llama-server, run these commands:"
    echo ""
    echo "  # Clone llama.cpp"
    echo "  git clone https://github.com/ggerganov/llama.cpp"
    echo "  cd llama.cpp"
    echo ""
    echo "  # Build with Metal support (Apple Silicon)"
    echo "  make LLAMA_METAL=1"
    echo ""
    echo "  # Copy binary to app support directory"
    echo "  cp build/bin/llama-server $BIN_DIR/"
    echo ""
fi
echo ""

# Check if model exists
MODEL_FILE="$MODELS_DIR/gemma-4-E2B-it-IQ4_XS.gguf"
if [ -f "$MODEL_FILE" ]; then
    echo "✓ Model found at $MODEL_FILE"
else
    echo "✗ Model NOT found"
    echo ""
    echo "To download the model, run these commands:"
    echo ""
    echo "  # Install huggingface-cli (if not installed)"
    echo "  pip install huggingface_hub"
    echo ""
    echo "  # Download the model"
    echo "  huggingface-cli download \\"
    echo "      google/gemma-4-E2B-it-IQ4_XS \\"
    echo "      gemma-4-E2B-it-IQ4_XS.gguf \\"
    echo "      --local-dir \"$MODELS_DIR\""
    echo ""
fi
echo ""

echo "=== Setup Complete ==="
echo ""
echo "Environment variables (optional):"
echo "  ZELDRIX_LLAMA_SERVER=$BIN_DIR/llama-server"
echo "  ZELDRIX_MODEL_PATH=$MODEL_FILE"
echo ""
echo "You can set these in your shell profile or pass them when running the app."