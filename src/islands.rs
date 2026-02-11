//! Islands with varied shapes and collision.
//!
//! Refined approach: multi-octave noise for natural terrain, data-driven island
//! definitions, Safe Island marker for spawn/safe zone (proj design doc).

use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;

/// Collision shape for islands. Used for ship and character collision.
#[derive(Component)]
pub struct IslandCollider {
    /// Radius for scatter placement and broad-phase (kept for compatibility).
    pub radius: f32,
}

/// Per-part collision shape matching the visible mesh. Used for accurate collision.
#[derive(Component, Clone)]
pub enum ColliderShape {
    /// Flat disc (cylinder) – for organic blobs, sand rings, atoll.
    Disc { radius: f32, half_height: f32 },
    /// Oriented box – for compound cuboid parts.
    Box { half_extents: Vec3 },
    /// Cylinder – for cone parts (approximated).
    Cylinder { radius: f32, half_height: f32 },
}

impl ColliderShape {
    /// If `point` is inside this shape (with `margin`), returns the push vector to escape.
    pub fn penetration(&self, center: Vec3, rotation: Quat, point: Vec3, margin: f32) -> Option<Vec3> {
        match self {
            ColliderShape::Disc { radius, half_height } => {
                let delta_xz = point.xz() - center.xz();
                let dist_xz = delta_xz.length();
                let in_radius = dist_xz < radius + margin;
                let in_height = point.y >= center.y - half_height - margin
                    && point.y <= center.y + half_height + margin;
                if in_radius && in_height {
                    let push_xz = if dist_xz > 0.001 {
                        (radius + margin - dist_xz) * (delta_xz / dist_xz)
                    } else {
                        Vec2::new(radius + margin, 0.0)
                    };
                    Some(Vec3::new(push_xz.x, 0.0, push_xz.y))
                } else {
                    None
                }
            }
            ColliderShape::Box { half_extents } => {
                let rel = rotation.inverse() * (point - center);
                let px = (half_extents.x + margin) - rel.x.abs();
                let py = (half_extents.y + margin) - rel.y.abs();
                let pz = (half_extents.z + margin) - rel.z.abs();
                if px > 0.0 && py > 0.0 && pz > 0.0 {
                    let (min_p, axis) = if px <= py && px <= pz {
                        (px, Vec3::new(rel.x.signum(), 0.0, 0.0))
                    } else if py <= px && py <= pz {
                        (py, Vec3::new(0.0, rel.y.signum(), 0.0))
                    } else {
                        (pz, Vec3::new(0.0, 0.0, rel.z.signum()))
                    };
                    Some(rotation * (axis * min_p))
                } else {
                    None
                }
            }
            ColliderShape::Cylinder { radius, half_height } => {
                let delta_xz = point.xz() - center.xz();
                let dist_xz = delta_xz.length();
                let in_radius = dist_xz < radius + margin;
                let in_height = point.y >= center.y - half_height - margin
                    && point.y <= center.y + half_height + margin;
                if in_radius && in_height {
                    let push_xz = if dist_xz > 0.001 {
                        (radius + margin - dist_xz) * (delta_xz / dist_xz)
                    } else {
                        Vec2::new(radius + margin, 0.0)
                    };
                    Some(Vec3::new(push_xz.x, 0.0, push_xz.y))
                } else {
                    None
                }
            }
        }
    }
}

/// Marks the spawn/safe island where players stock the ship (proj design doc).
#[derive(Component)]
pub struct SafeIsland;

/// Simple hash for deterministic pseudo-noise (no external deps).
fn hash21(p: Vec2) -> f32 {
    let p = p.to_array();
    let mut h = p[0].to_bits().wrapping_add(p[1].to_bits().wrapping_mul(374761393));
    h = (h ^ (h >> 13)).wrapping_mul(668265263);
    (h & 0xFFFF) as f32 / 65535.0
}

