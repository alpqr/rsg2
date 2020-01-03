use crate::scene::*;
use nalgebra_glm as glm;

slotmap::new_key_type! {
    pub struct RSGTransformKey;
}

#[derive(Clone, Copy)]
pub struct RSGTransformComponent {
    pub local_transform: glm::Mat4,
    pub world_transform: glm::Mat4
}

impl RSGTransformComponent {
    pub fn new(local_transform: glm::Mat4) -> Self {
        RSGTransformComponent {
            local_transform: local_transform,
            world_transform: local_transform
        }
    }
}

pub type RSGTransformComponentList = slotmap::SlotMap<RSGTransformKey, RSGTransformComponent>;

slotmap::new_key_type! {
    pub struct RSGOpacityKey;
}

#[derive(Clone, Copy)]
pub struct RSGOpacityComponent {
    pub opacity: f32,
    pub inherited_opacity: f32
}

impl RSGOpacityComponent {
    pub fn new(opacity: f32) -> Self {
        RSGOpacityComponent {
            opacity: opacity,
            inherited_opacity: opacity
        }
    }
}

pub type RSGOpacityComponentList = slotmap::SlotMap<RSGOpacityKey, RSGOpacityComponent>;

slotmap::new_key_type! {
    pub struct RSGMaterialKey;
}

#[derive(Clone, Copy)]
pub struct RSGMaterialComponent {
}

impl RSGMaterialComponent {
    pub fn new() -> Self {
        RSGMaterialComponent {
        }
    }
}

pub type RSGMaterialComponentList = slotmap::SlotMap<RSGMaterialKey, RSGMaterialComponent>;

#[derive(Clone, Debug, PartialEq)]
pub enum RSGMaterialProperty {
    // name, default_value
    Float(String, f32),
    Vec2(String, glm::Vec2),
    Vec3(String, glm::Vec3),
    Vec4(String, glm::Vec4),
    Int(String, i32),
    Int2(String, glm::IVec2),
    Int3(String, glm::IVec3),
    Int4(String, glm::IVec4),
    Mat2(String, glm::Mat2),
    Mat3(String, glm::Mat3),
    Mat4(String, glm::Mat4)
}

