use anyhow::Result;
use std::sync::Arc;

pub mod actor_system;
pub mod async_runtime;
pub mod garbage_collector;
pub mod jit_compiler;
pub mod memory_manager;
pub mod thread_pool;

// Re-export important types
pub use actor_system::*;
pub use async_runtime::*;
pub use garbage_collector::*;
pub use jit_compiler::*;
pub use memory_manager::*;
pub use thread_pool::*;

/// Main runtime system for Veyra
pub struct VeyraRuntime {
    pub memory_manager: Arc<MemoryManager>,
    pub garbage_collector: Arc<GarbageCollector>,
    pub async_runtime: Arc<AsyncRuntime>,
    pub thread_pool: Arc<ThreadPool>,
    pub actor_system: Arc<ActorSystem>,
    pub jit_compiler: Arc<JitCompiler>,
}

impl VeyraRuntime {
    pub fn new() -> Self {
        let memory_manager = Arc::new(MemoryManager::new());
        let garbage_collector = Arc::new(GarbageCollector::new(memory_manager.clone()));
        let async_runtime = Arc::new(AsyncRuntime::new());
        let thread_pool = Arc::new(ThreadPool::new(num_cpus::get()));
        let actor_system = Arc::new(ActorSystem::new());
        let jit_compiler = Arc::new(JitCompiler::new());

        Self {
            memory_manager,
            garbage_collector,
            async_runtime,
            thread_pool,
            actor_system,
            jit_compiler,
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        // Initialize all subsystems
        self.memory_manager.initialize().await?;
        self.garbage_collector.start().await?;
        self.async_runtime.initialize().await?;
        self.thread_pool.start().await?;
        self.actor_system.initialize().await?;
        self.jit_compiler.initialize().await?;

        Ok(())
    }

    pub async fn shutdown(&self) -> Result<()> {
        // Shutdown all subsystems in reverse order
        self.jit_compiler.shutdown().await?;
        self.actor_system.shutdown().await?;
        // Note: ThreadPool doesn't support mutable shutdown through Arc
        // In a real implementation, we'd use a different approach
        self.async_runtime.shutdown().await?;
        self.garbage_collector.stop().await?;
        self.memory_manager.shutdown().await?;

        Ok(())
    }
}

impl Default for VeyraRuntime {
    fn default() -> Self {
        Self::new()
    }
}
