// Standard Library Imports
// (None currently)

// External Crate Imports
use bevy::prelude::*;
use spacetimedb::ReducerContext;
use bevy::ecs::prelude::Resource;
use once_cell::sync::Lazy;
use spin::Mutex;

// --- Public API: Types and Components ---

/// Re-export Bevy's Component trait for convenience.
pub use bevy::ecs::component::Component;

/// Component used to link a Bevy `Entity` to its corresponding primary key
/// in a SpacetimeDB table (e.g., `Entity::entity_id`).
///
/// This example uses `u32`, assuming the SpacetimeDB primary key is `u32`.
/// It could be made generic or use a different type if needed.
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpacetimeId(pub u32);

/// Type alias for a closure intended to be executed within a SpacetimeDB reducer context.
///
/// This allows Bevy systems to queue database operations.
/// - `Ok(Some(new_id))`: Indicates a successful operation that created a new SpacetimeDB entity
///   with the given `new_id`. The integration layer might use this to link entities.
/// - `Ok(None)`: Indicates a successful operation that didn't create a new entity (e.g., an update).
/// - `Err(String)`: Indicates the operation failed.
pub type DbCommandClosure = Box<dyn FnOnce(&ReducerContext) -> Result<Option<u32>, String> + Send + Sync>;

/// Internal representation of a command queued from Bevy to be executed in SpacetimeDB.
/// Currently, only supports executing closures.
pub enum DbCommand {
    ExecuteClosure(DbCommandClosure),
}

/// Bevy `Resource` that acts as a queue for `DbCommand` instances.
/// Bevy systems add commands to this queue, and they are processed
/// within a SpacetimeDB reducer context (typically `process_tick`).
#[derive(Resource, Default)]
pub struct CommandQueue(pub Vec<DbCommand>);

/// Trait for actions that need to be executed on the Bevy `World` from outside
/// the main Bevy schedule, typically queued from SpacetimeDB reducers.
///
/// This allows SpacetimeDB logic to trigger actions within the Bevy environment
/// (e.g., sending events, modifying resources) in a thread-safe manner.
pub trait BevyWorldAction: Send + Sync {
    /// Executes the action against the provided Bevy `World`.
    fn execute(&self, world: &mut World);
}

// --- Internal State ---

// Static, mutex-protected buffer holding `BevyWorldAction`s queued from SpacetimeDB reducers.
// These actions are processed before the next Bevy `App::update()` call.
static PENDING_BEVY_ACTIONS: Lazy<Mutex<Vec<Box<dyn BevyWorldAction>>>> =
    Lazy::new(|| Mutex::new(Vec::new()));

// Placeholder component used to ensure the first spawned Bevy entity has index 1,
// matching typical database auto-increment starting points.
#[derive(Component)]
struct Placeholder;

// Concrete implementation of `BevyWorldAction` for sending a Bevy `Event`.
struct SendBevyEvent<T: Event + Clone + Send + Sync>(T);

// --- Bevy Plugin ---

/// The main Bevy `Plugin` for integrating with a SpacetimeDB server module.
///
/// Initializes necessary resources like the `CommandQueue` and adds
/// systems required for the integration.
pub struct SpacetimeDbServerPlugin;

impl Plugin for SpacetimeDbServerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CommandQueue>()
           // Add a system to spawn a placeholder entity at startup.
           .add_systems(PreStartup, add_single_entity_system);
        log::info!("SpacetimeDbServerPlugin initialized: CommandQueue resource added.");
    }
}

// --- Bevy Systems ---

/// System run during `PreStartup` to spawn a single placeholder entity.
/// This ensures subsequent Bevy-managed entities start with index 1,
/// aligning potentially with SpacetimeDB primary key generation.
pub fn add_single_entity_system(mut commands: Commands) {
    commands.spawn(Placeholder);
    log::trace!("Placeholder entity spawned to ensure entity IDs start from 1.");
}

// --- Public API: Functions ---

/// Executes a single update cycle of the provided Bevy `App`.
/// This should typically be called from the SpacetimeDB tick reducer.
pub fn run_bevy_update(app: &mut App) -> Result<(), String> {
    app.update();
    Ok(())
}

