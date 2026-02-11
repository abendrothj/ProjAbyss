# ProjAbyss Graphics Upgrade Roadmap

> Moving from the current primitive look to a Sea of Thieves / Subnautica–inspired visual style using Bevy's rendering stack.

## Current State

- **Ocean:** Subdivided plane, CPU vertex updates (Gerstner waves), basic `StandardMaterial` (solid color, alpha, reflectance)
- **Lighting:** DirectionalLight (25k illuminance, 4096 shadow map), PointLight fill, GlobalAmbientLight. HDR + Tonemapping::AgX
- **Materials:** Flat `srgb` colors, roughness/metallic on PBR. No textures or normal maps
- **Islands:** Procedural blobs + compound shapes (Cuboid, Cone, Sphere, Torus). No scatter or detail
- **Post-processing:** Tonemapping only. No Bloom, fog, or depth effects

---

## 1. Ocean Rendering (High Impact)

| Goal | Bevy Approach | Notes |
|------|---------------|-------|
| **PBR Water Shader** | Custom WGSL shader | Replace `StandardMaterial` with `Material` impl. |
| **Foam at wave crests** | Shader: sample wave height; add foam where `y + epsilon` crosses surface | Requires wave params in shader or uniform |
| **Subsurface scattering** | Fresnel + absorption in shader | Light passes through thin water; green tint at edges |
| **Screen-space reflections** | `bevy_ssr` or custom post-process | Expensive; optional for later |
| **Depth fade / opacity** | Shader: depth buffer vs. water depth | “Thalassophobia” – hide seafloor as depth increases |
| **Normal mapping** | Scrolling normal map over Gerstner base | Ripples without high vertex count |

**Priority:** Start with depth fade + foam in a custom water material. Then add normal mapping.

---

## 2. Atmospheric and Depth Effects

| Goal | Bevy Approach | Notes |
|------|---------------|-------|
| **Volumetric fog** | `bevy_fog` or custom post-process | God rays near surface; murky at depth |
| **Depth-based color grading** | Post-process: shift color by player Y | Red absorbed first; blue/green at depth (design bible) |
| **Underwater “snow”** | Particle system (GPU or CPU) | Marine snow particles for scale and motion |

**Priority:** Depth-based color grading ties into the Midnight Zone narrative. Fog can follow.

---

## 3. Lighting Upgrades

| Goal | Bevy Approach | Notes |
|------|---------------|-------|
| **Bloom** | `BloomSettings` on camera | Makes bioluminescence and diving bell lights pop |
| **Shadow casting** | `PointLight { shadows_enabled: true }` | Limit range for performance; use for bioluminescent props |
| **Shadow resolution** | Already 4096 on DirectionalLight | Consider per-light bias for quality |

**Priority:** Bloom is straightforward. Shadows on point lights when bioluminescent content exists.

---

## 4. Asset Fidelity

| Goal | Bevy Approach | Notes |
|------|---------------|-------|
| **GLTF models** | `bevy_gltf` / `AssetLoader` | Replace primitives for ship, diving bell, coral, rocks |
| **PBR textures** | `StandardMaterial` with `Color`, `NormalMap`, `OcclusionMap` | Metallic, roughness, AO from textures |
| **Weathered look** | Texture maps from Megascans or AI tools | Ship, bell, metal fixtures |

**Priority:** After gameplay is locked, swap ship and bell to GLTF. Add texture maps as assets are ready.

---

## 5. Procedural Detail

| Goal | Bevy Approach | Notes |
|------|---------------|-------|
| **Scatter system** | System: spawn rocks/seaweed/debris at island + seafloor | Use noise or grid for placement |
| **Vertex displacement** | Noise (Perlin/Simplex) in mesh generation | Seafloor and islands (already done for islands) |
| **Seafloor detail** | Subdivide plane + displace vertices | Similar to ocean; optional normal map |

**Priority:** Scatter around island bases first. Seafloor displacement can follow.

---

## Suggested Implementation Order

1. ~~**Bloom**~~ – DONE
2. ~~**Depth fade**~~ – Approximated via thickness + color grading
3. ~~**Foam at wave crests**~~ – DONE (vertex colors from wave gradient)
4. ~~**Depth-based color grading**~~ – DONE (camera Y drives blue tint)
5. ~~**Volumetric fog**~~ – DONE (DistanceFog on camera)
6. ~~**Scatter system**~~ – DONE (rocks, seaweed, debris)
7. ~~**Normal mapping on water**~~ – DONE (procedural 128×128 ripple normals, 24× tiling)
8. ~~**Marine snow**~~ – DONE (400 particles, underwater only, drift + recycle)
9. ~~**GLTF + textures**~~ – Sub colormap applied; island/rock procedural textures
10. ~~**Depth-based seafloor fade**~~ – Denser fog when underwater

---

## References

- [Bevy 0.18 Material](https://docs.rs/bevy/0.18.0/bevy/render/mesh/struct.Mesh.html)
- [Bevy Bloom](https://docs.rs/bevy/latest/bevy/core_pipeline/bloom/struct.BloomSettings.html)
- [Bevy Custom Shaders](https://bevyengine.org/learn/book/assets/shaders/)
- [proj.md](../proj.md) – Design bible (depth absorption, MegaLights → Bevy equivalents)