/// Smooth value noise: bilinear interpolation between hash samples.
fn smooth_noise(p: Vec2) -> f32 {
    let cell = Vec2::new(p.x.floor(), p.y.floor());
    let frac = Vec2::new(p.x.fract(), p.y.fract());
    let fade = |t: f32| t * t * (3.0 - 2.0 * t);
    let fx = fade(frac.x);
    let fy = fade(frac.y);
    let h00 = hash21(cell);
    let h10 = hash21(cell + Vec2::new(1.0, 0.0));
    let h01 = hash21(cell + Vec2::new(0.0, 1.0));
    let h11 = hash21(cell + Vec2::new(1.0, 1.0));
    let h0 = h00 * (1.0 - fx) + h10 * fx;
    let h1 = h01 * (1.0 - fx) + h11 * fx;
    h0 * (1.0 - fy) + h1 * fy
}

/// Fractal Brownian Motion: multi-octave noise for natural terrain variation.
fn fbm(p: Vec2, octaves: u32, seed: f32) -> f32 {
    let mut v = 0.0;
    let mut a = 0.5;
    let mut f = 1.0;
    let mut sum_a = 0.0;
    for i in 0..octaves {
        let off = seed * (i as f32 + 1.0) * 0.17;
        v += a * smooth_noise(Vec2::new(p.x * f + off, p.y * f + off * 0.7));
        sum_a += a;
        a *= 0.5;
        f *= 2.0;
    }
    v / sum_a
}

/// Height ratio for Caribbean-style flat islands (fraction of radius).
/// 0.05 = barely out of water, ~2–3m elevation.
const ISLAND_HEIGHT_RATIO: f32 = 0.05;

/// Creates an organic, flat island mesh (Caribbean style): wide disc barely above water.
/// Squashes a sphere to a pancake, then displaces with FBM for irregular coastline.
pub fn create_organic_blob_mesh(
    meshes: &mut Assets<Mesh>,
    radius: f32,
    subdivisions: u32,
    seed: f32,
    roughness: f32,
) -> Handle<Mesh> {
    let mut mesh = Sphere::new(radius).mesh().uv(subdivisions.max(8) * 2, subdivisions.max(8));
    let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
    else {
        return meshes.add(mesh);
    };
    let freq = 1.8;
    let y_scale = ISLAND_HEIGHT_RATIO;
    let y_offset = radius * y_scale; // so bottom sits at y=0

    for pos in positions.iter_mut() {
        let p = Vec3::from_array(*pos);
        let lat = (p.y / radius).clamp(-1.0, 1.0).acos();
        let lon = p.z.atan2(p.x);
        let u = Vec2::new(lat * freq + seed, lon * freq + seed * 0.7);
        let noise = fbm(u, 3, seed);
        let disp = (noise - 0.5) * 2.0 * radius * roughness;
        let scale = 1.0 + disp / radius;
        pos[0] *= scale;
        pos[1] *= scale * y_scale;
        pos[2] *= scale;
        pos[1] += y_offset;
    }
    mesh.compute_normals();
    meshes.add(mesh)
}

// --- Island definitions & spawning ---

#[derive(Clone, Copy)]
enum PartKind {
    Cuboid(Vec3),
    Cone { radius: f32, height: f32 },
}

fn spawn_compound(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    center: Vec3,
    coll_radius: f32,
    parts: &[(PartKind, Vec3, Vec3, bool)],
    island_mat: &Handle<StandardMaterial>,
    rock_mat: &Handle<StandardMaterial>,
) {
    let parent = commands
        .spawn((
            Transform::from_translation(center),
            IslandCollider { radius: coll_radius },
        ))
        .id();
    for (kind, local_pos, local_scale, is_rock) in parts.iter() {
        let (mesh, collider) = match kind {
            PartKind::Cuboid(s) => (
                Cuboid::new(s.x, s.y, s.z).mesh().build(),
                ColliderShape::Box {
                    half_extents: *local_scale * 0.5,
                },
            ),
            PartKind::Cone { radius, height } => (
                Cone::new(*radius, *height).mesh().resolution(24).build(),
                ColliderShape::Cylinder {
                    radius: radius * local_scale.x,
                    half_height: height * local_scale.y * 0.5,
                },
            ),
        };
        let child = commands
            .spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(if *is_rock {
                    rock_mat.clone()
                } else {
                    island_mat.clone()
                }),
                Transform::from_translation(*local_pos).with_scale(*local_scale),
                collider,
            ))
            .id();
        commands.entity(parent).add_child(child);
    }
}