/// Processes all commands currently in the `CommandQueue` resource.
///
/// This function takes commands queued by Bevy systems and executes their
/// associated closures within the provided SpacetimeDB `ReducerContext`.
/// It requires mutable access to the Bevy `App` to retrieve the queue.
/// This should typically be called from the SpacetimeDB tick reducer after `run_bevy_update`.
pub fn process_bevy_commands(app: &mut App, ctx: &ReducerContext) -> Result<(), String> {
    // Extract commands from the queue within the app's world.
    let commands_to_process: Vec<DbCommand> = {
        let mut command_queue = app.world_mut().resource_mut::<CommandQueue>();
        // Drain the queue to take ownership of the commands.
        command_queue.0.drain(..).collect()
    };

    if commands_to_process.is_empty() {
        return Ok(()); // No commands to process.
    }

    log::debug!("Processing {} Bevy->SpacetimeDB commands...", commands_to_process.len());
    for command in commands_to_process {
        match command {
            DbCommand::ExecuteClosure(closure) => {
                log::trace!("Executing SpacetimeDB command closure...");
                match closure(ctx) {
                    Ok(Some(new_entity_id)) => {
                        // A new entity was created in SpacetimeDB.
                        // Future enhancement: Could potentially queue a Bevy action
                        // here to add the SpacetimeId component to the corresponding Bevy entity.
                        log::trace!("Closure reported SpacetimeDB spawn with ID: {}", new_entity_id);
                    }
                    Ok(None) => {
                        // Closure executed successfully, no new entity reported.
                        log::trace!("SpacetimeDB command closure executed successfully (no spawn reported).");
                    }
                    Err(e) => {
                        // The closure returned an error.
                        log::error!("Error executing SpacetimeDB command closure: {}", e);
                        // Consider whether one error should halt processing of subsequent commands.
                        // return Err(e); // Uncomment to halt on first error.
                    }
                }
            }
        }
    }
    log::debug!("Finished processing Bevy->SpacetimeDB commands.");
    Ok(())
}

/// Schedules a `BevyWorldAction` to be executed on the Bevy `World`
/// before the next `run_bevy_update` call.
///
/// This function is designed to be called safely from SpacetimeDB reducers
/// (which run on different threads/contexts than the main Bevy loop).
/// It pushes the action onto the `PENDING_BEVY_ACTIONS` static queue.
pub fn schedule_bevy_action(action: Box<dyn BevyWorldAction>) {
    log::trace!("Scheduling a BevyWorldAction.");
    PENDING_BEVY_ACTIONS.lock().push(action);
}

/// Processes all pending actions stored in the `PENDING_BEVY_ACTIONS` queue.
///
/// This function should be called within the SpacetimeDB tick reducer context,
/// *before* calling `run_bevy_update`. It requires mutable access to the
/// Bevy `App` to get access to the `World`.
/// It drains the queue and executes each action against the `World`.
pub fn process_bevy_actions(app: &mut App) {
    let actions_to_process: Vec<Box<dyn BevyWorldAction>> = {
        // Attempt to acquire the lock non-blockingly.
        // In the unlikely event of contention (e.g., if called concurrently),
        // skip processing for this tick to avoid deadlocks.
        match PENDING_BEVY_ACTIONS.try_lock() {
            Some(mut guard) => guard.drain(..).collect(),
            None => {
                log::warn!("Could not acquire PENDING_BEVY_ACTIONS lock; skipping SpacetimeDB->Bevy action processing this tick.");
                return; // Skip processing if lock is busy.
            }
        }
    };

    if actions_to_process.is_empty() {
        return; // No actions scheduled.
    }

    log::debug!("Processing {} SpacetimeDB->Bevy actions...", actions_to_process.len());

    // Get mutable access to the world once for efficiency.
    let world = app.world_mut();
    for action in actions_to_process {
        log::trace!("Executing scheduled BevyWorldAction.");
        action.execute(world); // Execute the action (e.g., world.send_event).
    }
    log::debug!("Finished processing SpacetimeDB->Bevy actions.");
}

/// Helper function to create a boxed `BevyWorldAction` specifically for sending a Bevy `Event`.
/// The event type `T` must implement `Event`, `Clone`, `Send`, `Sync`, and be `'static`.
pub fn create_send_event_action<T: Event + Clone + Send + Sync + 'static>(event: T) -> Box<dyn BevyWorldAction> {
    log::trace!("Creating SendBevyEvent action for event type: {}", std::any::type_name::<T>());
    Box::new(SendBevyEvent(event))
}

// --- Trait Implementations ---

impl<T: Event + Clone + Send + Sync> BevyWorldAction for SendBevyEvent<T> {
    fn execute(&self, world: &mut World) {
        // Clone the event data to send it into the Bevy event system.
        world.send_event(self.0.clone());
        log::trace!("Executed SendBevyEvent action (sent event).");
    }
}

// ------------------------------ 