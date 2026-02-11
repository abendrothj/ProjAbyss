# Graphics & Island Generation – Deep Dive

> Options for improving visuals and procedural island/world generation in ProjAbyss.

---

## 1. Graphics Libraries

### Built-in (Bevy 0.17)

- **PBR** – StandardMaterial, metallic/roughness, normal maps
- **Bloom** – bevy_post_process
- **Fog** – DistanceFog (exponential)
- **HDR + Tonemapping** – AgX
- **Color grading** – per-camera

### Custom Shaders

| Approach | Pros | Cons |
|----------|------|------|
| **Material trait** | Full control, no extra deps | More code |
| **Custom WGSL** | Ocean foam, SSS, depth fade | Requires shader knowledge |
| **bevy_water** | Ready-made ocean | Check Bevy version |

**Recommendation:** Keep current approach (vertex colors for foam, procedural normals). Add custom `Material` when you need GPU wave sampling or subsurface scattering.

### Third-Party Graphics

- **bevy_atmosphere** – volumetric sky, god rays
- **bevy_ssr** – screen-space reflections (expensive)
- **bevy_defer** – deferred rendering (more lights)

---

## 2. Island Generation – Current vs. Alternatives

### Current (Custom)

- **Organic blobs:** Sphere + FBM vertex displacement (3 octaves, hash-based noise)
- **Compound shapes:** Cones, cuboids, torus (atoll)
- **Scatter:** Rocks, seaweed, debris, buoys around island perimeters
- **No external deps** – deterministic, hash-based

### Option A: `noise` crate

```toml
noise = "0.8"
```

- Perlin, Simplex, FBM, Worley (cellular)
- Replace `smooth_noise` / `fbm` in islands.rs with `noise::Perlin`, `noise::Fbm`
- **Pros:** Well-tested, feature-rich
- **Cons:** Extra dependency

### Option B: `fastnoise2`

Already used in marine_snow. Could unify noise across the project.

- **Pros:** Fast, SIMD, same lib everywhere
- **Cons:** API differs from classic Perlin

### Option C: Polygonal islands (SoT-style)

From ISLANDS_UE_RESEARCH.md:

- Voronoi + Delaunay triangulation
- Lloyd relaxation for smoother cells
- Perlin/fbm for elevation

**Crates:** `voronoi`, `delaunator`, `spade`

```toml
delaunator = "0.4"
# or
spade = "2.0"  # Delaunay + Voronoi
```

Output: polygon mesh instead of deformed sphere. More work, more distinct look.

### Option D: Runtime PCG (like UE5)

- Grid-partitioned generation
- Spawn islands/caves as player approaches
- **Crates:** Custom; use `noise` or `fastnoise2` for sampling

---

## 3. Scatter & Detail

### Current

- Rocks (boulders + pebbles) around island perimeters
- Seaweed (capsules) near larger islands
- Debris (crates, barrels) on seafloor grid
- Buoys in open water

### Enhancements

| Feature | Approach |
|---------|----------|
| **Coral** | Add mesh type, place in shallow biomes |
| **Kelp forest** | Tall capsules, vertex animation (sway) |
| **LOD** | Distance-based mesh swap or culling |
| **Instancing** | Bevy auto-instances same mesh+material |

---

## 4. Biomes (Design Doc)

From proj.md:

- **Shallows (0–50m):** Bright, coral, sand
- **Kelp Forest (50–150m):** Murky green, kelp
- **Midnight Zone (150m+):** Dark, basalt, bioluminescence

**Implementation:** Sample depth (camera Y) or position; choose scatter density, material, fog. No new library needed—logic in scatter.rs or a new `biome.rs`.

---

## 5. Suggested Order

1. **Island gen:** Try `noise` crate for FBM—drop-in replacement, validate look.
2. **Polygonal islands:** Experiment with `spade` or `delaunator` for one “archipelago” island type.
3. **Graphics:** Custom water Material only when you need SSS or GPU waves.
4. **Biomes:** Add depth-based scatter/biome logic before adding new assets.
