// Public module declaration (if needed, e.g., for integration tests)
// pub mod bevy_logic; // Keep if used elsewhere, otherwise remove if logic is inlined

// Standard Library Imports
use std::cell::UnsafeCell;
use std::time::Duration;

// External Crate Imports
use bevy::app::ScheduleRunnerPlugin;
use bevy::ecs::event::EventReader;
use bevy::prelude::*;
use bevy::time::TimePlugin;
use once_cell::sync::Lazy;
use spacetimedb::{Identity, ReducerContext, ScheduleAt, SpacetimeType, Table};

// Workspace Crate Imports (Integration Library)
use bevy_spacetimedb_server::{
    create_send_event_action, process_bevy_actions, process_bevy_commands, run_bevy_update,
    schedule_bevy_action, CommandQueue, DbCommand, DbCommandClosure, SpacetimeDbServerPlugin,
};

// --- Global Static Bevy Application State ---

// Container to hold the Bevy App instance within a static context.
// Required because SpacetimeDB reducers (like `process_tick`) are called
// outside the main Bevy execution context but need access to the App/World.
// SAFETY: This relies on careful manual synchronization. Access is guarded by
// assumptions that `init` runs once before any `process_tick`, and `process_tick`
// calls are serialized by SpacetimeDB.
struct UnsafeAppContainer(UnsafeCell<Option<App>>);
unsafe impl Send for UnsafeAppContainer {}
unsafe impl Sync for UnsafeAppContainer {}

static BEVY_APP: Lazy<UnsafeAppContainer> = Lazy::new(|| UnsafeAppContainer(UnsafeCell::new(None)));

// --- SpacetimeDB Type Definitions ---

#[derive(SpacetimeType, Clone, Debug, Default, Copy)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

// --- SpacetimeDB Table Definitions ---

/// SpacetimeDB table used solely to trigger the `process_tick` reducer at regular intervals.
#[spacetimedb::table(name = scheduled_tick, scheduled(process_tick))]
pub struct ScheduledTick {
    #[primary_key]
    #[auto_inc]
    pub scheduled_id: u64,
    pub scheduled_at: ScheduleAt,
}

/// Represents a player or other dynamic object in the game world.
/// Marked `public` so clients can subscribe to it.
#[spacetimedb::table(name = entity, public)]
#[derive(Debug, Clone)]
pub struct Entity {
    #[primary_key]
    /// The unique ID for this entity. In this setup, it matches the Bevy `Entity::index()`.
    pub entity_id: u32,
    /// Current position in the game world.
    pub position: Vec2,
    /// The SpacetimeDB `Identity` of the client that owns/controls this entity.
    #[unique]
    pub owner_identity: Identity,
}

// --- Bevy Event Definitions ---

/// Bevy event triggered by the `enter_game` reducer to request spawning
/// a new player entity within the Bevy world.
#[derive(Debug, Clone, Event)]
pub struct InstantiateEntityEvent {
    pub owner_identity: Identity,
    pub position: Vec2,
}

/// Bevy event triggered by the `receive_player_input` reducer when a client
/// sends movement input.
#[derive(Debug, Clone, Event)]
pub struct PlayerInputEvent {
    /// The ID of the entity this input applies to.
    pub player_id: u32,
    /// The input direction vector.
    pub direction: Vec2,
}

// --- Bevy Component Definitions ---

/// Bevy component holding the position of an entity within the Bevy world.
/// This is kept separate from the SpacetimeDB `Entity` table to allow
/// Bevy systems to operate on position data directly.
#[derive(Component, Debug, Clone)] // Added Debug, Clone
pub struct Position(pub Vec2);

// --- SpacetimeDB Reducers ---

