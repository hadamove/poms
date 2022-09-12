struct GridPoint {
    position: vec3<f32>,
    radius: f32,
    color: vec4<f32>,
};

struct GridPointsBuffer {
    data: array<GridPoint>,
};

struct GridUniform {
    origin: vec4<f32>,
    resolution: u32,
    offset: f32,
    // Add 8 bytes padding to avoid alignment issues.
    _padding: vec2<f32>,
};

struct Atom {
    position: vec3<f32>,
    radius: f32,
    color: vec4<f32>,
};

struct SortedAtomBuffer {
    atoms: array<Atom>,
};

struct GridCellStartIndicesBuffer {
    indices: array<u32>,
};;

@group(0) @binding(0) var<uniform> ses_grid: GridUniform;
@group(0) @binding(1) var<uniform> neighbor_grid: GridUniform;
@group(0) @binding(2) var<storage, read_write> grid_points: GridPointsBuffer;

@group(0) @binding(3) var<storage, read> sorted_atoms: SortedAtomBuffer;
@group(0) @binding(4) var<storage, read> grid_cell_start: GridCellStartIndicesBuffer;
@group(0) @binding(5) var<storage, read> grid_cell_size: GridCellStartIndicesBuffer;

@stage(compute)
@workgroup_size(64)
fn main(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    var grid_point_index: u32 = global_invocation_id.x;
    var total = arrayLength(&grid_points.data);
    if (grid_point_index >= total) {
        return;
    }

    // Compute the grid position
    var grid_point: vec3<f32> = ses_grid.origin.xyz + vec3<f32>(
        f32(grid_point_index % ses_grid.resolution),
        f32((grid_point_index / ses_grid.resolution) % ses_grid.resolution),
        f32(grid_point_index / (ses_grid.resolution * ses_grid.resolution))
    ) * ses_grid.offset;

    // Compute grid point's position, save it to a buffer
    grid_points.data[grid_point_index].position = grid_point;
    grid_points.data[grid_point_index].radius = 0.3;

    var offset_grid_point = grid_point - neighbor_grid.origin.xyz;

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
                var start_index: u32 = grid_cell_start.indices[u32(neighbor_cell_index)];
                var cell_size: u32 = grid_cell_size.indices[u32(neighbor_cell_index)];

                for (var i: i32 = 0; i < i32(cell_size); i = i + 1) {
                    var atom_index: u32 = start_index + u32(i);

                    var atom: Atom = sorted_atoms.atoms[atom_index];
                    var distance: f32 = length(grid_point - atom.position);
                    if (distance < atom.radius + 1.2) {
                        grid_points.data[grid_point_index].color = grid_points.data[grid_point_index].color / 2.0 + atom.color / 2.0;
                    }
                }
            }
        }
    }
}