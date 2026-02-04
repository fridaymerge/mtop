use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;
use std::collections::HashMap;
use sysinfo::System;
use crate::prices::HardwarePrices;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_electricity_rate")]
    pub electricity_rate_kwh: f64,
    #[serde(default)]
    pub hardware_costs: HardwareCosts,
}

fn default_electricity_rate() -> f64 {
    0.15
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct HardwareCosts {
    pub cpu: Option<f64>,
    pub ram_per_gb: Option<f64>,
    pub disk_per_gb: Option<f64>,
}

pub struct HardwareInfo {
    pub cpu_model: String,
    pub cpu_cores: usize,
    pub cpu_cost: f64,
    pub ram_gb: f64,
    pub ram_cost: f64,
    pub disk_gb: f64,
    pub disk_cost: f64,
    pub cpu_tdp_watts: f64,
    pub electricity_rate: f64,
}

impl HardwareInfo {
    pub fn new(sys: &System) -> Self {
        let config = Self::load_config();
        let market_prices = HardwarePrices::load_or_fetch();

        // Get CPU model from first CPU (they're all the same)
        let cpu_model = sys.cpus().first()
            .map(|cpu| cpu.brand().to_string())
            .unwrap_or_else(|| "Unknown CPU".to_string());
        let cpu_cores = sys.cpus().len();
        let ram_gb = sys.total_memory() as f64 / (1024.0 * 1024.0 * 1024.0);
        let disk_gb = 1000.0; // Placeholder - sysinfo disk support varies by OS

        let cpu_cost = config.hardware_costs.cpu
            .unwrap_or_else(|| Self::estimate_cpu_cost(&cpu_model));

        // Use market prices for RAM if no config override
        let ram_cost = config.hardware_costs.ram_per_gb
            .map(|rate| ram_gb * rate)
            .unwrap_or_else(|| market_prices.get_ram_cost(ram_gb));

        let disk_cost = config.hardware_costs.disk_per_gb
            .map(|rate| disk_gb * rate)
            .unwrap_or_else(|| disk_gb * 0.08);

        let cpu_tdp_watts = Self::estimate_cpu_tdp(&cpu_model);
        let electricity_rate = config.electricity_rate_kwh;

        Self {
            cpu_model,
            cpu_cores,
            cpu_cost,
            ram_gb,
            ram_cost,
            disk_gb,
            disk_cost,
            cpu_tdp_watts,
            electricity_rate,
        }
    }

    fn load_config() -> Config {
        fs::read_to_string("config.json")
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or(Config {
                electricity_rate_kwh: 0.15,
                hardware_costs: HardwareCosts::default(),
            })
    }

    fn estimate_cpu_cost(model: &str) -> f64 {
        let model_lower = model.to_lowercase();

        // Apple Silicon pricing
        if model_lower.contains("apple") || model_lower.contains("m1")
            || model_lower.contains("m2") || model_lower.contains("m3")
            || model_lower.contains("m4") {
            if model_lower.contains("ultra") {
                return 3000.0;
            } else if model_lower.contains("max") {
                return 1500.0;
            } else if model_lower.contains("pro") {
                return 800.0;
            } else {
                return 400.0;
            }
        }

        // Intel pricing
        if model_lower.contains("intel") {
            if model_lower.contains("i9") || model_lower.contains("xeon") {
                return 600.0;
            } else if model_lower.contains("i7") {
                return 400.0;
            } else if model_lower.contains("i5") {
                return 250.0;
            } else if model_lower.contains("i3") {
                return 150.0;
            }
        }

        // AMD pricing
        if model_lower.contains("amd") || model_lower.contains("ryzen") {
            if model_lower.contains("9") || model_lower.contains("threadripper") {
                return 500.0;
            } else if model_lower.contains("7") {
                return 350.0;
            } else if model_lower.contains("5") {
                return 200.0;
            }
        }

        300.0 // Default
    }

    fn estimate_cpu_tdp(model: &str) -> f64 {
        let model_lower = model.to_lowercase();

        // Apple Silicon TDP estimates
        if model_lower.contains("apple") || model_lower.contains("m1")
            || model_lower.contains("m2") || model_lower.contains("m3")
            || model_lower.contains("m4") {
            if model_lower.contains("ultra") {
                return 60.0;
            } else if model_lower.contains("max") {
                return 40.0;
            } else if model_lower.contains("pro") {
                return 30.0;
            } else {
                return 20.0;
            }
        }

        // Intel TDP estimates
        if model_lower.contains("intel") {
            if model_lower.contains("i9") || model_lower.contains("xeon") {
                return 125.0;
            } else if model_lower.contains("i7") || model_lower.contains("i5") {
                return 65.0;
            } else if model_lower.contains("i3") {
                return 51.0;
            }
        }

        // AMD TDP estimates
        if model_lower.contains("amd") || model_lower.contains("ryzen") {
            if model_lower.contains("9") || model_lower.contains("threadripper") {
                return 105.0;
            } else if model_lower.contains("7") || model_lower.contains("5") {
                return 65.0;
            }
        }

        65.0 // Default
    }

    pub fn get_power_consumption(&self, cpu_percent: f32) -> (f64, f64) {
        // CPU scales roughly linearly with usage (idle ~20% TDP, max ~100% TDP)
        let idle_power = self.cpu_tdp_watts * 0.2;
        let active_power = self.cpu_tdp_watts * 0.8;
        let current_power_watts = idle_power + (active_power * (cpu_percent as f64 / 100.0));

        // Add baseline system power (RAM, motherboard, etc.) ~12W
        let total_power_watts = current_power_watts + 12.0;

        // Convert to kWh and calculate cost per second
        let power_kwh = total_power_watts / 1000.0;
        let cost_per_hour = power_kwh * self.electricity_rate;
        let cost_per_second = cost_per_hour / 3600.0;

        (total_power_watts, cost_per_second)
    }

    pub fn get_top_processes(&self, sys: &System, limit: usize) -> Vec<ProcessInfo> {
        let total_memory = sys.total_memory() as f64;
        let cpu_percent = sys.global_cpu_usage();
        let (_power_watts, power_cost_per_second) = self.get_power_consumption(cpu_percent);

        // Get network connections per process
        let net_connections = Self::get_network_connections();

        let mut processes: Vec<ProcessInfo> = sys
            .processes()
            .iter()
            .map(|(pid, process)| {
                let cpu_percent = process.cpu_usage();
                let memory_bytes = process.memory();
                let memory_mb = memory_bytes as f64 / (1024.0 * 1024.0);
                let memory_percent = (memory_bytes as f64 / total_memory) * 100.0;

                // Hardware depreciation cost
                let cpu_cost = (cpu_percent as f64 / 100.0) * self.cpu_cost;
                let mem_cost = (memory_percent / 100.0) * self.ram_cost;
                let hw_cost = cpu_cost + mem_cost;

                // Power cost for this process
                let proc_power_cost = (cpu_percent as f64 / 100.0) * power_cost_per_second;

                let total_cost = hw_cost + proc_power_cost;

                // Get username
                let username = process.user_id()
                    .and_then(|uid| {
                        #[cfg(unix)]
                        {
                            Some(uid.to_string())
                        }
                        #[cfg(not(unix))]
                        {
                            Some("unknown".to_string())
                        }
                    })
                    .unwrap_or_else(|| "unknown".to_string());

                // Get network connection count for this PID
                let net_conn_count = net_connections.get(&(pid.as_u32())).copied().unwrap_or(0);

                ProcessInfo {
                    pid: pid.as_u32(),
                    name: process.name().to_string_lossy().to_string(),
                    username: username.chars().take(8).collect(),
                    cpu_percent,
                    memory_mb,
                    hw_cost,
                    power_cost: proc_power_cost,
                    total_cost,
                    net_connections: net_conn_count,
                }
            })
            .collect();

        // Sort by total cost
        processes.sort_by(|a, b| b.total_cost.partial_cmp(&a.total_cost).unwrap());

        processes.into_iter().take(limit).collect()
    }

    fn get_network_connections() -> HashMap<u32, usize> {
        let mut connections: HashMap<u32, usize> = HashMap::new();

        // Use lsof to get network connections per process (macOS/Linux)
        #[cfg(unix)]
        {
            if let Ok(output) = Command::new("lsof")
                .args(&["-i", "-n", "-P"])
                .output()
            {
                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    for line in stdout.lines().skip(1) {
                        // Parse lsof output
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() > 1 {
                            if let Ok(pid) = parts[1].parse::<u32>() {
                                *connections.entry(pid).or_insert(0) += 1;
                            }
                        }
                    }
                }
            }
        }

        connections
    }
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub username: String,
    pub cpu_percent: f32,
    pub memory_mb: f64,
    pub hw_cost: f64,
    pub power_cost: f64,
    pub total_cost: f64,
    pub net_connections: usize,
}
