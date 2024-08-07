
struct GridUniform {
    origin: vec4<f32>,
    resolution: u32,
    offset: f32,
    probe_radius: f32,
    // Add 4 bytes padding to avoid alignment issues.
    _padding: f32,
};

struct Atom {
    position: vec3<f32>,
    radius: f32,
    color: vec4<f32>,
};

struct GridCell {
    first_atom_index: u32,
    atoms_count: u32,
}

// Atoms Resource
@group(0) @binding(0) var<storage, read> atoms_sorted: array<Atom>;
@group(0) @binding(1) var<uniform> atoms_lookup_grid: GridUniform;
@group(0) @binding(2) var<storage, read> atoms_by_voxel: array<GridCell>;

// Distance Field Resource
@group(1) @binding(0) var<uniform> df_grid: GridUniform;

// Distance Field Grid Points Resource
@group(2) @binding(0) var<storage, read_write> df_grid_point_memory: array<u32>;
@group(2) @binding(1) var<uniform> df_grid_point_index_offset: u32;




@compute @workgroup_size(64)
fn main(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    var df_grid_point_memory_EXTERIOR: u32 = 0u;
    var df_grid_point_memory_INTERIOR: u32 = 1u;
    var df_grid_point_memory_BOUNDARY: u32 = 2u;

    var grid_point_index: u32 = global_invocation_id.x + df_grid_point_index_offset;
    var total: u32 = df_grid.resolution * df_grid.resolution * df_grid.resolution;
    if (grid_point_index >= total) {
        return;
    }

    df_grid_point_memory[grid_point_index] = df_grid_point_memory_EXTERIOR;

    // Compute the grid position
    var grid_point: vec3<f32> = df_grid.origin.xyz + vec3<f32>(
        f32(grid_point_index % df_grid.resolution),
        f32((grid_point_index / df_grid.resolution) % df_grid.resolution),
        f32(grid_point_index / (df_grid.resolution * df_grid.resolution))
    ) * df_grid.offset;

    var offset_grid_point: vec3<f32> = grid_point - atoms_lookup_grid.origin.xyz;

    var res: i32 = i32(atoms_lookup_grid.resolution);

    // Find index of the voxel containing the grid point
    var voxel_index: i32 = i32(offset_grid_point.x / atoms_lookup_grid.offset) +
        i32(offset_grid_point.y / atoms_lookup_grid.offset) * res +
        i32(offset_grid_point.z / atoms_lookup_grid.offset) * res * res;

    // Check all 27 neighboring voxels
    for (var x: i32 = -1; x <= 1; x = x + 1) {
        for (var y: i32 = -1; y <= 1; y = y + 1) {
            for (var z: i32 = -1; z <= 1; z = z + 1) {

                var neighbor_voxel_index: i32 = voxel_index + x + y * res + z * res * res;

                if (neighbor_voxel_index >= res * res * res || neighbor_voxel_index < 0) {
                    continue;
                }
                // TODO: Rename
                var grid_cell: GridCell = atoms_by_voxel[neighbor_voxel_index];

                // Classify the grid point based on the atoms in the neighboring voxels
                for (var i: i32 = 0; i < i32(grid_cell.atoms_count); i = i + 1) {
                    var atom_index: u32 = grid_cell.first_atom_index + u32(i);

                    var atom: Atom = atoms_sorted[atom_index];
                    var distance: f32 = length(grid_point - atom.position);

                    if (distance < atom.radius - df_grid.offset) {
                        // The grid point is inside an atom, no need to check the other atoms
                        df_grid_point_memory[grid_point_index] = df_grid_point_memory_INTERIOR;
                        return;
                    }
                    if (distance < atom.radius + df_grid.probe_radius) {
                        // The grid point is within the probe radius of an atom, it is a boundary point
                        df_grid_point_memory[grid_point_index] = df_grid_point_memory_BOUNDARY;
                    }
                }
            }
        }
    }
}