/// Spawns all islands. Call from setup_scene after materials are created.
pub fn spawn_all_islands(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    island_mat: &Handle<StandardMaterial>,
    rock_mat: &Handle<StandardMaterial>,
    sand_mat: &Handle<StandardMaterial>,
) {
    // Procedural organic blob island
    let organic_mesh = create_organic_blob_mesh(meshes, 35.0, 24, 1.23, 0.22);
    commands.spawn((
        Mesh3d(organic_mesh),
        MeshMaterial3d(island_mat.clone()),
        Transform::from_xyz(-120.0, 0.0, -200.0),
        IslandCollider { radius: 45.0 },
        ColliderShape::Disc { radius: 45.0, half_height: 2.0 },
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(50.0, 0.6))),
        MeshMaterial3d(sand_mat.clone()),
        Transform::from_xyz(-120.0, 0.0, -200.0),
        IslandCollider { radius: 50.0 },
        ColliderShape::Disc { radius: 50.0, half_height: 0.35 },
    ));

    // Volcanic cay: flat cone + low boulders (Caribbean style)
    spawn_compound(
        commands,
        meshes,
        Vec3::new(180.0, 0.0, 150.0),
        42.0,
        &[
            (PartKind::Cone { radius: 1.0, height: 1.0 }, Vec3::ZERO, Vec3::new(25.0, 3.5, 25.0), false),
            (PartKind::Cuboid(Vec3::ONE), Vec3::new(8.0, 2.5, -5.0), Vec3::new(8.0, 2.0, 8.0), false),
            (PartKind::Cuboid(Vec3::ONE), Vec3::new(-12.0, 1.0, 10.0), Vec3::new(5.0, 1.2, 5.0), true),
            (PartKind::Cuboid(Vec3::ONE), Vec3::new(15.0, 1.2, 8.0), Vec3::new(4.0, 1.0, 4.0), true),
        ],
        island_mat,
        rock_mat,
    );

    // Rocky archipelago: low spires + flat base
    spawn_compound(
        commands,
        meshes,
        Vec3::new(80.0, 0.0, -300.0),
        38.0,
        &[
            (PartKind::Cuboid(Vec3::ONE), Vec3::new(0.0, 2.0, 0.0), Vec3::new(14.0, 3.0, 10.0), true),
            (PartKind::Cuboid(Vec3::ONE), Vec3::new(-18.0, 1.2, -8.0), Vec3::new(8.0, 2.0, 6.0), true),
            (PartKind::Cuboid(Vec3::ONE), Vec3::new(12.0, 1.0, 12.0), Vec3::new(6.0, 2.0, 5.0), true),
            (PartKind::Cuboid(Vec3::ONE), Vec3::new(0.0, 0.25, 0.0), Vec3::new(40.0, 0.5, 35.0), false),
        ],
        island_mat,
        rock_mat,
    );

    // Atoll (flat torus ring) + central islet
    let parent = commands
        .spawn((
            Transform::from_translation(Vec3::new(-250.0, 0.0, 100.0)),
            IslandCollider { radius: 38.0 },
        ))
        .id();
    let torus_id = commands
        .spawn((
            Mesh3d(meshes.add(
                Torus::new(20.0, 6.0)
                    .mesh()
                    .minor_resolution(16)
                    .major_resolution(32),
            )),
            MeshMaterial3d(sand_mat.clone()),
            Transform::default().with_scale(Vec3::new(1.0, 0.12, 1.0)),
            ColliderShape::Disc { radius: 26.0, half_height: 0.8 },
        ))
        .id();
    let islet_id = commands
        .spawn((
            Mesh3d(meshes.add(Sphere::new(6.0).mesh().uv(16, 12))),
            MeshMaterial3d(island_mat.clone()),
            Transform::from_xyz(0.0, 0.4, 0.0).with_scale(Vec3::new(1.0, 0.08, 1.0)),
            ColliderShape::Disc { radius: 5.0, half_height: 0.5 },
        ))
        .id();
    commands.entity(parent).add_child(torus_id);
    commands.entity(parent).add_child(islet_id);

    // Organic coastal island
    let blob2 = create_organic_blob_mesh(meshes, 28.0, 20, 4.56, 0.25);
    commands.spawn((
        Mesh3d(blob2),
        MeshMaterial3d(island_mat.clone()),
        Transform::from_xyz(300.0, 0.0, -80.0),
        IslandCollider { radius: 35.0 },
        ColliderShape::Disc { radius: 35.0, half_height: 1.8 },
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(32.0, 0.5))),
        MeshMaterial3d(sand_mat.clone()),
        Transform::from_xyz(300.0, 0.0, -80.0),
        IslandCollider { radius: 32.0 },
        ColliderShape::Disc { radius: 32.0, half_height: 0.3 },
    ));

    // Flat cay: low plateau + rocks
    spawn_compound(
        commands,
        meshes,
        Vec3::new(-80.0, 0.0, 220.0),
        28.0,
        &[
            (PartKind::Cuboid(Vec3::ONE), Vec3::new(0.0, 1.2, 0.0), Vec3::new(25.0, 2.0, 20.0), true),
            (PartKind::Cuboid(Vec3::ONE), Vec3::new(-8.0, 0.6, -5.0), Vec3::new(12.0, 1.2, 10.0), true),
            (PartKind::Cuboid(Vec3::ONE), Vec3::new(10.0, 0.4, 8.0), Vec3::new(8.0, 1.0, 6.0), true),
        ],
        island_mat,
        rock_mat,
    );

    // Forested organic cay
    let blob3 = create_organic_blob_mesh(meshes, 22.0, 22, 7.89, 0.2);
    commands.spawn((
        Mesh3d(blob3),
        MeshMaterial3d(island_mat.clone()),
        Transform::from_xyz(50.0, 0.0, 280.0),
        IslandCollider { radius: 28.0 },
        ColliderShape::Disc { radius: 28.0, half_height: 1.4 },
    ));

    // Long reef: multiple segments
    spawn_compound(
        commands,
        meshes,
        Vec3::new(-350.0, 0.0, -150.0),
        48.0,
        &[
            (PartKind::Cuboid(Vec3::ONE), Vec3::new(-25.0, 0.0, 0.0), Vec3::new(25.0, 1.5, 6.0), true),
            (PartKind::Cuboid(Vec3::ONE), Vec3::new(0.0, 0.0, 0.0), Vec3::new(28.0, 1.2, 5.0), true),
            (PartKind::Cuboid(Vec3::ONE), Vec3::new(25.0, 0.0, 0.0), Vec3::new(22.0, 1.0, 5.0), true),
        ],
        island_mat,
        rock_mat,
    );

    // Rocky cluster: low outcrops
    spawn_compound(
        commands,
        meshes,
        Vec3::new(220.0, 0.0, 250.0),
        22.0,
        &[
            (PartKind::Cuboid(Vec3::ONE), Vec3::new(0.0, 1.2, 0.0), Vec3::new(12.0, 2.0, 10.0), true),
            (PartKind::Cuboid(Vec3::ONE), Vec3::new(-8.0, 0.8, 5.0), Vec3::new(6.0, 1.5, 5.0), true),
            (PartKind::Cuboid(Vec3::ONE), Vec3::new(6.0, 0.5, -6.0), Vec3::new(5.0, 1.2, 4.0), true),
        ],
        island_mat,
        rock_mat,
    );

    // Safe Island – spawn point, players stock ship here (proj design doc)
    let blob_spawn = create_organic_blob_mesh(meshes, 10.0, 16, 11.1, 0.28);
    commands.spawn((
        Mesh3d(blob_spawn),
        MeshMaterial3d(island_mat.clone()),
        Transform::from_xyz(-15.0, 0.0, 10.0),
        IslandCollider { radius: 14.0 },
        ColliderShape::Disc { radius: 14.0, half_height: 0.8 },
        SafeIsland,
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(14.0, 0.4))),
        MeshMaterial3d(sand_mat.clone()),
        Transform::from_xyz(-15.0, 0.0, 10.0),
        IslandCollider { radius: 14.0 },
        ColliderShape::Disc { radius: 14.0, half_height: 0.2 },
        SafeIsland,
    ));
}
