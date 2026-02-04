"""Hardware detection and cost estimation module."""
import platform
import psutil
import subprocess
import re
import json
import os
from typing import Dict, Optional


class HardwareInfo:
    """Detect hardware specs and estimate costs."""

    def __init__(self, config_path: str = "config.json"):
        self.cpu_model = self._get_cpu_model()
        self.cpu_cores = psutil.cpu_count(logical=False)
        self.cpu_threads = psutil.cpu_count(logical=True)
        self.ram_gb = round(psutil.virtual_memory().total / (1024**3), 1)
        self.disk_gb = round(psutil.disk_usage('/').total / (1024**3), 1)

        # Load config if exists
        self.config = self._load_config(config_path)

        # Estimate costs (can be overridden in config)
        self.cpu_cost = self._get_cpu_cost()
        self.ram_cost = self._get_ram_cost()
        self.disk_cost = self._get_disk_cost()

        # Power consumption estimates
        self.cpu_tdp_watts = self._estimate_cpu_tdp()
        self.electricity_rate = self.config.get('electricity_rate_kwh', 0.15)  # $/kWh

    def _load_config(self, config_path: str) -> Dict:
        """Load configuration from JSON file if it exists."""
        if os.path.exists(config_path):
            try:
                with open(config_path, 'r') as f:
                    return json.load(f)
            except Exception:
                pass
        return {}

    def _get_cpu_cost(self) -> float:
        """Get CPU cost from config or estimate."""
        if 'hardware_costs' in self.config and 'cpu' in self.config['hardware_costs']:
            return float(self.config['hardware_costs']['cpu'])
        return self._estimate_cpu_cost()

    def _get_ram_cost(self) -> float:
        """Get RAM cost from config or estimate."""
        if 'hardware_costs' in self.config and 'ram_per_gb' in self.config['hardware_costs']:
            ram_per_gb = float(self.config['hardware_costs']['ram_per_gb'])
            return self.ram_gb * ram_per_gb
        return self._estimate_ram_cost()

    def _get_disk_cost(self) -> float:
        """Get disk cost from config or estimate."""
        if 'hardware_costs' in self.config and 'disk_per_gb' in self.config['hardware_costs']:
            disk_per_gb = float(self.config['hardware_costs']['disk_per_gb'])
            return self.disk_gb * disk_per_gb
        return self._estimate_disk_cost()

    def _get_cpu_model(self) -> str:
        """Get CPU model name."""
        try:
            if platform.system() == "Darwin":  # macOS
                cmd = ["sysctl", "-n", "machdep.cpu.brand_string"]
                result = subprocess.check_output(cmd, text=True).strip()
                return result
            else:
                # Fallback for other systems
                import cpuinfo
                info = cpuinfo.get_cpu_info()
                return info.get('brand_raw', 'Unknown CPU')
        except Exception:
            return "Unknown CPU"

    def _estimate_cpu_cost(self) -> float:
        """Estimate CPU cost based on model."""
        cpu_lower = self.cpu_model.lower()

        # Apple Silicon pricing
        if 'apple' in cpu_lower or 'm1' in cpu_lower or 'm2' in cpu_lower or 'm3' in cpu_lower or 'm4' in cpu_lower:
            if 'ultra' in cpu_lower:
                return 3000.0
            elif 'max' in cpu_lower:
                return 1500.0
            elif 'pro' in cpu_lower:
                return 800.0
            else:
                return 400.0

        # Intel pricing
        if 'intel' in cpu_lower:
            if 'i9' in cpu_lower or 'xeon' in cpu_lower:
                return 600.0
            elif 'i7' in cpu_lower:
                return 400.0
            elif 'i5' in cpu_lower:
                return 250.0
            elif 'i3' in cpu_lower:
                return 150.0

        # AMD pricing
        if 'amd' in cpu_lower or 'ryzen' in cpu_lower:
            if '9' in cpu_lower or 'threadripper' in cpu_lower:
                return 500.0
            elif '7' in cpu_lower:
                return 350.0
            elif '5' in cpu_lower:
                return 200.0

        # Default
        return 300.0

    def _estimate_ram_cost(self) -> float:
        """Estimate RAM cost (~$5-8 per GB)."""
        return self.ram_gb * 7.0

    def _estimate_disk_cost(self) -> float:
        """Estimate disk cost (~$0.05-0.10 per GB for SSD)."""
        return self.disk_gb * 0.08

    def _estimate_cpu_tdp(self) -> float:
        """Estimate CPU TDP (Thermal Design Power) in watts."""
        cpu_lower = self.cpu_model.lower()

        # Apple Silicon TDP estimates
        if 'apple' in cpu_lower or 'm1' in cpu_lower or 'm2' in cpu_lower or 'm3' in cpu_lower or 'm4' in cpu_lower:
            if 'ultra' in cpu_lower:
                return 60.0  # M1/M2 Ultra
            elif 'max' in cpu_lower:
                return 40.0  # M1/M2/M3 Max
            elif 'pro' in cpu_lower:
                return 30.0  # M1/M2/M3 Pro
            else:
                return 20.0  # Base M1/M2/M3

        # Intel TDP estimates
        if 'intel' in cpu_lower:
            if 'i9' in cpu_lower or 'xeon' in cpu_lower:
                return 125.0
            elif 'i7' in cpu_lower:
                return 65.0
            elif 'i5' in cpu_lower:
                return 65.0
            elif 'i3' in cpu_lower:
                return 51.0

        # AMD TDP estimates
        if 'amd' in cpu_lower or 'ryzen' in cpu_lower:
            if '9' in cpu_lower or 'threadripper' in cpu_lower:
                return 105.0
            elif '7' in cpu_lower:
                return 65.0
            elif '5' in cpu_lower:
                return 65.0

        # Default estimate
        return 65.0

    def get_total_hardware_cost(self) -> float:
        """Get total estimated hardware cost."""
        return self.cpu_cost + self.ram_cost + self.disk_cost

    def get_power_consumption(self) -> Dict[str, float]:
        """Calculate current power consumption and cost."""
        cpu_percent = psutil.cpu_percent(interval=0.1)

        # Estimate current power draw
        # CPU scales roughly linearly with usage (idle ~20% TDP, max ~100% TDP)
        idle_power = self.cpu_tdp_watts * 0.2
        active_power = self.cpu_tdp_watts * 0.8
        current_power_watts = idle_power + (active_power * (cpu_percent / 100.0))

        # Add baseline system power (RAM, motherboard, etc.) ~10-15W
        system_baseline = 12.0
        total_power_watts = current_power_watts + system_baseline

        # Convert to kWh and calculate cost per hour
        power_kwh = total_power_watts / 1000.0
        cost_per_hour = power_kwh * self.electricity_rate
        cost_per_second = cost_per_hour / 3600.0

        return {
            'cpu_watts': current_power_watts,
            'total_watts': total_power_watts,
            'cost_per_hour': cost_per_hour,
            'cost_per_second': cost_per_second,
            'electricity_rate': self.electricity_rate
        }

    def get_cpu_usage_cost(self) -> Dict[str, float]:
        """Calculate current CPU usage cost."""
        cpu_percent = psutil.cpu_percent(interval=0.1)
        used_cost = (cpu_percent / 100.0) * self.cpu_cost

        # Get power consumption
        power_data = self.get_power_consumption()

        return {
            'percent': cpu_percent,
            'used_cost': used_cost,
            'total_cost': self.cpu_cost,
            'per_core': self._get_per_core_usage(),
            'power_watts': power_data['total_watts'],
            'power_cost_per_sec': power_data['cost_per_second']
        }

    def _get_per_core_usage(self) -> list:
        """Get per-core CPU usage."""
        return psutil.cpu_percent(interval=0.1, percpu=True)

    def get_memory_usage_cost(self) -> Dict[str, float]:
        """Calculate current memory usage cost."""
        mem = psutil.virtual_memory()
        used_cost = (mem.percent / 100.0) * self.ram_cost
        return {
            'percent': mem.percent,
            'used_cost': used_cost,
            'total_cost': self.ram_cost,
            'used_gb': round(mem.used / (1024**3), 1),
            'total_gb': self.ram_gb,
            'available_gb': round(mem.available / (1024**3), 1)
        }

    def get_disk_usage_cost(self) -> Dict[str, float]:
        """Calculate current disk usage cost."""
        disk = psutil.disk_usage('/')
        used_cost = (disk.percent / 100.0) * self.disk_cost
        return {
            'percent': disk.percent,
            'used_cost': used_cost,
            'total_cost': self.disk_cost,
            'used_gb': round(disk.used / (1024**3), 1),
            'total_gb': self.disk_gb,
            'free_gb': round(disk.free / (1024**3), 1)
        }

    def get_network_stats(self) -> Dict:
        """Get network I/O stats."""
        net = psutil.net_io_counters()
        return {
            'bytes_sent': net.bytes_sent,
            'bytes_recv': net.bytes_recv,
            'packets_sent': net.packets_sent,
            'packets_recv': net.packets_recv
        }

    def get_top_processes(self, limit: int = 10) -> list:
        """Get top processes by CPU and memory usage."""
        # Get power consumption data
        power_data = self.get_power_consumption()
        power_cost_per_second = power_data['cost_per_second']

        processes = []
        for proc in psutil.process_iter(['pid', 'name', 'cpu_percent', 'memory_percent', 'memory_info', 'username']):
            try:
                info = proc.info
                # Calculate hardware depreciation cost for this process
                cpu_cost = (info['cpu_percent'] / 100.0) * self.cpu_cost if info['cpu_percent'] else 0
                mem_cost = (info['memory_percent'] / 100.0) * self.ram_cost if info['memory_percent'] else 0

                # Calculate power cost for this process
                proc_power_cost = (info['cpu_percent'] / 100.0) * power_cost_per_second if info['cpu_percent'] else 0

                processes.append({
                    'pid': info['pid'],
                    'name': info['name'],
                    'username': info.get('username', 'unknown'),
                    'cpu_percent': info['cpu_percent'] or 0,
                    'memory_percent': info['memory_percent'] or 0,
                    'memory_mb': round(info['memory_info'].rss / (1024**2), 1) if info['memory_info'] else 0,
                    'cpu_cost': cpu_cost,
                    'mem_cost': mem_cost,
                    'power_cost': proc_power_cost,
                    'total_cost': cpu_cost + mem_cost + proc_power_cost
                })
            except (psutil.NoSuchProcess, psutil.AccessDenied):
                continue

        # Sort by total cost
        processes.sort(key=lambda x: x['total_cost'], reverse=True)
        return processes[:limit]
