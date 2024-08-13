
struct GridUniform {
    origin: vec4<f32>,
    resolution: u32,
    spacing: f32,
    probe_radius: f32,
    // Add 4 bytes padding to avoid alignment issues.
    _padding: f32,
};

struct Atom {
    position: vec3<f32>,
    radius: f32,
    color: vec4<f32>,
};

struct AtomSegment {
    first_atom_index: u32,
    atoms_count: u32,
}

// Distance Field Grid Points Resource
@group(0) @binding(0) var<storage, read_write> df_grid_point_memory: array<u32>;
@group(0) @binding(1) var<uniform> df_grid_point_index_offset: u32;

// Distance Field Resource
@group(1) @binding(0) var<uniform> df_grid: GridUniform;

// Atoms Resource
@group(2) @binding(0) var<storage, read> atoms_sorted: array<Atom>;
@group(2) @binding(1) var<uniform> atoms_lookup_grid: GridUniform;
@group(2) @binding(2) var<storage, read> atom_segment_by_voxel: array<AtomSegment>;


const EXTERIOR_GRID_POINT: u32 = 0u;
const INTERIOR_GRID_POINT: u32 = 1u;
const BOUNDARY_GRID_POINT: u32 = 2u;


@compute @workgroup_size(64)
fn main(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let grid_point_index: u32 = global_invocation_id.x + df_grid_point_index_offset;
    let total_grid_points_count: u32 = df_grid.resolution * df_grid.resolution * df_grid.resolution;
    if (grid_point_index >= total_grid_points_count) {
        return;
    }

    // Initialize the grid point memory to EXTERIOR
    df_grid_point_memory[grid_point_index] = EXTERIOR_GRID_POINT;

    // Compute the grid position
    let grid_point: vec3<f32> = df_grid.origin.xyz + vec3<f32>(
        f32(grid_point_index % df_grid.resolution),
        f32((grid_point_index / df_grid.resolution) % df_grid.resolution),
        f32(grid_point_index / (df_grid.resolution * df_grid.resolution))
    ) * df_grid.spacing;

    // Compute the offset of the grid point from the origin of the atoms lookup grid
    let offset_grid_point: vec3<f32> = grid_point - atoms_lookup_grid.origin.xyz;

    let res: i32 = i32(atoms_lookup_grid.resolution);

    // Find index of the voxel containing the grid point within the atoms lookup grid
    let voxel_index: i32 = i32(offset_grid_point.x / atoms_lookup_grid.spacing) +
        i32(offset_grid_point.y / atoms_lookup_grid.spacing) * res +
        i32(offset_grid_point.z / atoms_lookup_grid.spacing) * res * res;

    // Check all 27 neighboring voxels
    for (var x: i32 = -1; x <= 1; x += 1) {
        for (var y: i32 = -1; y <= 1; y += 1) {
            for (var z: i32 = -1; z <= 1; z += 1) {

                let neighbor_voxel_index: i32 = voxel_index + x + y * res + z * res * res;

                if (neighbor_voxel_index >= res * res * res || neighbor_voxel_index < 0) {
                    continue;
                }
                let atom_segment: AtomSegment = atom_segment_by_voxel[neighbor_voxel_index];

                // Classify the grid point based on the atoms in the neighboring voxels
                for (var i: u32 = 0; i < atom_segment.atoms_count; i += 1u) {

                    let atom: Atom = atoms_sorted[atom_segment.first_atom_index + i];
                    let distance: f32 = length(grid_point - atom.position);

                    if (distance < atom.radius - df_grid.spacing) {
                        // The grid point is inside an atom, no need to check the other atoms
                        df_grid_point_memory[grid_point_index] = INTERIOR_GRID_POINT;
                        return;
                    }
                    if (distance < atom.radius + df_grid.probe_radius) {
                        // The grid point is within the probe radius of an atom, it is a boundary point
                        df_grid_point_memory[grid_point_index] = BOUNDARY_GRID_POINT;
                    }
                }
            }
        }
    }
}