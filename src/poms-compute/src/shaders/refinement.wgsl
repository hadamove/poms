struct GridUniform {
    origin: vec4<f32>,
    resolution: u32,
    offset: f32,
    probe_radius: f32,
    // Add 4 bytes padding to avoid alignment issues.
    _padding: f32,
};


// Distance Field Grid Points Resource
@group(0) @binding(0) var<storage, read_write> df_grid_point_memory: array<u32>;
@group(0) @binding(1) var<uniform> df_grid_point_index_offset: u32;

// Distance Field Resource
@group(1) @binding(0) var<uniform> df_grid: GridUniform;
@group(1) @binding(1) var df_texture: texture_storage_3d<rgba16float, write>;


const EXTERIOR_GRID_POINT: u32 = 0u;
const INTERIOR_GRID_POINT: u32 = 1u;
const BOUNDARY_GRID_POINT: u32 = 2u;

const HUGE_DISTANCE: f32 = 100000.0;
const PREDECESSOR_OFFSET: u32 = 3u;

fn grid_point_index_to_position(grid_point_index: u32) -> vec3<f32> {
    return df_grid.origin.xyz + vec3<f32>(
        f32(grid_point_index % df_grid.resolution),
        f32((grid_point_index / df_grid.resolution) % df_grid.resolution),
        f32(grid_point_index / (df_grid.resolution * df_grid.resolution))
    ) * df_grid.offset;
}

fn compute_distance(grid_point_index: u32) -> f32 {
    let grid_point_pos: vec3<f32> = grid_point_index_to_position(grid_point_index);
    let res: i32 = i32(df_grid.resolution);

    var min_distance: f32 = HUGE_DISTANCE;

    // Loop over all 3x3x3 neighboring points, find the closest exterior point.
    for (var x: i32 = -1; x <= 1; x = x + 1) {
        for (var y: i32 = -1; y <= 1; y = y + 1) {
            for (var z: i32 = -1; z <= 1; z = z + 1) {

                let neighbor_index: u32 = u32(i32(grid_point_index) + x + y * res + z * res * res);
                if neighbor_index >= arrayLength(&df_grid_point_memory) {
                    continue;
                }

                let neighbor_point_pos: vec3<f32> = grid_point_index_to_position(neighbor_index);
                var distance: f32 = HUGE_DISTANCE;

                switch df_grid_point_memory[neighbor_index] {
                    case EXTERIOR_GRID_POINT: {
                        // If the neighbor is an exterior point, we can directly compute the distance.
                        distance = length(neighbor_point_pos - grid_point_pos);
                    }
                    default: {
                        // Compare the distance to the predecessor of the neighbor.
                        // This represents the distance to the closest exterior point of the neighbor.
                        let predecessor_index: u32 = df_grid_point_memory[neighbor_index] - PREDECESSOR_OFFSET;
                        let predecessor_point_pos: vec3<f32> = grid_point_index_to_position(predecessor_index);
                        distance = length(predecessor_point_pos - grid_point_pos);
                    }
                }

                if distance < min_distance {
                    min_distance = distance;

                    switch df_grid_point_memory[neighbor_index] {
                        case EXTERIOR_GRID_POINT: {
                            // The neighbor is an exterior point.
                            // Store the neighbor as the predecessor of the grid point.
                            df_grid_point_memory[grid_point_index] = neighbor_index + PREDECESSOR_OFFSET;
                        }
                        default: {
                            // The predecessor of the grid point is the predecessor of the neighbor.
                            df_grid_point_memory[grid_point_index] = df_grid_point_memory[neighbor_index];
                        }
                    }
                }
            }
        }
    }
    if min_distance < HUGE_DISTANCE + 1.0 {
        return df_grid.probe_radius - min_distance;
    }
    // No exterior point found.
    return -df_grid.offset;
}


@compute @workgroup_size(64)
fn main(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let grid_point_index: u32 = global_invocation_id.x + df_grid_point_index_offset;
    let total_grid_points_count: u32 = df_grid.resolution * df_grid.resolution * df_grid.resolution;
    if (grid_point_index >= total_grid_points_count) {
        return;
    }

    let texture_index = vec3<i32>(
        i32(grid_point_index % df_grid.resolution),
        i32((grid_point_index / df_grid.resolution) % df_grid.resolution),
        i32(grid_point_index / (df_grid.resolution * df_grid.resolution))
    );

    switch (df_grid_point_memory[grid_point_index]) {
        case EXTERIOR_GRID_POINT: {
            textureStore(df_texture, texture_index, vec4<f32>(df_grid.probe_radius));
        }
        case INTERIOR_GRID_POINT: {
            textureStore(df_texture, texture_index, vec4<f32>(-df_grid.offset));
        }
        default: {
            let distance = compute_distance(grid_point_index);
            textureStore(df_texture, texture_index, vec4<f32>(distance));
        }
    }
}
