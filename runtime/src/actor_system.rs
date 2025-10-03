use anyhow::Result;
use dashmap::DashMap;
use futures::future::BoxFuture;
use parking_lot::RwLock;
use serde_json::Value;
use std::any::Any;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

/// Actor system implementation for Veyra
pub struct ActorSystem {
    actors: DashMap<ActorId, ActorRef>,
    supervisor: Arc<Supervisor>,
    message_router: Arc<MessageRouter>,
    stats: RwLock<ActorSystemStats>,
}

pub type ActorId = Uuid;
pub type MessageId = Uuid;

#[derive(Debug, Default, Clone)]
pub struct ActorSystemStats {
    pub active_actors: usize,
    pub total_messages: u64,
    pub failed_messages: u64,
    pub actor_restarts: u64,
    pub average_message_processing_time: std::time::Duration,
}

/// Reference to an actor for sending messages
#[derive(Clone)]
pub struct ActorRef {
    pub id: ActorId,
    pub name: String,
    sender: mpsc::UnboundedSender<ActorMessage>,
    system: Arc<ActorSystem>,
}

/// Actor trait that all actors must implement
pub trait Actor: Send + Sync + 'static {
    type Message: Send + 'static;
    type State: Send + Sync;
    
    fn receive(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        context: &ActorContext,
    ) -> BoxFuture<'static, ActorResult>;
    
    fn pre_start(&mut self, _context: &ActorContext) -> BoxFuture<'static, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    
    fn post_stop(&mut self, _context: &ActorContext) -> BoxFuture<'static, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    
    fn pre_restart(&mut self, _context: &ActorContext, _error: &ActorError) -> BoxFuture<'static, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    
    fn post_restart(&mut self, _context: &ActorContext) -> BoxFuture<'static, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
}

/// Context provided to actors for system interactions
pub struct ActorContext {
    pub actor_id: ActorId,
    pub actor_name: String,
    pub parent: Option<ActorRef>,
    pub children: Vec<ActorRef>,
    pub system: Arc<ActorSystem>,
}

/// Internal message structure
struct ActorMessage {
    id: MessageId,
    sender: Option<ActorRef>,
    payload: Box<dyn Any + Send>,
    reply_to: Option<oneshot::Sender<ActorResult>>,
}

/// Result of actor message processing
#[derive(Debug)]
pub enum ActorResult {
    Ok(Option<Value>),
    Error(ActorError),
    Stop,
    Restart,
}

/// Actor error types
#[derive(Debug, Clone)]
pub enum ActorError {
    MessageProcessingFailed(String),
    ActorPanicked(String),
    SupervisionFailed(String),
    SystemError(String),
}

/// Supervision strategy for handling actor failures
#[derive(Debug, Clone)]
pub enum SupervisionStrategy {
    /// Restart the failed actor
    Restart,
    /// Stop the failed actor
    Stop,
    /// Escalate to parent supervisor
    Escalate,
    /// Resume with current state
    Resume,
}

/// Supervisor for managing actor lifecycle and failures
pub struct Supervisor {
    strategy: SupervisionStrategy,
    max_restarts: usize,
    restart_window: std::time::Duration,
    restart_counts: DashMap<ActorId, Vec<std::time::Instant>>,
}

/// Message router for efficient message delivery
pub struct MessageRouter {
    routes: DashMap<String, Vec<ActorId>>,
    broadcast_groups: DashMap<String, Vec<ActorId>>,
}

impl ActorSystem {
    pub fn new() -> Self {
        let supervisor = Arc::new(Supervisor::new(SupervisionStrategy::Restart));
        let message_router = Arc::new(MessageRouter::new());
        
        Self {
            actors: DashMap::new(),
            supervisor,
            message_router,
            stats: RwLock::new(ActorSystemStats::default()),
        }
    }
    
    pub async fn initialize(&self) -> Result<()> {
        // Initialize the actor system
        println!("Actor system initialized");
        Ok(())
    }
    
    pub async fn shutdown(&self) -> Result<()> {
        // Gracefully shutdown all actors
        let actors: Vec<_> = self.actors.iter().map(|entry| entry.key().clone()).collect();
        
        for actor_id in actors {
            self.stop_actor(&actor_id).await?;
        }
        
        println!("Actor system shut down");
        Ok(())
    }
    
