use anyhow::Result;
use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HardwarePrices {
    pub ram_per_gb: f64,
    pub cpu_multiplier: f64, // Multiplier for CPU estimates
    pub last_updated: DateTime<Utc>,
}

impl Default for HardwarePrices {
    fn default() -> Self {
        Self {
            ram_per_gb: 12.0, // 2026 default
            cpu_multiplier: 1.0,
            last_updated: Utc::now(),
        }
    }
}

impl HardwarePrices {
    pub fn cache_path() -> Result<PathBuf> {
        let cache_dir = dirs::cache_dir()
            .or_else(|| dirs::home_dir().map(|h| h.join(".cache")))
            .ok_or_else(|| anyhow::anyhow!("Could not find cache directory"))?;

        let mtop_cache = cache_dir.join("mtop");
        fs::create_dir_all(&mtop_cache)?;

        Ok(mtop_cache.join("prices.json"))
    }

    pub fn load_or_fetch() -> Self {
        // Try to load from cache
        if let Ok(cached) = Self::load_from_cache() {
            // Check if we need to update (once per day)
            let now = Local::now();
            let last_update = DateTime::<Local>::from(cached.last_updated);

            if now.date_naive() == last_update.date_naive() {
                // Already updated today
                return cached;
            }
        }

        // Fetch new prices
        match Self::fetch_prices() {
            Ok(prices) => {
                // Save to cache
                let _ = prices.save_to_cache();
                prices
            }
            Err(_) => {
                // Fall back to cached or default
                Self::load_from_cache().unwrap_or_default()
            }
        }
    }

    fn load_from_cache() -> Result<Self> {
        let path = Self::cache_path()?;
        let contents = fs::read_to_string(path)?;
        let prices: Self = serde_json::from_str(&contents)?;
        Ok(prices)
    }

    fn save_to_cache(&self) -> Result<()> {
        let path = Self::cache_path()?;
        let contents = serde_json::to_string_pretty(self)?;
        fs::write(path, contents)?;
        Ok(())
    }

    fn fetch_prices() -> Result<Self> {
        // Fetch current RAM prices from PCPartPicker data
        // For now, we'll scrape a simple endpoint or use defaults with trend data

        // Try to get RAM prices from a simple API
        let ram_price = Self::fetch_ram_price().unwrap_or(12.0);

        Ok(Self {
            ram_per_gb: ram_price,
            cpu_multiplier: 1.0,
            last_updated: Utc::now(),
        })
    }

    fn fetch_ram_price() -> Result<f64> {
        // Scrape RAM prices from a reliable source
        // Using PCPartPicker as example (they have public data)

        // For production, you'd want to scrape actual data
        // For now, we'll use a formula based on date/trends

        // RAM has been getting more expensive in 2025-2026
        // Baseline: $8/GB in 2024, trending up ~15% per year
        let base_price: f64 = 8.0;
        let years_since_2024: f64 = 2.0; // We're in 2026
        let inflation_rate: f64 = 0.15; // 15% per year

        let current_price = base_price * (1.0 + inflation_rate).powf(years_since_2024);

        // Try to fetch real data (placeholder for actual implementation)
        // In real implementation, you'd parse HTML from PCPartPicker or similar
        if let Ok(response) = reqwest::blocking::get("https://httpbin.org/status/200") {
            if response.status().is_success() {
                // Successfully connected - in production, parse actual price data
                // For now, return calculated estimate
                return Ok(current_price);
            }
        }

        // Fallback to calculation
        Ok(current_price)
    }

    pub fn get_ram_cost(&self, ram_gb: f64) -> f64 {
        ram_gb * self.ram_per_gb
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_calculation() {
        let prices = HardwarePrices::default();
        assert_eq!(prices.get_ram_cost(16.0), 16.0 * 12.0);
    }

    #[test]
    fn test_cache_path() {
        let path = HardwarePrices::cache_path();
        assert!(path.is_ok());
    }
}
