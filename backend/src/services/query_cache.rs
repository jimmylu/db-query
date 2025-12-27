// Query Result Cache Service
//
// Implements LRU cache for query results with TTL support.
// Reduces database load by caching frequently accessed query results.

use crate::services::database::adapter::QueryResult;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// Cached query result with metadata
#[derive(Debug, Clone)]
struct CachedResult {
    /// Query result data
    result: QueryResult,
    /// Time when cached
    cached_at: Instant,
    /// Time-to-live duration
    ttl: Duration,
    /// Number of times this cache entry was hit
    hit_count: u64,
}

impl CachedResult {
    /// Check if cache entry is expired
    fn is_expired(&self) -> bool {
        self.cached_at.elapsed() > self.ttl
    }
}

/// LRU cache entry for tracking access order
#[derive(Debug, Clone)]
struct LruEntry {
    /// Cache key
    key: String,
    /// Last access time
    last_accessed: Instant,
}

/// Query result cache with LRU eviction and TTL
///
/// Features:
/// - LRU eviction when cache is full
/// - TTL-based expiration
/// - Cache statistics (hit/miss ratio)
/// - Configurable max size and default TTL
pub struct QueryResultCache {
    /// Cache storage (key -> cached result)
    cache: Arc<Mutex<HashMap<String, CachedResult>>>,
    /// LRU tracking (ordered by last access time)
    lru_list: Arc<Mutex<Vec<LruEntry>>>,
    /// Maximum number of entries
    max_size: usize,
    /// Default TTL for cached entries
    default_ttl: Duration,
    /// Cache statistics
    stats: Arc<Mutex<CacheStats>>,
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Total cache hits
    pub hits: u64,
    /// Total cache misses
    pub misses: u64,
    /// Total evictions
    pub evictions: u64,
    /// Total expirations
    pub expirations: u64,
}

impl CacheStats {
    /// Calculate hit ratio (0.0 to 1.0)
    pub fn hit_ratio(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

impl QueryResultCache {
    /// Create a new query cache
    ///
    /// # Arguments
    ///
    /// * `max_size` - Maximum number of cached entries (default: 1000)
    /// * `default_ttl_secs` - Default TTL in seconds (default: 300 = 5 minutes)
    pub fn new(max_size: usize, default_ttl_secs: u64) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            lru_list: Arc::new(Mutex::new(Vec::new())),
            max_size,
            default_ttl: Duration::from_secs(default_ttl_secs),
            stats: Arc::new(Mutex::new(CacheStats::default())),
        }
    }

    /// Create with default settings (1000 entries, 5 min TTL)
    pub fn default() -> Self {
        Self::new(1000, 300)
    }

    /// Generate cache key from SQL query and connection ID
    ///
    /// Uses hash for efficient key generation
    pub fn generate_key(connection_id: &str, sql: &str) -> String {
        let mut hasher = DefaultHasher::new();
        connection_id.hash(&mut hasher);
        sql.hash(&mut hasher);
        format!("{}:{:x}", connection_id, hasher.finish())
    }

    /// Get cached result if available and not expired
    ///
    /// Returns None if cache miss or expired
    pub fn get(&self, key: &str) -> Option<QueryResult> {
        let mut cache = self.cache.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        if let Some(cached) = cache.get_mut(key) {
            if cached.is_expired() {
                // Expired - remove and count as miss
                cache.remove(key);
                stats.misses += 1;
                stats.expirations += 1;

                // Remove from LRU list
                let mut lru = self.lru_list.lock().unwrap();
                lru.retain(|entry| entry.key != key);

                tracing::debug!("Cache expired for key: {}", key);
                return None;
            }

            // Hit - update access time and stats
            cached.hit_count += 1;
            stats.hits += 1;

            // Update LRU
            let mut lru = self.lru_list.lock().unwrap();
            if let Some(entry) = lru.iter_mut().find(|e| e.key == key) {
                entry.last_accessed = Instant::now();
            }

            tracing::debug!("Cache hit for key: {} (hit_count: {})", key, cached.hit_count);
            return Some(cached.result.clone());
        }

        // Miss
        stats.misses += 1;
        tracing::debug!("Cache miss for key: {}", key);
        None
    }

    /// Store query result in cache
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key (use generate_key)
    /// * `result` - Query result to cache
    /// * `ttl` - Optional custom TTL (uses default if None)
    pub fn put(&self, key: String, result: QueryResult, ttl: Option<Duration>) {
        let mut cache = self.cache.lock().unwrap();
        let mut lru = self.lru_list.lock().unwrap();

        // Check if we need to evict
        if cache.len() >= self.max_size && !cache.contains_key(&key) {
            self.evict_lru(&mut cache, &mut lru);
        }

        let cached = CachedResult {
            result,
            cached_at: Instant::now(),
            ttl: ttl.unwrap_or(self.default_ttl),
            hit_count: 0,
        };

        cache.insert(key.clone(), cached);

        // Add to LRU list
        lru.push(LruEntry {
            key: key.clone(),
            last_accessed: Instant::now(),
        });

        tracing::debug!("Cached result for key: {} (cache size: {})", key, cache.len());
    }