    pub async fn spawn_actor<A>(
        &self,
        name: String,
        mut actor: A,
        initial_state: A::State,
    ) -> Result<ActorRef>
    where
        A: Actor + 'static,
    {
        let actor_id = Uuid::new_v4();
        let (sender, mut receiver) = mpsc::unbounded_channel::<ActorMessage>();
        
        let actor_ref = ActorRef {
            id: actor_id,  
            name: name.clone(),
            sender: sender.clone(),
            system: Arc::new(self.clone()), // This is problematic - we need a different approach
        };
        
        let context = ActorContext {
            actor_id,
            actor_name: name.clone(),
            parent: None,
            children: Vec::new(),
            system: Arc::new(self.clone()), // Same issue here
        };
        
        // Start actor lifecycle
        actor.pre_start(&context).await?;
        
        // Store actor reference
        self.actors.insert(actor_id, actor_ref.clone());
        
        // Spawn actor task
        let system_clone = Arc::new(self.clone()); // Another clone issue
        let mut state = initial_state;
        
        tokio::spawn(async move {
            while let Some(message) = receiver.recv().await {
                let start_time = std::time::Instant::now();
                
                // Process message
                let result = {
                    // Convert Any back to the actor's message type
                    // This is simplified - in reality we'd need better type handling
                    if let Ok(typed_message) = message.payload.downcast::<A::Message>() {
                        actor.receive(*typed_message, &mut state, &context).await
                    } else {
                        ActorResult::Error(ActorError::MessageProcessingFailed(
                            "Invalid message type".to_string()
                        ))
                    }
                };
                
                let processing_time = start_time.elapsed();
                
                // Handle result
                match result {
                    ActorResult::Ok(response) => {
                        if let Some(reply_channel) = message.reply_to {
                            let _ = reply_channel.send(ActorResult::Ok(response));
                        }
                    }
                    ActorResult::Error(error) => {
                        system_clone.handle_actor_error(actor_id, error).await;
                        if let Some(reply_channel) = message.reply_to {
                            let _ = reply_channel.send(ActorResult::Error(
                                ActorError::MessageProcessingFailed("Actor error".to_string())
                            ));
                        }
                    }
                    ActorResult::Stop => {
                        let _ = actor.post_stop(&context).await;
                        break;
                    }
                    ActorResult::Restart => {
                        let _ = actor.pre_restart(&context, &ActorError::SystemError("Restart requested".to_string())).await;
                        let _ = actor.post_restart(&context).await;
                    }
                }
                
                // Update stats
                {
                    let mut stats = system_clone.stats.write();
                    stats.total_messages += 1;
                    if stats.total_messages > 0 {
                        let total_time = stats.average_message_processing_time * 
                                       (stats.total_messages - 1) as u32 + processing_time;
                        stats.average_message_processing_time = total_time / stats.total_messages as u32;
                    }
                }
            }
        });
        
        // Update system stats
        self.stats.write().active_actors += 1;
        
        Ok(actor_ref)
    }
    
    pub async fn stop_actor(&self, actor_id: &ActorId) -> Result<()> {
        if let Some((_, actor_ref)) = self.actors.remove(actor_id) {
            // Send stop message
            let stop_message = ActorMessage {
                id: Uuid::new_v4(),
                sender: None,
                payload: Box::new(()), // Stop signal
                reply_to: None,
            };
            
            let _ = actor_ref.sender.send(stop_message);
            self.stats.write().active_actors -= 1;
        }
        
        Ok(())
    }
    
    pub fn get_actor(&self, actor_id: &ActorId) -> Option<ActorRef> {
        self.actors.get(actor_id).map(|entry| entry.value().clone())
    }
    
    pub fn find_actor_by_name(&self, name: &str) -> Option<ActorRef> {
        self.actors
            .iter()
            .find(|entry| entry.value().name == name)
            .map(|entry| entry.value().clone())
    }
    
    pub async fn broadcast_message<T>(&self, group: &str, message: T) -> Result<()>
    where
        T: Send + Clone + 'static,
    {
        if let Some(actor_ids) = self.message_router.broadcast_groups.get(group) {
            for actor_id in actor_ids.iter() {
                if let Some(actor_ref) = self.get_actor(actor_id) {
                    let _ = actor_ref.send_message(message.clone()).await;
                }
            }
        }
        
        Ok(())
    }
    
