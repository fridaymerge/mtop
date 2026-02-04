#!/usr/bin/env python3
"""Quick test to see mtop data without the TUI."""
from hardware import HardwareInfo
import time

hw = HardwareInfo()

print("\n" + "="*60)
print("💰 MTOP TEST OUTPUT")
print("="*60)

print(f"\n🖥️  HARDWARE INFO")
print(f"CPU: {hw.cpu_model}")
print(f"Cores: {hw.cpu_cores} physical / {hw.cpu_threads} threads")
print(f"RAM: {hw.ram_gb} GB")
print(f"Disk: {hw.disk_gb} GB")

print(f"\n💎 HARDWARE VALUES")
print(f"CPU Value:  ${hw.cpu_cost:,.2f}")
print(f"RAM Value:  ${hw.ram_cost:,.2f}")
print(f"Disk Value: ${hw.disk_cost:,.2f}")
print(f"TOTAL:      ${hw.get_total_hardware_cost():,.2f}")

print(f"\n📊 CURRENT USAGE (sampling...)")
time.sleep(0.5)

cpu_data = hw.get_cpu_usage_cost()
mem_data = hw.get_memory_usage_cost()
disk_data = hw.get_disk_usage_cost()

print(f"\nCPU:  {cpu_data['percent']:.1f}% = ${cpu_data['used_cost']:.2f} / ${cpu_data['total_cost']:.2f}")
print(f"RAM:  {mem_data['percent']:.1f}% = ${mem_data['used_cost']:.2f} / ${mem_data['total_cost']:.2f}")
print(f"      ({mem_data['used_gb']:.1f} GB / {mem_data['total_gb']:.1f} GB)")
print(f"Disk: {disk_data['percent']:.1f}% = ${disk_data['used_cost']:.2f} / ${disk_data['total_cost']:.2f}")
print(f"      ({disk_data['used_gb']:.1f} GB / {disk_data['total_gb']:.1f} GB)")

print(f"\n💸 TOP 5 MOST EXPENSIVE PROCESSES")
print(f"{'PID':<8} {'Name':<30} {'CPU%':<8} {'Mem%':<8} {'Cost/s':<10}")
print("-" * 70)

processes = hw.get_top_processes(limit=5)
for proc in processes:
    print(f"{proc['pid']:<8} {proc['name'][:29]:<30} {proc['cpu_percent']:<8.1f} {proc['memory_percent']:<8.1f} ${proc['total_cost']:<9.4f}")

print(f"\n🔥 PER-CORE CPU USAGE")
per_core = cpu_data['per_core']
for i, usage in enumerate(per_core):
    bar_width = 20
    filled = int((usage / 100.0) * bar_width)
    bar = "█" * filled + "░" * (bar_width - filled)
    print(f"Core {i:2d}: [{bar}] {usage:5.1f}%")

print("\n" + "="*60)
print("✅ Test complete! Run 'python3 mtop.py' for the full TUI")
print("="*60 + "\n")
