# ProjAbyss

Co-op extraction survival. Sail → Scan → Dive → Extract.

**Bevy + Rust. Pure code. All in.**

## Requirements

- Rust 1.75+
- cargo

## Run

```bash
cargo run
```

## Controls

- **WASD** – Ship throttle & steer (ship) / Move (character)
- **Space** – Jump (character)

## Structure

| Module | What |
|--------|------|
| `ocean.rs` | Gerstner wave solver for buoyancy |
| `ship.rs` | Ship with buoyancy, engine, pontoons |
| `diving_bell.rs` | Bell with oxygen drain underwater |
| `character.rs` | First-person marine character |

## Notes

- Manual physics (no Rapier) – ship/character use velocity components
- Wave height at Y=0, Bevy Y-up
- Same wave math as Unreal/Godot versions
- Winch + joints: add later with bevy_rapier3d or custom constraint
- Vision: Sea of Thieves ocean + Subnautica depth (underwater life, flora, biomes, fauna)
