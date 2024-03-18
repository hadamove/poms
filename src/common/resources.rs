use self::{atoms_with_lookup::AtomsWithLookupResource, ses_grid::SesGridResource};

pub mod atoms_with_lookup;
pub mod df_texture;
pub mod ses_grid;

/// For efficiency, some resources (e.g. the molecule) are shared between render and compute passes.
pub struct CommonResources {
    // This makes sense here
    pub molecule_resource: AtomsWithLookupResource,
    pub ses_resource: SesGridResource, // TODO: This can probably be separate for render & compute
}

impl CommonResources {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            ses_resource: SesGridResource::new(device),
            molecule_resource: AtomsWithLookupResource::new(device),
        }
    }
}
