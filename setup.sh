#!/bin/bash
# Setup script for mtop

echo "💰 mtop - Money Top Setup"
echo "========================="
echo ""

# Check Python version
if ! command -v python3 &> /dev/null; then
    echo "❌ Python 3 is required but not found"
    exit 1
fi

echo "✅ Python 3 found: $(python3 --version)"
echo ""

# Install dependencies
echo "📦 Installing dependencies..."
pip install -r requirements.txt

if [ $? -eq 0 ]; then
    echo "✅ Dependencies installed successfully"
else
    echo "❌ Failed to install dependencies"
    exit 1
fi

echo ""

# Make scripts executable
chmod +x mtop.py
chmod +x run.sh

echo "🔧 Made scripts executable"
echo ""

# Test hardware detection
echo "🖥️  Testing hardware detection..."
python3 -c "from hardware import HardwareInfo; hw = HardwareInfo(); print(f'CPU: {hw.cpu_model}'); print(f'RAM: {hw.ram_gb} GB'); print(f'Total Value: \${hw.get_total_hardware_cost():.2f}')"

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ Setup complete!"
    echo ""
    echo "Run mtop with:"
    echo "  ./run.sh"
    echo "  OR"
    echo "  python3 mtop.py"
    echo ""
    echo "For more info: cat USAGE.md"
else
    echo "❌ Hardware detection test failed"
    exit 1
fi
