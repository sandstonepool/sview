//! IP Geolocation for peer analysis
//!
//! Provides IP-to-location lookup using ip-api.com (free, no API key required).
//! Supports batch queries and caching to minimize API calls.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// Geolocation information for an IP address
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct GeoLocation {
    /// City name
    pub city: String,
    /// Country code (2-letter ISO)
    pub country_code: String,
    /// Full country name
    pub country: String,
    /// Region/state
    pub region: String,
    /// ISP name
    pub isp: Option<String>,
    /// Latitude
    pub lat: Option<f64>,
    /// Longitude
    pub lon: Option<f64>,
}

#[allow(dead_code)]
impl GeoLocation {
    /// Format as short string (City, CC)
    pub fn short(&self) -> String {
        if self.city.is_empty() || self.city == "?" {
            self.country_code.clone()
        } else {
            format!("{}, {}", self.city, self.country_code)
        }
    }
}

/// Cached geolocation entry
#[allow(dead_code)]
struct CacheEntry {
    location: Option<GeoLocation>,
    fetched_at: Instant,
}

/// Geolocation service with caching
#[allow(dead_code)]
pub struct GeoIPService {
    cache: HashMap<String, CacheEntry>,
    cache_ttl: Duration,
    client: reqwest::Client,
    /// Rate limiting: max queries per batch
    batch_limit: usize,
    /// Track last batch time for rate limiting
    last_batch: Option<Instant>,
}

impl Default for GeoIPService {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl GeoIPService {
    /// Create a new GeoIP service
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("Failed to create HTTP client for GeoIP");

        Self {
            cache: HashMap::new(),
            cache_ttl: Duration::from_secs(3600), // 1 hour cache
            client,
            batch_limit: 100, // ip-api.com allows 100 per batch
            last_batch: None,
        }
    }

    /// Check if an IP is private/local (not suitable for geolocation)
    pub fn is_private_ip(ip: &str) -> bool {
        if ip == "127.0.0.1" || ip == "::1" || ip == "localhost" {
            return true;
        }

        // Check IPv4 private ranges
        if let Some(first_octet) = ip.split('.').next() {
            if let Ok(octet) = first_octet.parse::<u8>() {
                // 10.x.x.x, 172.16-31.x.x, 192.168.x.x
                if octet == 10 || octet == 127 {
                    return true;
                }
                if octet == 172 {
                    if let Some(second) = ip.split('.').nth(1) {
                        if let Ok(second_octet) = second.parse::<u8>() {
                            if (16..=31).contains(&second_octet) {
                                return true;
                            }
                        }
                    }
                }
                if octet == 192 {
                    if let Some(second) = ip.split('.').nth(1) {
                        if second == "168" {
                            return true;
                        }
                    }
                }
            }
        }

        // IPv6 private ranges
        if ip.starts_with("fe80:") || ip.starts_with("fc") || ip.starts_with("fd") {
            return true;
        }

        false
    }

    /// Get cached location for an IP (returns None if not cached or expired)
    pub fn get_cached(&self, ip: &str) -> Option<&GeoLocation> {
        if let Some(entry) = self.cache.get(ip) {
            if entry.fetched_at.elapsed() < self.cache_ttl {
                return entry.location.as_ref();
            }
        }
        None
    }

    /// Lookup a single IP (async)
    pub async fn lookup(&mut self, ip: &str) -> Option<GeoLocation> {
        // Check cache first
        if let Some(entry) = self.cache.get(ip) {
            if entry.fetched_at.elapsed() < self.cache_ttl {
                return entry.location.clone();
            }
        }

        // Skip private IPs
        if Self::is_private_ip(ip) {
            return None;
        }

        // Fetch from API
        let url = format!(
            "http://ip-api.com/json/{}?fields=status,country,countryCode,region,city,lat,lon,isp",
            ip
        );

        match self.client.get(&url).send().await {
            Ok(response) => {
                if let Ok(json) = response.json::<serde_json::Value>().await {
                    let location = self.parse_response(&json);
                    self.cache.insert(
                        ip.to_string(),
                        CacheEntry {
                            location: location.clone(),
                            fetched_at: Instant::now(),
                        },
                    );
                    return location;
                }
            }
            Err(e) => {
                warn!("GeoIP lookup failed for {}: {}", ip, e);
            }
        }

        None
    }