/// Reducer called once when the SpacetimeDB module initializes.
/// Sets up the Bevy App instance, schedules the tick, and registers Bevy systems/events.
#[spacetimedb::reducer(init)]
pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    log::info!("Initializing Spacetime Module and Bevy App...");

    // --- Bevy App Setup ---
    let mut app = App::new();
    app.add_plugins(
        // Use MinimalPlugins and disable features not needed for server-side logic.
        MinimalPlugins
            .build()
            .disable::<ScheduleRunnerPlugin>() // No need for Bevy to run its own schedule loop
            .disable::<TimePlugin>(), // SpacetimeDB handles time/ticks
    );
    // Add the integration plugin, which sets up the CommandQueue resource.
    app.add_plugins(SpacetimeDbServerPlugin);

    // Register Bevy events used for communication between STDB reducers and Bevy systems.
    app.add_event::<InstantiateEntityEvent>();
    app.add_event::<PlayerInputEvent>();

    // Add Bevy systems.
    app.add_systems(
        Update, // Run these systems during the Bevy App::update() cycle.
        (
            // System to handle InstantiateEntityEvent and queue STDB insertion.
            instantiate_entity_system,
            // Apply movement input to Bevy Position components.
            apply_player_movement_system,
            // Detect changes in Bevy Position and queue STDB updates.
            update_stdb_position_system,
        )
            // Define execution order: apply movement *before* checking for changes to update STDB.
            .chain(), // Apply .chain() for clear sequential ordering
    );

    // Store the configured Bevy App instance in the static variable.
    // SAFETY: This assumes `init` is called exactly once by SpacetimeDB.
    let app_ptr = BEVY_APP.0.get();
    unsafe {
        *app_ptr = Some(app);
    }
    log::info!("Bevy App initialized and stored globally.");

    // --- SpacetimeDB Initialization ---
    // Schedule the first tick. `process_tick` will be called repeatedly.
    ctx.db.scheduled_tick().try_insert(ScheduledTick {
        scheduled_id: 0, // Start ID at 0
        scheduled_at: ScheduleAt::Interval(Duration::from_millis(16).into()), // Approx 60 FPS
    })?;
    log::info!("Initial SpacetimeDB tick scheduled.");

    log::info!("Spacetime Module initialization complete.");
    Ok(())
}

/// Reducer called when a client connects.
#[spacetimedb::reducer(client_connected)]
pub fn connect(_ctx: &ReducerContext) -> Result<(), String> {
    // Placeholder: Add logic if needed when a client connects.
    Ok(())
}

/// Reducer called when a client disconnects.
#[spacetimedb::reducer(client_disconnected)]
pub fn disconnect(_ctx: &ReducerContext) -> Result<(), String> {
    // Placeholder: Add logic if needed, e.g., remove the player's entity.
    // Note: Need to find the entity associated with ctx.sender identity.
    Ok(())
}

/// The main integration point between SpacetimeDB's tick and Bevy's update cycle.
/// This reducer is scheduled to run at regular intervals by the `ScheduledTick` table.
#[spacetimedb::reducer]
pub fn process_tick(ctx: &ReducerContext, _tick: ScheduledTick) -> Result<(), String> {
    // Retrieve the Bevy App instance from the static storage.
    // SAFETY: Relies on `init` having run and `process_tick` being serialized.
    let app_ptr = BEVY_APP.0.get();
    let app = unsafe { (*app_ptr).as_mut() };

    if let Some(app) = app {
        // 1. Process Actions Queued from STDB -> Bevy:
        //    Execute any actions (like sending events) that were scheduled by reducers
        //    since the last tick using `schedule_bevy_action`.
        process_bevy_actions(app);

        // 2. Run Bevy's Update Cycle:
        //    Execute all Bevy systems scheduled for the `Update` stage.
        //    This includes systems that read events (like PlayerInputEvent)
        //    and modify Bevy components (like Position).
        if let Err(e) = run_bevy_update(app) {
            log::error!("Failed to run Bevy update cycle: {}", e);
            // Depending on the error, might want to return Err(e) here.
        }

        // 3. Process Commands Queued from Bevy -> STDB:
        //    Execute any SpacetimeDB operations (like table inserts/updates)
        //    that were queued by Bevy systems during the `run_bevy_update`
        //    using the `CommandQueue` resource.
        if let Err(e) = process_bevy_commands(app, ctx) {
            log::error!("Failed to process Bevy->SpacetimeDB commands: {}", e);
            // Depending on the error, might want to return Err(e) here.
        }
    } else {
        // This should not happen if `init` ran correctly.
        log::error!("Bevy App not initialized in process_tick. Cannot update.");
        return Err("Bevy App is not initialized".to_string());
    }

    Ok(())
}

