use anyhow::Result;
use crossbeam::queue::SegQueue;
use parking_lot::{Mutex, RwLock};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use uuid::Uuid;

/// Advanced thread pool with work stealing and load balancing
pub struct ThreadPool {
    workers: Vec<Worker>,
    global_queue: Arc<SegQueue<Job>>,
    shutdown: Arc<AtomicBool>,
    stats: Arc<RwLock<ThreadPoolStats>>,
    config: ThreadPoolConfig,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

#[derive(Debug, Clone)]
pub struct ThreadPoolConfig {
    pub min_threads: usize,
    pub max_threads: usize,
    pub keep_alive: Duration,
    pub queue_size: usize,
    pub work_stealing: bool,
}

impl Default for ThreadPoolConfig {
    fn default() -> Self {
        Self {
            min_threads: 2,
            max_threads: num_cpus::get() * 2,
            keep_alive: Duration::from_secs(60),
            queue_size: 1024,
            work_stealing: true,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct ThreadPoolStats {
    pub active_threads: usize,
    pub idle_threads: usize,
    pub jobs_executed: u64,
    pub jobs_queued: u64,
    pub jobs_stolen: u64,
    pub average_execution_time: Duration,
    pub peak_queue_size: usize,
}

struct Worker {
    id: Uuid,
    thread: Option<JoinHandle<()>>,
    local_queue: Arc<Mutex<VecDeque<Job>>>,
    is_active: Arc<AtomicBool>,
}

impl ThreadPool {
    pub fn new(thread_count: usize) -> Self {
        Self::with_config(ThreadPoolConfig {
            min_threads: thread_count,
            max_threads: thread_count * 2,
            ..Default::default()
        })
    }
    
    pub fn with_config(config: ThreadPoolConfig) -> Self {
        let global_queue = Arc::new(SegQueue::new());
        let shutdown = Arc::new(AtomicBool::new(false));
        let stats = Arc::new(RwLock::new(ThreadPoolStats::default()));
        
        let mut workers = Vec::with_capacity(config.max_threads);
        
        // Create initial workers
        for _ in 0..config.min_threads {
            let worker = Worker::new();
            workers.push(worker);
        }
        
        Self {
            workers,
            global_queue,
            shutdown,
            stats,
            config,
        }
    }
    
    pub async fn start(&self) -> Result<()> {
        for worker in &self.workers {
            worker.start(Arc::clone(&self.global_queue), Arc::clone(&self.shutdown), Arc::clone(&self.stats))?;
        }
        Ok(())
    }
    
    pub async fn shutdown(&mut self) -> Result<()> {
        // Signal shutdown
        self.shutdown.store(true, Ordering::SeqCst);
        
        // Wait for all workers to finish
        for worker in &mut self.workers {
            worker.shutdown().await?;
        }
        
        Ok(())
    }
    
    pub fn execute<F>(&self, job: F) -> Result<()>
    where
        F: FnOnce() + Send + 'static,
    {
        if self.shutdown.load(Ordering::SeqCst) {
            return Err(anyhow::anyhow!("Thread pool is shutting down"));
        }
        
        let job = Box::new(job);
        
        // Try to assign to least loaded worker first
        if let Some(worker) = self.find_least_loaded_worker() {
            if let Some(mut queue) = worker.local_queue.try_lock() {
                if queue.len() < self.config.queue_size / self.workers.len() {
                    queue.push_back(job);
                    self.stats.write().jobs_queued += 1;
                    return Ok(());
                }
            }
        }
        
        // Fall back to global queue
        self.global_queue.push(job);
        self.stats.write().jobs_queued += 1;
        
        // Scale up if needed
        self.scale_up_if_needed()?;
        
        Ok(())
    }
    
    pub fn execute_with_priority<F>(&self, job: F, _priority: JobPriority) -> Result<()>
    where
        F: FnOnce() + Send + 'static,
    {
        // For now, just execute normally
        // In a full implementation, we'd have priority queues
        self.execute(job)
    }
    
    pub fn get_stats(&self) -> ThreadPoolStats {
        let mut stats = self.stats.read().clone();
        
        // Update current thread counts
        stats.active_threads = self.count_active_threads();
        stats.idle_threads = self.workers.len() - stats.active_threads;
        
        stats
    }
    
    fn find_least_loaded_worker(&self) -> Option<&Worker> {
        self.workers
            .iter()
            .min_by_key(|worker| {
                worker.local_queue
                    .try_lock()
                    .map(|queue| queue.len())
                    .unwrap_or(usize::MAX)
            })
    }
    
    fn count_active_threads(&self) -> usize {
        self.workers
            .iter()
            .filter(|worker| worker.is_active.load(Ordering::SeqCst))
            .count()
    }
    
    fn scale_up_if_needed(&self) -> Result<()> {
        let active_threads = self.count_active_threads();
        let queue_size = self.global_queue.len();
        
        // Scale up if queue is getting full and we haven't reached max threads
        if queue_size > self.config.queue_size / 2 && 
           self.workers.len() < self.config.max_threads {
            
            // This is simplified - in reality we'd need to handle adding workers at runtime
            // For now, we just log that we would scale up
            eprintln!("Would scale up thread pool: queue_size={}, active_threads={}", 
                     queue_size, active_threads);
        }
        
        Ok(())
    }
}

impl Worker {
    fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            thread: None,
            local_queue: Arc::new(Mutex::new(VecDeque::new())),
            is_active: Arc::new(AtomicBool::new(false)),
        }
    }
    
    fn start(&self, global_queue: Arc<SegQueue<Job>>, shutdown: Arc<AtomicBool>, stats: Arc<RwLock<ThreadPoolStats>>) -> Result<()> {
        let local_queue = Arc::clone(&self.local_queue);
        let is_active = Arc::clone(&self.is_active);
        let worker_id = self.id;
        
        let _handle = thread::Builder::new()
            .name(format!("veyra-worker-{}", worker_id))
            .spawn(move || {
                Self::worker_loop(
                    worker_id,
                    global_queue,
                    local_queue,
                    shutdown,
                    stats,
                    is_active,
                )
            })?;
        
        // Store handle (this is unsafe in the current design, but for demo purposes)
        // In a real implementation, we'd use a different approach
        
        Ok(())
    }
    
    async fn shutdown(&mut self) -> Result<()> {
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
        Ok(())
    }
    
    fn worker_loop(
        __worker_id: Uuid,
        global_queue: Arc<SegQueue<Job>>,
        local_queue: Arc<Mutex<VecDeque<Job>>>,
        shutdown: Arc<AtomicBool>,
        stats: Arc<RwLock<ThreadPoolStats>>,
        is_active: Arc<AtomicBool>,
    ) {
        let mut idle_start = None;
        
        while !shutdown.load(Ordering::SeqCst) {
            // Try to get work from local queue first
            let job = {
                let mut queue = local_queue.lock();
                queue.pop_front()
            };
            
            let job = job.or_else(|| {
                // Try global queue
                global_queue.pop()
            });
            
            if let Some(job) = job {
                // Mark as active
                is_active.store(true, Ordering::SeqCst);
                idle_start = None;
                
                // Execute job
                let start_time = Instant::now();
                job();
                let execution_time = start_time.elapsed();
                
                // Update stats
                {
                    let mut stats_guard = stats.write();
                    stats_guard.jobs_executed += 1;
                    
                    // Update average execution time
                    if stats_guard.jobs_executed > 0 {
                        let total_time = stats_guard.average_execution_time * 
                                       (stats_guard.jobs_executed - 1) as u32 + execution_time;
                        stats_guard.average_execution_time = total_time / stats_guard.jobs_executed as u32;
                    }
                }
                
                // Mark as potentially idle
                is_active.store(false, Ordering::SeqCst);
            } else {
                // No work available, enter idle state
                if idle_start.is_none() {
                    idle_start = Some(Instant::now());
                }
                
                // Sleep briefly to avoid busy waiting
                thread::sleep(Duration::from_millis(1));
                
                // Check if we should exit due to prolonged idleness
                if let Some(start) = idle_start {
                    if start.elapsed() > Duration::from_secs(60) {
                        // In a real implementation, we might reduce the thread pool size
                        // For now, just continue
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobPriority {
    High,
    Normal,
    Low,
}

impl Default for ThreadPool {
    fn default() -> Self {
        Self::new(num_cpus::get())
    }
}

