# mtop Usage Guide 💰

## Quick Start

```bash
# Run mtop
python3 mtop.py

# Or use the launcher script
./run.sh
```

## What You'll See

Your terminal will display:

### 1. Hardware Inventory Panel
- Your CPU model and estimated value
- RAM capacity and value
- Disk capacity and value
- Total hardware value

### 2. Resource Cost Bars
- **CPU Usage**: Shows real-time CPU usage as dollars
  - Example: `$0.75 / $1500.00` means you're using 75% of your CPU
- **Memory Usage**: RAM usage in dollars with GB amounts
- **Disk Usage**: Storage usage in dollars with GB amounts

### 3. CPU Cores Grid
- Per-core CPU utilization
- Color-coded by usage (green < 50%, yellow < 75%, red >= 75%)

### 4. Most Expensive Processes
- Top 12 processes ranked by "cost"
- Shows CPU%, Memory%, and their combined dollar value per second
- Helps you see which apps are "expensive" to run

## Keyboard Controls

- `q` - Quit mtop
- `r` - Force refresh all data

## Customization

### Setting Custom Hardware Costs

1. Copy the example config:
   ```bash
   cp config.json.example config.json
   ```

2. Edit `config.json`:
   ```json
   {
     "hardware_costs": {
       "cpu": 1500.0,           # Your CPU's value
       "ram_per_gb": 7.0,       # Cost per GB of RAM
       "disk_per_gb": 0.08      # Cost per GB of disk
     }
   }
   ```

3. Run mtop - it will use your custom values

## Understanding the Costs

### How CPU Cost Works
If your CPU is worth $1500:
- At 0% usage: $0.00 (idle)
- At 50% usage: $750.00 (half capacity)
- At 100% usage: $1500.00 (maxed out)

This represents "how much of your hardware investment" you're actively using.

### Process Costs
Each process shows a "cost per second" which is:
- CPU cost: `(process CPU% / 100) × total_cpu_value`
- Memory cost: `(process Memory% / 100) × total_ram_value`
- Total: sum of both

Example: A process using 10% CPU and 5% RAM on a $1500 CPU and $400 RAM machine:
- CPU cost: `0.10 × $1500 = $150.00`
- Memory cost: `0.05 × $400 = $20.00`
- Total cost: `$170.00` per second of runtime

## Hardware Price Estimates

Default pricing logic:

**CPUs:**
- Apple M4/M3/M2/M1 Ultra: $3000
- Apple M4/M3/M2/M1 Max: $1500
- Apple M4/M3/M2/M1 Pro: $800
- Apple M4/M3/M2/M1: $400
- Intel i9/Xeon: $600
- Intel i7: $400
- Intel i5: $250
- Intel i3: $150
- AMD Ryzen 9/Threadripper: $500
- AMD Ryzen 7: $350
- AMD Ryzen 5: $200

**RAM:** $7 per GB (typical market rate)

**Disk:** $0.08 per GB (SSD pricing)

## Tips

- Watch which processes are "expensive" and consider if you need them running
- Use this to justify hardware upgrades: "I'm constantly maxing out my CPU value!"
- Great for awareness of background processes burning through resources
- Makes resource monitoring way more interesting

## Troubleshooting

### "Permission denied" errors
Some processes may be inaccessible. This is normal - system processes often require elevated privileges.

### Numbers seem off?
Edit `config.json` to set accurate hardware costs for your specific machine.

### Refresh is slow?
The app updates every second. CPU usage calculations need a brief interval to be accurate.

## Fun Facts

- Running at 100% CPU for 1 hour doesn't "cost" you anything in real money (obviously)
- This is purely a fun way to visualize resource usage
- But it does help understand depreciation and "wear" on hardware
- Makes you think twice about Chrome with 50 tabs open
