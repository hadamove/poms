struct GridUniform {
    origin: vec4<f32>,
    resolution: u32,
    offset: f32,
    size: f32,
    // Add 4 bytes padding to avoid alignment issues.
    _padding: f32,
};


@group(0) @binding(0) var<uniform> ses_grid: GridUniform;
@group(0) @binding(1) var<uniform> probe_radius: f32;
@group(0) @binding(2) var<uniform> grid_point_index_offset: u32;

@group(1) @binding(3) var<storage, read_write> grid_point_class: array<u32>;
@group(1) @binding(4) var<storage, read_write> predecessor: array<u32>;

@group(2) @binding(0) var distance_texture: texture_storage_3d<rgba16float, write>;

fn grid_point_index_to_position(grid_point_index: u32) -> vec3<f32> {
    return ses_grid.origin.xyz + vec3<f32>(
        f32(grid_point_index % ses_grid.resolution),
        f32((grid_point_index / ses_grid.resolution) % ses_grid.resolution),
        f32(grid_point_index / (ses_grid.resolution * ses_grid.resolution))
    ) * ses_grid.offset;
}


fn compute_distance(grid_point_index: u32) -> f32 {
    var HUGE_DISTANCE = 100000.0;

    var grid_point: vec3<f32> = grid_point_index_to_position(grid_point_index);
    var res = i32(ses_grid.resolution);

    // We need to check points at maximum probe_radius distance from the current point.
    var search_range: i32 = 1;
    var min_distance = HUGE_DISTANCE;
    var min_neighbor = 0u;

    // Loop over all neighboring points in the search range, find the closest exterior point.
    for (var x: i32 = -search_range; x <= search_range; x = x + 1) {
        for (var y: i32 = -search_range; y <= search_range; y = y + 1) {
            for (var z: i32 = -search_range; z <= search_range; z = z + 1) {
                var neighbor_index = u32(i32(grid_point_index) + x + y * res + z * res * res);

                if neighbor_index >= arrayLength(&grid_point_class) {
                    continue;
                }

                var neighbor_point = grid_point_index_to_position(neighbor_index);
                var distance = HUGE_DISTANCE;

                if grid_point_class[neighbor_index] == 0u {
                    distance = length(neighbor_point - grid_point);
                } else if predecessor[neighbor_index] != 0u {
                    var pred = grid_point_index_to_position(predecessor[neighbor_index] - 1u);
                    distance = length(pred - grid_point);
                }

                if distance < min_distance {
                    min_distance = distance;

                    if grid_point_class[neighbor_index] == 0u {
                        predecessor[grid_point_index] = neighbor_index + 1u;
                    } else {
                        predecessor[grid_point_index] = predecessor[neighbor_index];
                    }
                }
            }
        }
    }
    if min_distance < HUGE_DISTANCE + 1.0 {
        return probe_radius - min_distance;
    }
    // No exterior point found.
    return -ses_grid.offset;
}


@compute @workgroup_size(64)
fn main(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    var grid_point_index: u32 = global_invocation_id.x + grid_point_index_offset;
    var total = ses_grid.resolution * ses_grid.resolution * ses_grid.resolution;
    if (grid_point_index >= total) {
        return;
    }

    var texture_index = vec3<i32>(
        i32(grid_point_index % ses_grid.resolution),
        i32((grid_point_index / ses_grid.resolution) % ses_grid.resolution),
        i32(grid_point_index / (ses_grid.resolution * ses_grid.resolution))
    );

    // Switch cases with constants are not supported yet.
    switch (grid_point_class[grid_point_index]) {
        case 0u: { // Exterior point
            textureStore(distance_texture, texture_index, vec4<f32>(probe_radius));
            return;
        }
        case 1u: { // Interior point
            textureStore(distance_texture, texture_index, vec4<f32>(-ses_grid.offset));
            return;
        }
        case 2u: { // Boundary point
            var distance = compute_distance(grid_point_index);
            textureStore(distance_texture, texture_index, vec4<f32>(distance));
            return;
        }
        default: {}
    }
}