    /// Evict least recently used entry
    fn evict_lru(&self, cache: &mut HashMap<String, CachedResult>, lru: &mut Vec<LruEntry>) {
        if lru.is_empty() {
            return;
        }

        // Sort by last accessed time (oldest first)
        lru.sort_by_key(|entry| entry.last_accessed);

        // Remove oldest
        if let Some(oldest) = lru.first() {
            let key_to_remove = oldest.key.clone();
            cache.remove(&key_to_remove);
            lru.remove(0);

            let mut stats = self.stats.lock().unwrap();
            stats.evictions += 1;

            tracing::debug!("Evicted cache entry: {}", key_to_remove);
        }
    }

    /// Clear all cache entries
    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        let mut lru = self.lru_list.lock().unwrap();

        let count = cache.len();
        cache.clear();
        lru.clear();

        tracing::info!("Cleared {} cache entries", count);
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> CacheStats {
        self.stats.lock().unwrap().clone()
    }

    /// Get current cache size
    pub fn size(&self) -> usize {
        self.cache.lock().unwrap().len()
    }

    /// Remove expired entries
    pub fn cleanup_expired(&self) {
        let mut cache = self.cache.lock().unwrap();
        let mut lru = self.lru_list.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        let expired_keys: Vec<String> = cache
            .iter()
            .filter(|(_, cached)| cached.is_expired())
            .map(|(key, _)| key.clone())
            .collect();

        for key in &expired_keys {
            cache.remove(key);
            lru.retain(|entry| &entry.key != key);
            stats.expirations += 1;
        }

        if !expired_keys.is_empty() {
            tracing::info!("Cleaned up {} expired cache entries", expired_keys.len());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_result() -> QueryResult {
        QueryResult {
            rows: vec![
                json!({"id": 1, "name": "Alice"}),
                json!({"id": 2, "name": "Bob"}),
            ],
            row_count: 2,
            execution_time_ms: 100,
        }
    }

    #[test]
    fn test_cache_creation() {
        let cache = QueryResultCache::new(100, 60);
        assert_eq!(cache.size(), 0);
        assert_eq!(cache.max_size, 100);
    }

    #[test]
    fn test_cache_key_generation() {
        let key1 = QueryResultCache::generate_key("conn1", "SELECT * FROM users");
        let key2 = QueryResultCache::generate_key("conn1", "SELECT * FROM users");
        let key3 = QueryResultCache::generate_key("conn2", "SELECT * FROM users");

        assert_eq!(key1, key2); // Same query, same connection
        assert_ne!(key1, key3); // Different connection
    }

    #[test]
    fn test_cache_put_and_get() {
        let cache = QueryResultCache::new(10, 60);
        let result = create_test_result();

        let key = "test_key".to_string();
        cache.put(key.clone(), result.clone(), None);

        let cached = cache.get(&key);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().row_count, 2);
    }

    #[test]
    fn test_cache_miss() {
        let cache = QueryResultCache::new(10, 60);
        let result = cache.get("nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_cache_expiration() {
        let cache = QueryResultCache::new(10, 1); // 1 second TTL
        let result = create_test_result();

        let key = "test_key".to_string();
        cache.put(key.clone(), result, Some(Duration::from_millis(100)));

        // Should exist immediately
        assert!(cache.get(&key).is_some());

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(150));

        // Should be expired
        assert!(cache.get(&key).is_none());

        let stats = cache.get_stats();
        assert_eq!(stats.expirations, 1);
    }

    #[test]
    fn test_cache_stats() {
        let cache = QueryResultCache::new(10, 60);
        let result = create_test_result();

        let key = "test_key".to_string();
        cache.put(key.clone(), result, None);

        // Generate some hits and misses
        cache.get(&key); // hit
        cache.get(&key); // hit
        cache.get("nonexistent"); // miss

        let stats = cache.get_stats();
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert!(stats.hit_ratio() > 0.6); // 2/3 ≈ 0.67
    }

    #[test]
    fn test_cache_clear() {
        let cache = QueryResultCache::new(10, 60);
        let result = create_test_result();

        cache.put("key1".to_string(), result.clone(), None);
        cache.put("key2".to_string(), result, None);

        assert_eq!(cache.size(), 2);

        cache.clear();
        assert_eq!(cache.size(), 0);
    }

    #[test]
    fn test_lru_eviction() {
        let cache = QueryResultCache::new(3, 60); // Max 3 entries
        let result = create_test_result();

        // Fill cache
        cache.put("key1".to_string(), result.clone(), None);
        std::thread::sleep(Duration::from_millis(10));
        cache.put("key2".to_string(), result.clone(), None);
        std::thread::sleep(Duration::from_millis(10));
        cache.put("key3".to_string(), result.clone(), None);

        assert_eq!(cache.size(), 3);

        // Add one more - should evict key1 (oldest)
        std::thread::sleep(Duration::from_millis(10));
        cache.put("key4".to_string(), result, None);

        assert_eq!(cache.size(), 3);
        assert!(cache.get("key1").is_none()); // Evicted
        assert!(cache.get("key2").is_some());
        assert!(cache.get("key3").is_some());
        assert!(cache.get("key4").is_some());

        let stats = cache.get_stats();
        assert_eq!(stats.evictions, 1);
    }
}
