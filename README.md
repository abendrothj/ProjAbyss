# ProjAbyss

Co-op extraction survival. Sail → Scan → Dive → Extract.

**Bevy 0.17 + Rapier + Rust.**

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
| **Global** | E – enter/exit vehicle, Escape – release cursor |

## Structure

| Module | What |
|--------|------|
| `ocean.rs` | Gerstner waves, water mesh, SEA_LEVEL |
| `ship.rs` | Dynamic ship, ExternalForce buoyancy, engine |
| `diving_bell.rs` | Submersible, oxygen drain, KinematicVelocityBased |
| `character.rs` | First-person, KinematicCharacterController, swim |
| `player.rs` | Mode switching, camera, depth color/fog |
| `islands.rs` | Organic blobs, compound shapes, FBM noise |
| `scatter.rs` | Rocks, seaweed, debris, buoys |
| `marine_snow.rs` | Underwater particles |

## Notes

- **Physics:** bevy_rapier3d for ship, sub, character, islands, seafloor
- **Water:** Surface at SEA_LEVEL (-2), no collider (player falls through)
- **Vision:** Sea of Thieves ocean + Subnautica depth (biomes, fauna)
