# ProjAbyss – Library Ecosystem Guide

How to integrate and use additional libraries with Bevy + Rapier for an ocean exploration game.

---

## Current Stack

| Layer | Library | Version | Purpose |
|-------|---------|---------|---------|
| Engine | Bevy | 0.17 | ECS, rendering, input |
| Physics | bevy_rapier3d | 0.32 | Collision, rigid bodies, joints |
| Noise | fastnoise2 | (marine_snow) | Marine snow randomness |
| Assets | (built-in) | – | GLTF, images |

---

## Graphics Libraries

| Library | Use Case | Bevy 0.17 |
|---------|----------|-----------|
| **bevy_pbr** | Built-in PBR | ✓ |
| **bevy_post_process** | Bloom, etc. | ✓ |
| **bevy_fog** | DistanceFog | ✓ (DistanceFog in bevy_pbr) |
| **bevy_water** | Ocean shader | Check compatibility |
| **bevy_atmosphere** | Sky/volumetric | Check compatibility |
| **naga** | Shader compilation | Bevy dependency |
| **wgpu** | GPU backend | Bevy dependency |

For custom water/ocean shaders, use Bevy's `Material` trait + WGSL. See GRAPHICS_UPGRADE_ROADMAP.md.

---

## Island & Procedural Generation

| Library | Use Case | Notes |
|---------|----------|-------|
| **fastnoise2** | FBM, Perlin, Simplex | Already used in marine_snow |
| **noise** | Perlin, Simplex, FBM, Worley | Stable, widely used |
| **simplex-noise** | Simplex only | Lightweight |
| **libnoise** | Classic libnoise port | More complex terrains |

**Current:** Custom hash-based smooth noise + FBM in `islands.rs` (no external deps). For Voronoi/Delaunay polygonal islands (SoT-style), consider `voronoi` or `delaunator` crates.

---

## When to Add New Libraries

### 1. **Audio** – `bevy_audio` or `bevy_kira`
- **Bevy 0.17** has `bevy_audio` with basic playback.
- **bevy_kira** – more control (spatial, ducking, reverb).
- **Use case:** ocean ambience, engine hum, depth alarm, shark roars.

### 2. **Networking** – `bevy_replicon` or `laminar`
- **bevy_replicon** – ECS replication over UDP.
- **Use case:** co-op ship sharing, sync diving bell, artifact pickup.
- **Note:** Rapier is deterministic; sync physics state, not raw inputs.

### 3. **UI** – Bevy UI or `bevy_egui`
- **Bevy UI** – built-in, good for HUD.
- **bevy_egui** – immediate-mode, useful for debug/inventory.
- **Use case:** oxygen meter, inventory, winch UI.

### 4. **Procedural Generation** – `fastnoise2`, `noise`, or custom
- **fastnoise2** – already in marine_snow.
- **noise** – Perlin, Simplex, FBM, Worley.
- **Custom** – islands.rs uses hash-based smooth noise + FBM (zero deps).
- **Use case:** island layout, rift caves, artifact spawn points. See GRAPHICS_ISLAND_GEN.md.

### 5. **Save / Load** – `ron` + `serde`
- **ron** – human-readable.
- **serde** – derive Serialize/Deserialize for components.
- **Use case:** save ship position, loot, progress.

### 6. **Paths / AI** – `bevy_path` or custom
- **NavMesh** – for shark / fish pathfinding.
- **Use case:** hostile shark, boid schooling.

---

## Adding a Library – Checklist

1. **Compatibility:** Check Bevy version (0.17 vs 0.18).
2. **Cargo.toml:** Add the crate with version.
3. **Features:** Disable defaults if not needed (e.g. `default-features = false`).
4. **Plugin:** Add `.add_plugins(SomePlugin::default())` in `main.rs`.
5. **Systems:** Add systems in the right schedule (Startup, Update, FixedUpdate).

---

## Example: Adding bevy_kira for Audio

```toml
# Cargo.toml
[dependencies]
bevy_kira_audio = { version = "0.18", default-features = false }
```

```rust
// main.rs
use bevy_kira_audio::AudioPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AudioPlugin)
        // ...
}
```

---

## Example: Adding bevy_egui for Debug UI

```toml
[dependencies]
bevy_egui = "0.38"
```

```rust
// main.rs
use bevy_egui::EguiPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_systems(Update, debug_ui)
        // ...
}
```

---

## Rapier + Other Libraries

- **Rapier + Networking:** Sync `Transform` and `Velocity`; avoid syncing internal Rapier handles.
- **Rapier + Save:** Serialize `Transform`, `Velocity`; restore by setting components.
- **Rapier + AI:** Use `RigidBodyPosition` or `Transform` for target positions; apply `ExternalForce` or `Velocity` for movement.

---

## Version Compatibility Matrix

| Bevy | bevy_rapier3d | bevy_kira_audio | bevy_egui |
|------|---------------|-----------------|-----------|
| 0.17 | 0.32 ✓ | 0.18 | 0.35 |
| 0.18 | (none yet) | 0.19 | 0.36 |

Always check each crate’s Cargo.toml for the exact Bevy version it depends on.
