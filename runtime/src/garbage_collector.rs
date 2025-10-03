use anyhow::Result;
use parking_lot::RwLock;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::task::JoinHandle;
use tokio::time::interval;

use crate::memory_manager::{MemoryManager, ObjectRef};

/// Garbage collector with mark-and-sweep and generational collection
pub struct GarbageCollector {
    memory_manager: Arc<MemoryManager>,
    roots: RwLock<HashSet<ObjectRef>>,
    collection_thread: RwLock<Option<JoinHandle<()>>>,
    stats: RwLock<GcStats>,
    config: GcConfig,
}

#[derive(Debug, Clone)]
pub struct GcConfig {
    /// Interval between garbage collection cycles
    pub collection_interval: Duration,
    /// Memory pressure threshold to trigger collection
    pub memory_threshold: usize,
    /// Enable generational collection
    pub generational: bool,
    /// Number of generations for generational GC
    pub generations: usize,
}

impl Default for GcConfig {
    fn default() -> Self {
        Self {
            collection_interval: Duration::from_millis(100),
            memory_threshold: 64 * 1024 * 1024, // 64MB
            generational: true,
            generations: 3,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GcStats {
    pub collections: u64,
    pub objects_collected: u64,
    pub bytes_collected: u64,
    pub collection_time: Duration,
    pub last_collection: Option<Instant>,
}

impl GarbageCollector {
    pub fn new(memory_manager: Arc<MemoryManager>) -> Self {
        Self {
            memory_manager,
            roots: RwLock::new(HashSet::new()),
            collection_thread: RwLock::new(None),
            stats: RwLock::new(GcStats::default()),
            config: GcConfig::default(),
        }
    }
    
    pub fn with_config(memory_manager: Arc<MemoryManager>, config: GcConfig) -> Self {
        Self {
            memory_manager,
            roots: RwLock::new(HashSet::new()),
            collection_thread: RwLock::new(None),
            stats: RwLock::new(GcStats::default()),
            config,
        }
    }
    
    pub async fn start(&self) -> Result<()> {
        let memory_manager = Arc::clone(&self.memory_manager);
        let config = self.config.clone();
        let stats = Arc::new(RwLock::new(GcStats::default()));
        
        let handle = tokio::spawn(async move {
            let mut interval = interval(config.collection_interval);
            
            loop {
                interval.tick().await;
                
                // Check if collection is needed
                let memory_usage = memory_manager.get_memory_usage().await;
                if memory_usage.total_allocated > config.memory_threshold {
                    let start_time = Instant::now();
                    
                    // Perform collection
                    if let Ok(collected) = Self::collect_garbage(&memory_manager, &config).await {
                        let collection_time = start_time.elapsed();
                        
                        // Update stats
                        let mut stats_guard = stats.write();
                        stats_guard.collections += 1;
                        stats_guard.objects_collected += collected.objects as u64;
                        stats_guard.bytes_collected += collected.bytes as u64;
                        stats_guard.collection_time += collection_time;
                        stats_guard.last_collection = Some(start_time);
                    }
                }
            }
        });
        
        *self.collection_thread.write() = Some(handle);
        Ok(())
    }
    
    pub async fn stop(&self) -> Result<()> {
        if let Some(handle) = self.collection_thread.write().take() {
            handle.abort();
            let _ = handle.await;
        }
        Ok(())
    }
    
    pub fn add_root(&self, object: ObjectRef) {
        self.roots.write().insert(object);
    }
    
    pub fn remove_root(&self, object: &ObjectRef) {
        self.roots.write().remove(object);
    }
    
    pub async fn force_collection(&self) -> Result<CollectionResult> {
        Self::collect_garbage(&self.memory_manager, &self.config).await
    }
    
    async fn collect_garbage(
        memory_manager: &MemoryManager,
        config: &GcConfig,
    ) -> Result<CollectionResult> {
        if config.generational {
            Self::generational_collect(memory_manager, config).await
        } else {
            Self::mark_and_sweep(memory_manager).await
        }
    }
    
    async fn mark_and_sweep(memory_manager: &MemoryManager) -> Result<CollectionResult> {
        // Mark phase
        let mut marked = HashSet::new();
        let roots = memory_manager.get_roots().await;
        
        for root in roots {
            Self::mark_object(&mut marked, &root, memory_manager).await;
        }
        
        // Sweep phase
        let all_objects = memory_manager.get_all_objects().await;
        let mut collected_objects = 0;
        let mut collected_bytes = 0;
        
        for object_ref in all_objects {
            if !marked.contains(&object_ref) {
                let size = memory_manager.get_object_size(&object_ref).await;
                memory_manager.deallocate_object(object_ref).await?;
                collected_objects += 1;
                collected_bytes += size;
            }
        }
        
        Ok(CollectionResult {
            objects: collected_objects,
            bytes: collected_bytes,
        })
    }
    
    async fn generational_collect(
        memory_manager: &MemoryManager,
        config: &GcConfig,
    ) -> Result<CollectionResult> {
        let mut total_result = CollectionResult { objects: 0, bytes: 0 };
        
        // Collect younger generations more frequently
        for generation in 0..config.generations {
            let result = Self::collect_generation(memory_manager, generation).await?;
            total_result.objects += result.objects;
            total_result.bytes += result.bytes;
            
            // Only collect older generations occasionally
            if generation > 0 && total_result.objects < 100 {
                break;
            }
        }
        
        Ok(total_result)
    }
    
    async fn collect_generation(
        memory_manager: &MemoryManager,
        generation: usize,
    ) -> Result<CollectionResult> {
        // Simplified generational collection
        // In a real implementation, this would track object ages
        // and collect objects based on their generation
        
        let objects_in_generation = memory_manager.get_objects_in_generation(generation).await;
        let mut marked = HashSet::new();
        
        // Mark objects reachable from older generations and roots
        let roots = memory_manager.get_roots().await;
        for root in roots {
            Self::mark_object(&mut marked, &root, memory_manager).await;
        }
        
        // Sweep unmarked objects in this generation
        let mut collected_objects = 0;
        let mut collected_bytes = 0;
        
        for object_ref in objects_in_generation {
            if !marked.contains(&object_ref) {
                let size = memory_manager.get_object_size(&object_ref).await;
                memory_manager.deallocate_object(object_ref).await?;
                collected_objects += 1;
                collected_bytes += size;
            }
        }
        
        Ok(CollectionResult {
            objects: collected_objects,
            bytes: collected_bytes,
        })
    }
    
    fn mark_object<'a>(
        marked: &'a mut HashSet<ObjectRef>,
        object_ref: &'a ObjectRef,
        memory_manager: &'a MemoryManager,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
        if marked.contains(object_ref) {
            return;
        }
        
        marked.insert(object_ref.clone());
        
            // Mark all objects referenced by this object
            if let Ok(references) = memory_manager.get_object_references(object_ref).await {
                for referenced in references {
                    Self::mark_object(marked, &referenced, memory_manager).await;
                }
            }
        })
    }
    
    pub fn get_stats(&self) -> GcStats {
        (*self.stats.read()).clone()
    }
}

#[derive(Debug, Clone)]
pub struct CollectionResult {
    pub objects: usize,
    pub bytes: usize,
}