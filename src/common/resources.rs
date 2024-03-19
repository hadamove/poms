use self::{atoms_with_lookup::AtomsWithLookupResource, df_grid::DistanceFieldGridResource};

pub mod atoms_with_lookup;
pub mod df_grid;
pub mod df_texture;

/// For efficiency, some resources (e.g. the molecule) are shared between render and compute passes.
pub struct CommonResources {
    // This makes sense here
    pub molecule_resource: AtomsWithLookupResource,
    pub df_grid_resource: DistanceFieldGridResource, // TODO: This can probably be separate for render & compute
}

impl CommonResources {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            df_grid_resource: DistanceFieldGridResource::new(device),
            molecule_resource: AtomsWithLookupResource::new(device),
        }
    }
}
