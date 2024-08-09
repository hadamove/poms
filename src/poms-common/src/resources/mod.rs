use atoms_with_lookup::AtomsWithLookupResource;

pub mod atoms_with_lookup;
pub mod df_texture;

/// For efficiency, some resources (e.g. the molecule) are shared between render and compute passes.
pub struct CommonResources {
    pub atoms_resource: AtomsWithLookupResource,
}

impl CommonResources {
    /// Creates a new instance of `CommonResources`.
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            atoms_resource: AtomsWithLookupResource::new(device),
        }
    }
}