    /// Batch lookup multiple IPs (more efficient for many IPs)
    pub async fn lookup_batch(&mut self, ips: &[String]) -> HashMap<String, GeoLocation> {
        let mut results = HashMap::new();
        let mut to_fetch: Vec<String> = Vec::new();

        // Check cache and filter private IPs
        for ip in ips {
            if Self::is_private_ip(ip) {
                continue;
            }

            if let Some(entry) = self.cache.get(ip) {
                if entry.fetched_at.elapsed() < self.cache_ttl {
                    if let Some(loc) = &entry.location {
                        results.insert(ip.clone(), loc.clone());
                    }
                    continue;
                }
            }

            to_fetch.push(ip.clone());
        }

        // Limit batch size
        to_fetch.truncate(self.batch_limit);

        if to_fetch.is_empty() {
            return results;
        }

        // Rate limit: 45 requests per minute for ip-api.com
        if let Some(last) = self.last_batch {
            let elapsed = last.elapsed();
            if elapsed < Duration::from_millis(1500) {
                // Wait a bit before next batch
                debug!("GeoIP rate limiting, skipping batch");
                return results;
            }
        }
        self.last_batch = Some(Instant::now());

        // Build batch request
        let batch_query: Vec<serde_json::Value> = to_fetch
            .iter()
            .map(|ip| {
                serde_json::json!({
                    "query": ip,
                    "fields": "status,country,countryCode,region,city,lat,lon,isp"
                })
            })
            .collect();

        // Fetch batch
        match self
            .client
            .post("http://ip-api.com/batch")
            .json(&batch_query)
            .send()
            .await
        {
            Ok(response) => {
                if let Ok(json_array) = response.json::<Vec<serde_json::Value>>().await {
                    for (i, json) in json_array.iter().enumerate() {
                        if i < to_fetch.len() {
                            let ip = &to_fetch[i];
                            let location = self.parse_response(json);
                            self.cache.insert(
                                ip.clone(),
                                CacheEntry {
                                    location: location.clone(),
                                    fetched_at: Instant::now(),
                                },
                            );
                            if let Some(loc) = location {
                                results.insert(ip.clone(), loc);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                warn!("GeoIP batch lookup failed: {}", e);
            }
        }

        results
    }

    /// Parse ip-api.com JSON response
    fn parse_response(&self, json: &serde_json::Value) -> Option<GeoLocation> {
        let status = json.get("status")?.as_str()?;
        if status != "success" {
            return None;
        }

        Some(GeoLocation {
            city: json
                .get("city")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string(),
            country_code: json
                .get("countryCode")
                .and_then(|v| v.as_str())
                .unwrap_or("??")
                .to_string(),
            country: json
                .get("country")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string(),
            region: json
                .get("region")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            isp: json.get("isp").and_then(|v| v.as_str()).map(String::from),
            lat: json.get("lat").and_then(|v| v.as_f64()),
            lon: json.get("lon").and_then(|v| v.as_f64()),
        })
    }

    /// Clear the cache
    #[allow(dead_code)]
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache statistics
    #[allow(dead_code)]
    pub fn cache_stats(&self) -> (usize, usize) {
        let total = self.cache.len();
        let valid = self
            .cache
            .values()
            .filter(|e| e.fetched_at.elapsed() < self.cache_ttl)
            .count();
        (valid, total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_private_ip() {
        assert!(GeoIPService::is_private_ip("127.0.0.1"));
        assert!(GeoIPService::is_private_ip("10.0.0.1"));
        assert!(GeoIPService::is_private_ip("192.168.1.1"));
        assert!(GeoIPService::is_private_ip("172.16.0.1"));
        assert!(GeoIPService::is_private_ip("172.31.255.255"));
        assert!(!GeoIPService::is_private_ip("8.8.8.8"));
        assert!(!GeoIPService::is_private_ip("1.1.1.1"));
        assert!(!GeoIPService::is_private_ip("172.32.0.1")); // Not private
    }

    #[test]
    fn test_geo_location_short() {
        let loc = GeoLocation {
            city: "Sydney".to_string(),
            country_code: "AU".to_string(),
            country: "Australia".to_string(),
            region: "NSW".to_string(),
            isp: None,
            lat: None,
            lon: None,
        };
        assert_eq!(loc.short(), "Sydney, AU");
    }
}
