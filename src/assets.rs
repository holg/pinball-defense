use crate::prelude::*;

#[derive(Resource, AssetCollection)]
pub struct PinballDefenseAssets {
    #[asset(path = "models/pinball_world_1.gltf#Mesh1/Primitive0")]
    pub world_1_mesh: Handle<Mesh>,

    #[asset(path = "models/pinball_world_1.gltf#Mesh0/Primitive0")]
    pub flipper_right: Handle<Mesh>,

    #[asset(path = "models/pinball_world_1.gltf#Mesh2/Primitive0")]
    pub flipper_left: Handle<Mesh>,
}