/// Reducer called by a client to signal their intent to join the game.
#[spacetimedb::reducer]
pub fn enter_game(ctx: &ReducerContext, name: String) -> Result<(), String> {
    let owner_identity = ctx.sender; // Identify the client making the request.
    log::info!("Player '{}' ({:?}) requesting to enter game...", name, owner_identity);

    // Prevent duplicate entities for the same player.
    if ctx
        .db
        .entity()
        .iter()
        .any(|e| e.owner_identity == owner_identity)
    {
        log::warn!(
            "Player {:?} already has an entity. Ignoring enter_game request.",
            owner_identity
        );
        return Ok(());
    }

    // Instead of directly inserting into STDB, schedule a Bevy event.
    // The `instantiate_entity_system` will handle this event during the next Bevy update.
    let instantiate_event = InstantiateEntityEvent {
        owner_identity,
        position: Vec2 { x: 0.0, y: 0.0 }, // Initial position
    };
    let event_action = create_send_event_action(instantiate_event);

    // Use the safe scheduling mechanism from the integration library.
    schedule_bevy_action(event_action);
    log::trace!(
        "Scheduled InstantiateEntityEvent action for identity {:?}",
        owner_identity
    );

    Ok(())
}

/// Reducer called by a client to send movement input.
#[spacetimedb::reducer]
pub fn receive_player_input(ctx: &ReducerContext, x: f32, y: f32) -> Result<(), String> {
    let player_identity = ctx.sender;
    log::trace!(
        "Received input ({}, {}) from identity {:?}",
        x,
        y,
        player_identity
    );

    // Find the SpacetimeDB entity associated with the sending client.
    let entity = ctx
        .db
        .entity().owner_identity().find(player_identity);

    if entity.is_none() {
        log::warn!("Received input from identity {:?} which has no associated Entity.", player_identity);
        return Ok(());
    } 

    let player_entity_id = entity.unwrap().entity_id;

    // Schedule a Bevy event to handle the input within the Bevy world.
    // The `apply_player_movement_system` will process this.
    let input_event = PlayerInputEvent {
        player_id: player_entity_id,
        direction: Vec2 { x, y },
    };
    let event_action = create_send_event_action(input_event);

    schedule_bevy_action(event_action);
    log::trace!(
        "Scheduled PlayerInputEvent action for entity ID {}",
        player_entity_id
    );

    Ok(())
}

// --- Bevy Systems ---

/// Bevy system that processes `InstantiateEntityEvent`s.
/// It spawns a corresponding Bevy entity with a `Position` component
/// and queues a command to insert the entity data into the SpacetimeDB `Entity` table.
pub fn instantiate_entity_system(
    mut commands: Commands,
    mut events: EventReader<InstantiateEntityEvent>,
    mut command_queue: ResMut<CommandQueue>,
) {
    for event in events.read() {
        log::debug!("Processing InstantiateEntityEvent for {:?}", event.owner_identity);

        // 1. Spawn the Bevy entity with its initial position.
        let bevy_entity = commands
            .spawn(Position(event.position))
            // Consider adding the SpacetimeId component here if needed for lookups
            // .insert(SpacetimeId(bevy_entity.index()))
            .id();

        // Use the Bevy entity's index as the primary key for the SpacetimeDB table.
        // This provides a direct link between the Bevy entity and the STDB row.
        let new_entity_id = bevy_entity.index();

        // Clone data needed for the closure (moving `event` data into the closure).
        let position_to_insert = event.position;
        let owner_identity_to_insert = event.owner_identity;

        // 2. Queue a command to insert the entity into SpacetimeDB.
        // This closure will be executed later within the `process_tick` reducer context.
        let cmd: DbCommandClosure = Box::new(move |ctx| {
            log::info!(
                "Executing STDB insert for Bevy entity {}, owner {:?}",
                new_entity_id,
                owner_identity_to_insert
            );
            ctx.db.entity().try_insert(crate::Entity {
                entity_id: new_entity_id,
                position: position_to_insert,
                owner_identity: owner_identity_to_insert,
            })?;
            log::info!("Inserted STDB entity row with ID: {}", new_entity_id);
            // Return Ok(Some(new_entity_id)) if the integration layer needs to know the ID.
            Ok(None)
        });
        command_queue.0.push(DbCommand::ExecuteClosure(cmd));
        log::trace!("Queued STDB insert command for entity ID {}", new_entity_id);
    }
}