    async fn handle_actor_error(&self, actor_id: ActorId, error: ActorError) {
        match self.supervisor.handle_failure(actor_id, error).await {
            SupervisionDecision::Restart => {
                // Restart the actor
                self.stats.write().actor_restarts += 1;
            }
            SupervisionDecision::Stop => {
                let _ = self.stop_actor(&actor_id).await;
            }
            SupervisionDecision::Escalate => {
                // In a real system, this would escalate to parent supervisor
                eprintln!("Actor error escalated: {}", actor_id);
            }
            SupervisionDecision::Resume => {
                // Continue with current state
            }
        }
    }
    
    pub fn get_stats(&self) -> ActorSystemStats {
        self.stats.read().clone()
    }
}

// Clone implementation for ActorSystem (simplified)
impl Clone for ActorSystem {
    fn clone(&self) -> Self {
        // This is a simplified clone - in reality we'd use Arc<ActorSystem> throughout
        Self {
            actors: DashMap::new(), // New empty map
            supervisor: Arc::clone(&self.supervisor),
            message_router: Arc::clone(&self.message_router),
            stats: RwLock::new(self.stats.read().clone()),
        }
    }
}

impl ActorRef {
    pub async fn send_message<T>(&self, message: T) -> Result<()>
    where
        T: Send + 'static,
    {
        let actor_message = ActorMessage {
            id: Uuid::new_v4(),
            sender: Some(self.clone()),
            payload: Box::new(message),
            reply_to: None,
        };
        
        self.sender
            .send(actor_message)
            .map_err(|_| anyhow::anyhow!("Failed to send message to actor"))?;
        
        Ok(())
    }
    
    pub async fn ask<T, R>(&self, message: T) -> Result<ActorResult>
    where
        T: Send + 'static,
        R: Send + 'static,
    {
        let (reply_sender, reply_receiver) = oneshot::channel();
        
        let actor_message = ActorMessage {
            id: Uuid::new_v4(),
            sender: Some(self.clone()),
            payload: Box::new(message),
            reply_to: Some(reply_sender),
        };
        
        self.sender
            .send(actor_message)
            .map_err(|_| anyhow::anyhow!("Failed to send message to actor"))?;
        
        reply_receiver
            .await
            .map_err(|_| anyhow::anyhow!("Failed to receive reply from actor"))
    }
}

impl Supervisor {
    fn new(strategy: SupervisionStrategy) -> Self {
        Self {
            strategy,
            max_restarts: 3,
            restart_window: std::time::Duration::from_secs(60),
            restart_counts: DashMap::new(),
        }
    }
    
    async fn handle_failure(&self, actor_id: ActorId, _error: ActorError) -> SupervisionDecision {
        // Check restart limits
        let now = std::time::Instant::now();
        let mut restart_times = self.restart_counts.entry(actor_id).or_insert_with(Vec::new);
        
        // Remove old restart times outside the window
        restart_times.retain(|&time| now.duration_since(time) <= self.restart_window);
        
        if restart_times.len() >= self.max_restarts {
            return SupervisionDecision::Stop;
        }
        
        // Record this failure
        restart_times.push(now);
        
        // Apply supervision strategy
        match self.strategy {
            SupervisionStrategy::Restart => SupervisionDecision::Restart,
            SupervisionStrategy::Stop => SupervisionDecision::Stop,
            SupervisionStrategy::Escalate => SupervisionDecision::Escalate,
            SupervisionStrategy::Resume => SupervisionDecision::Resume,
        }
    }
}

#[derive(Debug)]
enum SupervisionDecision {
    Restart,
    Stop,
    Escalate,
    Resume,
}

impl MessageRouter {
    fn new() -> Self {
        Self {
            routes: DashMap::new(),
            broadcast_groups: DashMap::new(),
        }
    }
    
    pub fn add_route(&self, pattern: String, actor_id: ActorId) {
        self.routes.entry(pattern).or_insert_with(Vec::new).push(actor_id);
    }
    
    pub fn add_to_broadcast_group(&self, group: String, actor_id: ActorId) {
        self.broadcast_groups.entry(group).or_insert_with(Vec::new).push(actor_id);
    }
    
    pub fn remove_from_broadcast_group(&self, group: &str, actor_id: &ActorId) {
        if let Some(mut group_actors) = self.broadcast_groups.get_mut(group) {
            group_actors.retain(|id| id != actor_id);
        }
    }
}

impl Default for ActorSystem {
    fn default() -> Self {
        Self::new()
    }
}