#[derive(Clone, Debug, PartialEq)]
pub struct RSGMaterialShaderSet {
    pub vertex_shader: String,
    pub fragment_shader: String,
    pub properties: Vec<RSGMaterialProperty>
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RSGMaterialCustomValue {
    Float(f32),
    Vec2(glm::Vec2),
    Vec3(glm::Vec3),
    Vec4(glm::Vec4),
    Int(i32),
    Int2(glm::IVec2),
    Int3(glm::IVec3),
    Int4(glm::IVec4),
    Mat2(glm::Mat2),
    Mat3(glm::Mat3),
    Mat4(glm::Mat4)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RSGMaterialBuiltinValue {
    ModelMatrix,
    ViewMatrix,
    ProjectionMatrix,
    ModelViewMatrix,
    ViewProjectionMatrix,
    ModelViewProjectionMatrix,
    NormalMatrix
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RSGMaterialPropertyValue {
    Builtin(RSGMaterialBuiltinValue),
    Custom(RSGMaterialCustomValue)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RSGMaterialCullMode {
    None,
    Front,
    Back
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RSGMaterialFrontFace {
    CCW,
    CW
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RSGMaterialCompareOp {
    Never,
    Less,
    Equal,
    LessOrEqual,
    Greater,
    NotEqual,
    GreaterOrEqual,
    Always
}

bitflags::bitflags! {
    pub struct RSGMaterialColorMask: u32 {
        const R = 0x01;
        const G = 0x02;
        const B = 0x04;
        const A = 0x08;
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RSGMaterialBlendFactor {
    Zero,
    One,
    SrcColor,
    OneMinusSrcColor,
    DstColor,
    OneMinusDstColor,
    SrcAlpha,
    OneMinusSrcAlpha,
    DstAlpha,
    OneMinusDstAlpha,
    ConstantColor,
    OneMinusConstantColor,
    ConstantAlpha,
    OneMinusConstantAlpha,
    SrcAlphaSaturate,
    Src1Color,
    OneMinusSrc1Color,
    Src1Alpha,
    OneMinusSrc1Alpha
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RSGMaterialBlendOp {
    Add,
    Subtract,
    ReverseSubtract,
    Min,
    Max
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RSGMaterialBlend {
    pub color_write: RSGMaterialColorMask,
    pub blend_enable: bool,
    pub src_color: RSGMaterialBlendFactor,
    pub dst_color: RSGMaterialBlendFactor,
    pub op_color: RSGMaterialBlendOp,
    pub src_alpha: RSGMaterialBlendFactor,
    pub dst_alpha: RSGMaterialBlendFactor,
    pub op_alpha: RSGMaterialBlendOp
}

impl Default for RSGMaterialBlend {
    fn default() -> Self {
        RSGMaterialBlend {
            color_write: RSGMaterialColorMask::all(),
            blend_enable: false,
            src_color: RSGMaterialBlendFactor::One,
            dst_color: RSGMaterialBlendFactor::OneMinusSrcAlpha,
            op_color: RSGMaterialBlendOp::Add,
            src_alpha: RSGMaterialBlendFactor::One,
            dst_alpha: RSGMaterialBlendFactor::OneMinusSrcAlpha,
            op_alpha: RSGMaterialBlendOp::Add
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RSGMaterialGraphicsState {
    pub depth_test: bool,
    pub depth_write: bool,
    pub depth_op: RSGMaterialCompareOp,
    pub cull_mode: RSGMaterialCullMode,
    pub front_face: RSGMaterialFrontFace,
    pub blend: RSGMaterialBlend
}

impl Default for RSGMaterialGraphicsState {
    fn default() -> Self {
        RSGMaterialGraphicsState {
            depth_test: true,
            depth_write: true,
            depth_op: RSGMaterialCompareOp::Less,
            cull_mode: RSGMaterialCullMode::Back,
            front_face: RSGMaterialFrontFace::CCW,
            blend: Default::default()
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RSGMaterial {
    pub shader_set_id: u32,
    pub property_values: std::collections::HashMap<String, RSGMaterialPropertyValue>,
    pub graphics_state: RSGMaterialGraphicsState
}

impl RSGMaterial {
    pub fn effective_graphics_state(&self, inherited_opacity: f32) -> RSGMaterialGraphicsState {
        let mut state = self.graphics_state;
        let has_transparency = inherited_opacity < 1.0 || state.blend.blend_enable;
        if has_transparency {
            state.depth_write = false;
            if !state.blend.blend_enable {
                state.blend = Default::default();
                state.blend.blend_enable = true;
            }
        }
        state
    }
}

pub type RSGMaterialComponentData = slotmap::SecondaryMap<RSGMaterialKey, RSGMaterial>;

slotmap::new_key_type! {
    pub struct RSGMeshKey;
}

#[derive(Clone, Copy)]
pub struct RSGMeshComponent {
}

impl RSGMeshComponent {
    pub fn new() -> Self {
        RSGMeshComponent {
        }
    }
}

pub type RSGMeshComponentList = slotmap::SlotMap<RSGMeshKey, RSGMeshComponent>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RSGMeshVertexInputType {
    Float,
    Vec2,
    Vec3,
    Vec4,
    Int,
    Int2,
    Int3,
    Int4,
    Mat2,
    Mat3,
    Mat4
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RSGMeshVertexInput {
    // [index,] type, view_index, offset
    Position(RSGMeshVertexInputType, u32, usize),
    Normal(RSGMeshVertexInputType, u32, usize),
    Tangent(RSGMeshVertexInputType, u32, usize),
    Color(u32, RSGMeshVertexInputType, u32, usize),
    TexCoord(u32, RSGMeshVertexInputType, u32, usize),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RSGMeshBufferView {
    pub buffer_id: u32,
    pub offset: usize,
    pub size: usize,
    pub stride: usize
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RSGMeshIndexBufferView {
    U16(RSGMeshBufferView),
    U32(RSGMeshBufferView)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RSGMeshTopology {
    Triangles,
    TriangleStrip,
    Lines,
    LineStrip,
    Points
}

#[derive(Clone, Debug, PartialEq)]
pub struct RSGSubMesh {
    pub topology: RSGMeshTopology,
    pub vertex_count: u32,
    pub inputs: smallvec::SmallVec<[RSGMeshVertexInput; 8]>,
    pub index_count: Option<u32>,
    pub index_view: Option<RSGMeshIndexBufferView>
}

#[derive(Clone, Debug, PartialEq)]
pub struct RSGMesh {
    pub vertex_views: smallvec::SmallVec<[RSGMeshBufferView; 8]>,
    pub submeshes: smallvec::SmallVec<[RSGSubMesh; 1]>,
    pub bounds_3d: Option<RSGAabb>
}

pub type RSGMeshComponentData = slotmap::SecondaryMap<RSGMeshKey, RSGMesh>;

#[derive(Clone, Debug, PartialEq)]
pub struct RSGMeshBuffer {
    pub data: Vec<f32>,
    pub source: String
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RSGAabb {
    pub minimum: glm::Vec3,
    pub maximum: glm::Vec3
}

impl RSGAabb {
    pub fn center(&self) -> glm::Vec3 {
        (self.minimum + self.maximum) * 0.5
    }
}

impl Default for RSGAabb {
    fn default() -> Self {
        RSGAabb { minimum: glm::zero(), maximum: glm::zero() }
    }
}

impl std::fmt::Display for RSGAabb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(min=[{}, {}, {}], max=[{}, {}, {}])",
            self.minimum.x, self.minimum.y, self.minimum.z,
            self.maximum.x, self.maximum.y, self.maximum.z)
    }
}

slotmap::new_key_type! {
    pub struct RSGLayerKey;
}

#[derive(Clone, Copy)]
pub struct RSGLayerComponent {
}

impl RSGLayerComponent {
    pub fn new() -> Self {
        RSGLayerComponent {
        }
    }
}

pub type RSGLayerComponentList = slotmap::SlotMap<RSGLayerKey, RSGLayerComponent>;

#[derive(Clone, Copy, Default)]
pub struct RSGComponentLinks {
    pub transform_key: Option<RSGTransformKey>,
    pub opacity_key: Option<RSGOpacityKey>,
    pub material_key: Option<RSGMaterialKey>,
    pub mesh_key: Option<RSGMeshKey>,
    pub layer_key: Option<RSGLayerKey>
}

#[derive(Default)]
pub struct RSGComponentContainer {
    pub transforms: RSGTransformComponentList,
    pub opacities: RSGOpacityComponentList,
    pub materials: RSGMaterialComponentList,
    pub material_data: RSGMaterialComponentData,
    pub meshes: RSGMeshComponentList,
    pub mesh_data: RSGMeshComponentData,
    pub layers: RSGLayerComponentList
}

impl RSGComponentContainer {
    pub fn add_default_root<ObserverT>(&mut self, scene: &mut RSGScene<RSGComponentLinks, ObserverT>) -> RSGNodeKey
        where ObserverT: RSGObserver
    {
        scene.set_root(RSGNode::with_component_links(
            RSGComponentBuilder::new(self).transform(glm::one()).opacity(1.0).links()))
    }

    pub fn remove(&mut self, component_links: RSGComponentLinks) {
        if let Some(key) = component_links.transform_key {
            self.transforms.remove(key);
        }
        if let Some(key) = component_links.opacity_key {
            self.opacities.remove(key);
        }
        if let Some(key) = component_links.material_key {
            self.materials.remove(key);
        }
        if let Some(key) = component_links.mesh_key {
            self.meshes.remove(key);
        }
        if let Some(key) = component_links.layer_key {
            self.layers.remove(key);
        }
    }

    pub fn is_opaque(&self, links: &RSGComponentLinks) -> bool {
        if let Some(opacity_key) = links.opacity_key {
            if self.opacities[opacity_key].inherited_opacity < 1.0 {
                return false;
            }
        }
        if let Some(material_key) = links.material_key {
            if self.material_data[material_key].graphics_state.blend.blend_enable {
                return false;
            }
        }
        return true;
    }

    pub fn print_scene<ObserverT>(&self, scene: &RSGScene<RSGComponentLinks, ObserverT>,
        start_node_key: RSGNodeKey, max_depth: Option<u32>)
        where ObserverT: RSGObserver
    {
        for (key, depth) in scene.traverse(start_node_key) {
            if max_depth.is_some() && depth > max_depth.unwrap() {
                println!("... <truncated>");
                break;
            }

            let component_links = scene.get_component_links(key);
            let indent = (0..depth).map(|_| "    ").collect::<String>();
            println!("{}----{:?} alpha={}", indent, key, !self.is_opaque(component_links));

            if let Some(transform_key) = component_links.transform_key {
                let t = self.transforms[transform_key];
                println!("{}    local translate=({}, {}, {}) world translate=({}, {}, {})", indent,
                    t.local_transform[12], t.local_transform[13], t.local_transform[14],
                    t.world_transform[12], t.world_transform[13], t.world_transform[14]);
            }

            if let Some(opacity_key) = component_links.opacity_key {
                let o = self.opacities[opacity_key];
                println!("{}    opacity={} inherited opacity={}", indent, o.opacity, o.inherited_opacity);
            }

            if let Some(material_key) = component_links.material_key {
                let material = &self.material_data[material_key];
                println!("{}    material property value count={}", indent, material.property_values.len());
            }

            if let Some(mesh_key) = component_links.mesh_key {
                let mesh = &self.mesh_data[mesh_key];
                println!("{}    mesh submesh count={}", indent, mesh.submeshes.len());
            }

            if let Some(_) = component_links.layer_key {
                println!("{}    layer root", indent);
            }
        }
    }
}

pub struct RSGComponentBuilder<'a> {
    links: RSGComponentLinks,
    container: &'a mut RSGComponentContainer
}

impl<'a> RSGComponentBuilder<'a> {
    pub fn new(container: &'a mut RSGComponentContainer) -> Self {
        RSGComponentBuilder {
            links: Default::default(),
            container: container
        }
    }

    pub fn transform(&mut self, local_transform: glm::Mat4) -> &mut Self {
        self.links.transform_key = Some(self.container.transforms.insert(RSGTransformComponent::new(local_transform)));
        self
    }

    pub fn opacity(&mut self, opacity: f32) -> &mut Self {
        self.links.opacity_key = Some(self.container.opacities.insert(RSGOpacityComponent::new(opacity)));
        self
    }

    pub fn material(&mut self, material: RSGMaterial) -> &mut Self {
        let key = self.container.materials.insert(RSGMaterialComponent::new());
        self.links.material_key = Some(key);
        self.container.material_data.insert(key, material);
        self
    }

    pub fn mesh(&mut self, mesh: RSGMesh) -> &mut Self {
        let key = self.container.meshes.insert(RSGMeshComponent::new());
        self.links.mesh_key = Some(key);
        self.container.mesh_data.insert(key, mesh);
        self
    }

    pub fn layer(&mut self) -> &mut Self {
        self.links.layer_key = Some(self.container.layers.insert(RSGLayerComponent::new()));
        self
    }

    pub fn links(&mut self) -> RSGComponentLinks {
        self.links
    }
}

bitflags::bitflags! {
    pub struct RSGDirtyFlags: u32 {
        const TRANSFORM = 0x01;
        const OPACITY = 0x02;
        const MATERIAL = 0x04;
        const MATERIAL_VALUES = 0x08;
        const MESH = 0x10;
    }
}

pub type RSGDirtySubtreeRootList = smallvec::SmallVec<[RSGNodeKey; 16]>;

#[derive(Debug, Default)]
pub struct RSGSceneObserver {
    pub changed: bool,
    pub hierarchy_changed: bool,
    pub dirty_world_roots: RSGDirtySubtreeRootList,
    pub dirty_opacity_roots: RSGDirtySubtreeRootList,
    pub dirty_material_nodes: RSGDirtySubtreeRootList,
    pub dirty_material_value_nodes: RSGDirtySubtreeRootList,
    pub dirty_mesh_nodes: RSGDirtySubtreeRootList
}

impl RSGObserver for RSGSceneObserver {
    fn notify(&mut self, event: RSGEvent) {
        self.changed = true;
        match event {
            RSGEvent::SubtreeAddedOrReattached(key) => {
                self.hierarchy_changed = true;
                self.dirty_world_roots.push(key);
                self.dirty_opacity_roots.push(key);
                self.dirty_material_nodes.push(key);
                self.dirty_material_value_nodes.push(key);
                self.dirty_mesh_nodes.push(key);
            }
            RSGEvent::SubtreeAboutToBeRemoved(_) => self.hierarchy_changed = true,
            RSGEvent::Dirty(key, f) => {
                match RSGDirtyFlags::from_bits(f) {
                    Some(flags) if flags.contains(RSGDirtyFlags::TRANSFORM) => self.dirty_world_roots.push(key),
                    Some(flags) if flags.contains(RSGDirtyFlags::OPACITY) => self.dirty_opacity_roots.push(key),
                    Some(flags) if flags.contains(RSGDirtyFlags::MATERIAL) => self.dirty_material_nodes.push(key),
                    Some(flags) if flags.contains(RSGDirtyFlags::MATERIAL_VALUES) => self.dirty_material_value_nodes.push(key),
                    Some(flags) if flags.contains(RSGDirtyFlags::MESH) => self.dirty_mesh_nodes.push(key),
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

impl RSGSceneObserver {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn reset(&mut self) {
        self.changed = false;
        self.hierarchy_changed = false;
        self.dirty_world_roots.clear();
        self.dirty_opacity_roots.clear();
        self.dirty_material_nodes.clear();
        self.dirty_material_value_nodes.clear();
        self.dirty_mesh_nodes.clear();
    }
}

fn update_world_transforms<ObserverT>(
    transform_components: RSGTransformComponentList,
    scene: &RSGScene<RSGComponentLinks, ObserverT>,
    subtree_roots: &[RSGNodeKey]) -> RSGTransformComponentList
    where ObserverT: RSGObserver
{
    let mut transforms = transform_components;
    for subtree_root_key in subtree_roots {
        for (key, _) in scene.traverse(*subtree_root_key) {
            if let Some(transform_key) = scene.get_component_links(key).transform_key {
                let mut world_transform = transforms[transform_key].local_transform;
                for key in scene.ancestors(key) {
                    let links = scene.get_component_links(key);
                    if let Some(transform_key) = links.transform_key {
                        world_transform *= transforms[transform_key].world_transform;
                        break;
                    }
                    if links.layer_key.is_some() {
                        break;
                    }
                }
                transforms[transform_key].world_transform = world_transform;
            }
        }
    }
    transforms
}

fn update_inherited_opacities<ObserverT>(
    opacity_components: RSGOpacityComponentList,
    scene: &RSGScene<RSGComponentLinks, ObserverT>,
    subtree_roots: &[RSGNodeKey]) -> RSGOpacityComponentList
    where ObserverT: RSGObserver
{
    let mut opacities = opacity_components;
    for subtree_root_key in subtree_roots {
        for (key, _) in scene.traverse(*subtree_root_key) {
            if let Some(opacity_key) = scene.get_component_links(key).opacity_key {
                let mut inherited_opacity = opacities[opacity_key].opacity;
                for key in scene.ancestors(key) {
                    let links = scene.get_component_links(key);
                    if let Some(opacity_key) = links.opacity_key {
                        inherited_opacity *= opacities[opacity_key].inherited_opacity;
                        break;
                    }
                    if links.layer_key.is_some() {
                        break;
                    }
                }
                opacities[opacity_key].inherited_opacity = inherited_opacity;
            }
        }
    }
    opacities
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RSGOrthographicProjection {
    pub xmag: f32,
    pub ymag: f32,
    pub near: f32,
    pub far: f32
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RSGPerspectiveProjection {
    pub aspect_ratio: f32,
    pub fov: f32,
    pub near: f32,
    pub far: f32
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RSGCamera {
    Orthographic(RSGOrthographicProjection),
    Perspective(RSGPerspectiveProjection)
}

impl Default for RSGCamera {
    fn default() -> Self {
        RSGCamera::Perspective(RSGPerspectiveProjection {
            aspect_ratio: 1.777,
            fov: 45.0,
            near: 0.01,
            far: 1000.0
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RSGCameraWorldTransformDerivedProperties {
    pub position: glm::Vec3,
    pub direction: glm::Vec3
}

impl Default for RSGCameraWorldTransformDerivedProperties {
    fn default() -> Self {
        RSGCameraWorldTransformDerivedProperties {
            position: glm::vec3(0.0, 0.0, 0.0),
            direction: glm::vec3(0.0, 0.0, -1.0)
        }
    }
}

impl RSGCameraWorldTransformDerivedProperties {
    pub fn new(world_transform: &glm::Mat4) -> Self {
        let camera_world = world_transform;
        let camera_position = glm::vec3(camera_world[12], camera_world[13], camera_world[14]);
        let scaling_correct_camera_world = glm::transpose(&glm::inverse(&glm::mat4_to_mat3(&camera_world)));
        let camera_direction = glm::normalize(&(scaling_correct_camera_world * glm::vec3(0.0, 0.0, -1.0)));
        RSGCameraWorldTransformDerivedProperties {
            position: camera_position,
            direction: camera_direction
        }
    }
}

#[inline]
fn calculate_sorting_distance(world_transform: &glm::Mat4, bounds: &RSGAabb,
    camera_properties: &RSGCameraWorldTransformDerivedProperties) -> f32
{
    let center = bounds.center();
    let world_center = glm::vec4_to_vec3(&(world_transform * glm::vec4(center.x, center.y, center.z, 1.0)));
    glm::dot(&(world_center - camera_properties.position), &camera_properties.direction)
}

pub type RSGRenderList = Vec<(RSGNodeKey, f32)>;

pub fn build_render_lists<ObserverT>(
    components: &mut RSGComponentContainer,
    scene: &RSGScene<RSGComponentLinks, ObserverT>,
    start_node_key: RSGNodeKey,
    camera_properties_3d: Option<RSGCameraWorldTransformDerivedProperties>,
    dirty_world_roots: &[RSGNodeKey],
    dirty_opacity_roots: &[RSGNodeKey],
    opaque_list: &mut RSGRenderList,
    alpha_list: &mut RSGRenderList,
    pool: &scoped_pool::Pool)
    where ObserverT: RSGObserver + Sync
{
    pool.scoped(|scope| {
        let (opacity_tx, opacity_rx) = std::sync::mpsc::channel();
        let mut update_opacities = !dirty_opacity_roots.is_empty();
        if update_opacities {
            let opacities = std::mem::replace(&mut components.opacities, Default::default());
            scope.execute(move || {
                opacity_tx.send(update_inherited_opacities(opacities, scene, dirty_opacity_roots)).unwrap();
            });
        }

        let (transform_tx, transform_rx) = std::sync::mpsc::channel();
        let mut update_transforms = !dirty_world_roots.is_empty();
        if update_transforms {
            let transforms = std::mem::replace(&mut components.transforms, Default::default());
            scope.execute(move || {
                transform_tx.send(update_world_transforms(transforms, scene, dirty_world_roots)).unwrap();
            });
        }

        opaque_list.clear();
        alpha_list.clear();

        let mut stacking_order_2d = 0;
        for (key, _) in scene.traverse(start_node_key) {
            let links = scene.get_component_links(key);
            if let Some(mesh_key) = links.mesh_key {
                let mesh_data = components.mesh_data.get(mesh_key).unwrap();
                if update_opacities {
                    components.opacities = opacity_rx.recv().unwrap();
                    update_opacities = false;
                }
                if let Some(cam_props) = camera_properties_3d {
                    if update_transforms {
                        components.transforms = transform_rx.recv().unwrap();
                        update_transforms = false;
                    }
                    let sort_dist = calculate_sorting_distance(
                        &components.transforms[links.transform_key.unwrap()].world_transform,
                        &mesh_data.bounds_3d.unwrap(),
                        &cam_props);
                    if components.is_opaque(links) {
                        // front to back
                        let pos = opaque_list.binary_search_by(|e| e.1.partial_cmp(&sort_dist).unwrap()).unwrap_or_else(|i| i);
                        opaque_list.insert(pos, (key, sort_dist));
                    } else {
                        // back to front
                        let pos = alpha_list.binary_search_by(|e| sort_dist.partial_cmp(&e.1).unwrap()).unwrap_or_else(|i| i);
                        alpha_list.insert(pos, (key, sort_dist));
                    }
                } else {
                    if components.is_opaque(links) {
                        opaque_list.push((key, stacking_order_2d as f32));
                    } else {
                        // tree order is back to front
                        alpha_list.push((key, stacking_order_2d as f32));
                    }
                    stacking_order_2d += 1;
                }
            }
            if links.layer_key.is_some() && key != start_node_key {
                break;
            }
        }

        if camera_properties_3d.is_none() {
            // tree order was back to front, so reverse to get front to back
            opaque_list.reverse();
        }

        if update_opacities {
            components.opacities = opacity_rx.recv().unwrap();
        }
        if update_transforms {
            components.transforms = transform_rx.recv().unwrap();
        }
    });
}
