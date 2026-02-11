# ProjAbyss

Co-op underwater exploration. Sail → Swim → Dive → Extract.

**Bevy 0.17 + Rapier + Rust.**

Underwater exploration first: swim freely in shallows and mid-depth; sub only for excessively deep parts. See [docs/DESIGN.md](docs/DESIGN.md).

## Requirements

- Rust 1.75+
- cargo

## Run

```bash
cargo run
```

## Controls

| Mode | Action |
|------|--------|
| **Character** | WASD move, Space jump, Mouse look |
| **Ship** | WASD throttle/steer, Mouse look, E enter/exit |
| **Submersible** | WASD drive, Space/Shift ascend/descend, Mouse look, E enter/exit |
| **Boat (winch)** | R reel in, T reel out |
| **Global** | E – enter/exit vehicle, Escape – pause/resume, Enter/E – start from menu, F5 – save, F9 – load |

## Structure

| Module | What |
|--------|------|
| `game_state.rs` | Menu, pause, GameState (Menu/Playing/Paused) |
| `interaction.rs` | Interactable, InteractKind (EnterShip, EnterSub, Pickup) |
| `world.rs` | MAP_SIZE (5km), MAP_FLOOR_Y, spawn position |
| `ocean.rs` | Gerstner waves, water mesh, SEA_LEVEL |
| `ship.rs` | Dynamic ship, ExternalForce buoyancy, engine |
| `diving_bell.rs` | Submersible, oxygen drain, KinematicVelocityBased |
| `winch.rs` | RopeJoint tether ship–sub, reel in/out |
| `character.rs` | First-person, swim, oxygen, pressure, respawn |
| `player.rs` | Mode switching, camera, prompts, depth color/fog |
| `save_load.rs` | F5 save, F9 load (save.ron) |
| `artifacts.rs` | Artifacts, inventory |
| `audio.rs` | Pickup sound (add assets/audio/pickup.ogg) |
| `islands.rs` | Organic blobs, compound shapes, FBM noise |
| `scatter.rs` | Rocks, seaweed, debris, buoys |
| `marine_snow.rs` | Underwater particles |

## Docs

- **docs/DESIGN.md** – Vision, depth zones, swim vs sub
- **docs/EXPLORATION.md** – Depth thresholds, implementation
- **docs/FOUNDATION.md** – All foundational systems (oxygen, save, interaction, etc.)
- **docs/GRAPHICS_UPGRADE_ROADMAP.md** – Visual direction

## Notes

- **Physics:** bevy_rapier3d for ship, sub, character, islands, seafloor, winch (RopeJoint)
- **Water:** Surface at SEA_LEVEL (-2), no collider (player falls through)
- **Vision:** Underwater exploration first; sub only for deep parts
- **Dependencies:** bevy 0.17, bevy_rapier3d 0.32, ron, serde
