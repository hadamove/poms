
struct GridUniform {
    origin: vec4<f32>,
    resolution: u32,
    offset: f32,
    size: f32,
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

@group(0) @binding(0) var<uniform> df_grid: GridUniform;
@group(0) @binding(1) var<uniform> probe_radius: f32;
@group(0) @binding(2) var<uniform> grid_point_index_offset: u32;

// Input buffers
@group(1) @binding(0) var<storage, read> atoms_sorted: array<Atom>; // 1. resource group
@group(1) @binding(1) var<uniform> neighbor_grid: GridUniform; // 2. resource group
@group(1) @binding(2) var<storage, read> grid_cells: array<GridCell>; // 2. resource group

// Output buffer
@group(1) @binding(3) var<storage, read_write> grid_point_class: array<u32>; // 3. resource group



@compute @workgroup_size(64)
fn main(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    var GRID_POINT_CLASS_EXTERIOR: u32 = 0u;
    var GRID_POINT_CLASS_INTERIOR: u32 = 1u;
    var GRID_POINT_CLASS_BOUNDARY: u32 = 2u;

    var grid_point_index: u32 = global_invocation_id.x + grid_point_index_offset;
    var total: u32 = df_grid.resolution * df_grid.resolution * df_grid.resolution;
    if (grid_point_index >= total) {
        return;
    }

    grid_point_class[grid_point_index] = GRID_POINT_CLASS_EXTERIOR;

    // Compute the grid position
    var grid_point: vec3<f32> = df_grid.origin.xyz + vec3<f32>(
        f32(grid_point_index % df_grid.resolution),
        f32((grid_point_index / df_grid.resolution) % df_grid.resolution),
        f32(grid_point_index / (df_grid.resolution * df_grid.resolution))
    ) * df_grid.offset;

    var offset_grid_point: vec3<f32> = grid_point - neighbor_grid.origin.xyz;

    var res: i32 = i32(neighbor_grid.resolution);
    var grid_cell_index: i32 = i32(offset_grid_point.x / neighbor_grid.offset) +
        i32(offset_grid_point.y / neighbor_grid.offset) * res +
        i32(offset_grid_point.z / neighbor_grid.offset) * res * res;

    // Check all 27 cells with neighboring atoms
    for (var x: i32 = -1; x <= 1; x = x + 1) {
        for (var y: i32 = -1; y <= 1; y = y + 1) {
            for (var z: i32 = -1; z <= 1; z = z + 1) {

                var neighbor_cell_index: i32 = grid_cell_index + x + y * res + z * res * res;
                
                if (neighbor_cell_index >= res * res * res || neighbor_cell_index < 0) {
                    continue;
                }
                var grid_cell: GridCell = grid_cells[neighbor_cell_index];

                for (var i: i32 = 0; i < i32(grid_cell.atoms_count); i = i + 1) {
                    var atom_index: u32 = grid_cell.first_atom_index + u32(i);

                    var atom: Atom = atoms_sorted[atom_index];
                    var distance: f32 = length(grid_point - atom.position);

                    if (distance < atom.radius - df_grid.offset) {
                        grid_point_class[grid_point_index] = GRID_POINT_CLASS_INTERIOR;
                        return;
                    }
                    if (distance < atom.radius + probe_radius) {
                        grid_point_class[grid_point_index] = GRID_POINT_CLASS_BOUNDARY;
                    }
                }
            }
        }
    }
}