# Islands in Unreal Engine – Research Summary

> Internet research on how islands are implemented in UE5 and related games (Sea of Thieves–style, ProjAbyss design). For Bevy parity/feature mapping.

---

## 1. Epic’s PCG Framework (UE5)

**Procedural Content Generation (PCG)** is UE5’s built‑in system for procedural content.

### Features
- **PCG Graph** – node-based graph for spawning meshes, foliage, decals, etc.
- **Scatter Graphs** – place rocks, coral, foliage by sampling surfaces (landscape, meshes).
- **Generation modes**
  - **Non-partitioned** – whole area at once
  - **Partitioned** – grid cells for large worlds
  - **Hierarchical** – coarse grid for big objects, fine grid for details
  - **Runtime** – generate during gameplay

### Typical use for islands
- **Landmass** – manual sculpt in Landscape Editor OR procedural via PCG
- **Scatter** – PCG graph to place rocks, coral, foliage on island/seafloor surfaces
- **Biomes** – PCG Biome Core plugin for attribute tables, feedback loops, recursive sub-graphs

---

## 2. Procedural Island Generation in UE5

### A. IslandGeneratorPlugin (Zoxemik)

- **Tech**: `UDynamicMesh` + Geometry Script
- **Actor**: `AIslandConstructor` extends `ADynamicMeshActor`
- **Flow**:
  1. Seed from `GameInstance` (deterministic)
  2. Spawn cones/boxes, merge with base plane
  3. `ApplyMeshSolidify` (voxelize)
  4. `ApplyIterativeSmoothingToMesh`
  5. `ApplyMeshPlaneCut` for shape
  6. `SetMeshUVsFromPlanarProjection` for materials
- **Parameters**: island count, size, height, tessellation, grid resolution
- **Integration**: Notifies `GameMode` when generation finishes (e.g. NavMesh)

### B. Sea of Thieves–style approaches

**Polygon graph (Voronoi + Delaunay):**
- Random points → Voronoi + Delaunay triangulation
- Lloyd relaxation for smoother cells
- Perlin noise + falloff for elevation
- Elevation + moisture for biomes
- Output: mesh and textures

**Hex grid + Perlin:**
- Hex grid + Perlin noise
- Steps: island shape → smoothing → elevation → biomes

**Houdini Engine:**
- HDA parameters for many island variants
- Matches SoT art style

---

## 3. Safe Island / Spawn Island (ProjAbyss design)

From `proj.md`:

- **Role**: Spawn point and safe zone; players stock ship (fuel, oxygen, repair wood).
- **UE plan**: “Landmass: Sculpt ‘Spawn Island’ (Safe Zone)” – manual sculpt in editor.
- **PCG**: “Scatter Graph to populate the ocean floor with rocks/coral” – ocean floor only, not islands.

---

## 4. UE5 → Bevy Mapping (ProjAbyss)

| UE5 concept | Bevy equivalent in ProjAbyss |
|-------------|------------------------------|
| Procedural island mesh | `create_organic_blob_mesh` (sphere + smooth noise) |
| Compound shapes (cones, boxes) | `spawn_compound` with `PartKind::Cone` / `Cuboid` |
| Island collision | `IslandCollider` (radius-based) |
| PCG scatter (rocks, coral) | `scatter.rs` (rocks, seaweed, debris) |
| Manual sculpt spawn island | Static placement in `setup_scene` |
| Geometry Script / UDynamicMesh | `Mesh`, `VertexAttributeValues`, procedural mesh changes |

---

## 5. Bevy Refinements (2026-02)

Applied after research:

- **Multi-octave noise (FBM)**: Organic blobs use 3-octave fractal Brownian motion instead of single-octave smooth noise. More natural coastline and terrain variation.
- **Data-driven layout**: `spawn_all_islands()` moved into `islands.rs`; `main.rs` only passes material handles. PartKind and spawn_compound are internal helpers.
- **SafeIsland component**: Spawn island at (-15, 0, 10) marked with `SafeIsland` for future gameplay (stocking ship, spawn point).
- **Rapier colliders**: All islands use `RigidBody::Fixed` + `Collider` (cylinder, cuboid); `IslandCollider` retained for scatter placement.
- **Plane-cut deferred**: Clipping sphere at Y=0 would require mesh surgery; left for later.

---

## 6. References

- [PCG Biome Core (UE 5.7)](https://dev.epicgames.com/documentation/en-us/unreal-engine/procedural-content-generation-pcg-biome-core-and-sample-plugins-in-unreal-engine)
- [IslandGeneratorPlugin](https://github.com/Zoxemik/IslandGeneratorPlugin) – Geometry Script island generation
- [Polygonal Island Generation (SoT-style)](https://github.com/temur-kh/polygonal-island-generation) – Voronoi + Delaunay
- [UE Procedurally Scatter Items (PCG)](https://dev.epicgames.com/community/learning/tutorials/7BMa/unreal-engine-procedurally-scatter-items-without-collision-using-pcg-in-ue-5-2)
- [Patel 2010 – Polygon Map Generation](http://www-cs-students.stanford.edu/~amitp/game-programming/polygon-map-generation/)
