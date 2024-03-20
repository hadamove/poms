struct GridUniform {
    origin: vec4<f32>,
    resolution: u32,
    offset: f32,
    size: f32,
    // Add 4 bytes padding to avoid alignment issues.
    _padding: f32,
};


// Distance Field Resource
@group(0) @binding(0) var<uniform> probe_radius: f32;
@group(0) @binding(1) var<uniform> df_grid: GridUniform;
@group(0) @binding(2) var df_texture: texture_storage_3d<rgba16float, write>;

// Distance Field Grid Points Resource
@group(1) @binding(0) var<storage, read_write> df_grid_point_memory: array<u32>;
@group(1) @binding(1) var<uniform> df_grid_point_index_offset: u32;


fn grid_point_index_to_position(grid_point_index: u32) -> vec3<f32> {
    return df_grid.origin.xyz + vec3<f32>(
        f32(grid_point_index % df_grid.resolution),
        f32((grid_point_index / df_grid.resolution) % df_grid.resolution),
        f32(grid_point_index / (df_grid.resolution * df_grid.resolution))
    ) * df_grid.offset;
}


fn compute_distance(grid_point_index: u32) -> f32 {
    var HUGE_DISTANCE: f32 = 100000.0;

    var grid_point_pos: vec3<f32> = grid_point_index_to_position(grid_point_index);
    var res: i32 = i32(df_grid.resolution);

    var search_range: i32 = 1;
    var min_distance: f32 = HUGE_DISTANCE;

    // Loop over all 3x3x3 neighboring points, find the closest exterior point.
    for (var x: i32 = -1; x <= 1; x = x + 1) {
        for (var y: i32 = -1; y <= 1; y = y + 1) {
            for (var z: i32 = -search_range; z <= search_range; z = z + 1) {
                // TODO: Remove this type casting
                var neighbor_index: u32 = u32(i32(grid_point_index) + x + y * res + z * res * res);

                if neighbor_index >= arrayLength(&df_grid_point_memory) {
                    continue;
                }

                var neighbor_point_pos: vec3<f32> = grid_point_index_to_position(neighbor_index);
                var distance: f32 = HUGE_DISTANCE;

                if df_grid_point_memory[neighbor_index] == 0u {
                    distance = length(neighbor_point_pos - grid_point_pos);
                } else if df_grid_point_memory[neighbor_index] != 0u {
                    var predecessor_pos: vec3<f32> = grid_point_index_to_position(df_grid_point_memory[neighbor_index] - 3u);
                    distance = length(predecessor_pos - grid_point_pos);
                }

                if distance < min_distance {
                    min_distance = distance;

                    if df_grid_point_memory[neighbor_index] == 0u {
                        // TODO: Explain `+ 3u`, this whole thing
                        df_grid_point_memory[grid_point_index] = neighbor_index + 3u;
                    } else {
                        df_grid_point_memory[grid_point_index] = df_grid_point_memory[neighbor_index];
                    }
                }
            }
        }
    }
    if min_distance < HUGE_DISTANCE + 1.0 {
        return probe_radius - min_distance;
    }
    // No exterior point found.
    return -df_grid.offset;
}


@compute @workgroup_size(64)
fn main(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    var grid_point_index: u32 = global_invocation_id.x + df_grid_point_index_offset;
    var total: u32 = df_grid.resolution * df_grid.resolution * df_grid.resolution;
    if (grid_point_index >= total) {
        return;
    }

    var texture_index = vec3<i32>(
        i32(grid_point_index % df_grid.resolution),
        i32((grid_point_index / df_grid.resolution) % df_grid.resolution),
        i32(grid_point_index / (df_grid.resolution * df_grid.resolution))
    );

    // Switch cases with constants are not supported yet.
    switch (df_grid_point_memory[grid_point_index]) {
        case 0u: { // Exterior point
            textureStore(df_texture, texture_index, vec4<f32>(probe_radius));
            return;
        }
        case 1u: { // Interior point
            textureStore(df_texture, texture_index, vec4<f32>(-df_grid.offset));
            return;
        }
        default: { // Boundary point
            var distance = compute_distance(grid_point_index);
            textureStore(df_texture, texture_index, vec4<f32>(distance));
            return;
        }
    }
}
