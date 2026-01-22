//! Enemy spawning system.
//!
//! NOTE: SpawnZone-based spawning is deprecated. Use the monster grid in level files instead.
//! Enemies are now spawned directly by the world builder from level data.

// This module is kept for backwards compatibility but is no longer actively used.
// Monster spawning now happens in src/world/builder.rs via spawn_monsters_from_grid().
