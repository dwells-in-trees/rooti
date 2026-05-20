pub(crate) struct SpeciesData {

    // Render agnostic leaf properties
    pub(crate) leaf_placement: LeafPlacement,
    pub(crate) leaf_shape: LeafShape,
    pub(crate) leaf_size_min: f32,
    pub(crate) leaf_size_max: f32,

    // Scaffold for immutable biological properties

    // Scaffold for other properties
}

pub(crate) enum Species {
    Ginkgo,
    // Future tree species will go here
}

#[derive(Clone, Copy)]
pub(crate) enum LeafShape {
    Ginkgo,
}

pub(crate) enum LeafPlacement {
    AtBranchPoints,
    //AtBranchTips,
    //AlongSegments,
}

impl Species {
    pub fn data(&self) -> SpeciesData {
        match self {
            Species::Ginkgo => SpeciesData {
                leaf_placement: LeafPlacement::AtBranchPoints,
                leaf_shape: LeafShape::Ginkgo,
                leaf_size_min: 3.0,
                leaf_size_max: 12.0,
                // Future properties
            },
            // Future species
        }
    }
}