/// Bevy system that processes `PlayerInputEvent`s.
/// It finds the corresponding Bevy entity and updates its local `Position` component.
/// This system *only* modifies Bevy state.
pub fn apply_player_movement_system(
    mut events: EventReader<PlayerInputEvent>,
    // Query for Bevy entities that have a Position component.
    mut query: Query<(bevy::prelude::Entity, &mut Position)>,
) {
    for event in events.read() {
        log::trace!("Processing PlayerInputEvent for entity ID {}", event.player_id);
        // Iterate through Bevy entities with Position components.
        for (bevy_entity, mut position) in query.iter_mut() {
            // Match the Bevy entity's index with the ID from the event.
            if bevy_entity.index() == event.player_id {
                log::trace!("Applying movement to Bevy entity {}", bevy_entity.index());
                // Update the Bevy `Position` component directly.
                position.0.x += event.direction.x;
                position.0.y += event.direction.y;
                log::trace!("Updated Bevy Position for {}: {:?}", bevy_entity.index(), position.0);

                // Stop searching for this event, assuming one Bevy entity per player_id.
                break;
            }
        }
    }
}

/// Bevy system that detects changes in the `Position` component.
/// When a change is detected, it queues a command to update the corresponding
/// entity's position in the SpacetimeDB `Entity` table.
pub fn update_stdb_position_system(
    // Query for entities where the Position component has changed since the last update.
    query: Query<(bevy::prelude::Entity, &Position), Changed<Position>>,
    mut command_queue: ResMut<CommandQueue>,
) {
    for (bevy_entity, position) in query.iter() {
        // Get the ID (which matches the SpacetimeDB entity_id).
        let entity_id_to_update = bevy_entity.index();
        // Clone the current position value to move into the closure.
        let new_position = position.0;

        log::trace!(
            "Detected position change for Bevy entity {}, queuing STDB update.",
            entity_id_to_update
        );

        // Queue a command to update the SpacetimeDB table.
        let cmd: DbCommandClosure = Box::new(move |ctx| {
            log::trace!(
                "Executing STDB position update for entity {} to {:?}",
                entity_id_to_update,
                new_position
            );
            // Find the SpacetimeDB row by its primary key (entity_id).
            // Call .iter() first to get an iterator, then filter.
            if let Some(mut entity_row) = ctx
                .db
                .entity()
                .iter() // Get iterator first
                .filter(|e| e.entity_id == entity_id_to_update)
                .next()
            {
                // Update the position field.
                entity_row.position = new_position;
                // Apply the update to the database table using the PK index.
                ctx.db.entity().entity_id().update(entity_row);
                log::trace!("Updated STDB entity {} position.", entity_id_to_update);
            } else {
                // This might happen if the entity was deleted between the Bevy update and STDB update.
                log::warn!(
                    "Could not find SpacetimeDB entity {} to update position.",
                    entity_id_to_update
                );
            }
            Ok(None)
        });
        command_queue.0.push(DbCommand::ExecuteClosure(cmd));
        log::trace!("Queued STDB position update command for entity {}", entity_id_to_update);
    }
}

