use rsg::scene::*;
use rsg::components::*;
use nalgebra_glm as glm;
use smallvec::*;

type Scene = RSGScene::<RSGComponentLinks, RSGSceneObserver>;
type MeshBuffers = std::collections::HashMap<u32, RSGMeshBuffer>;
type ShaderSets = std::collections::HashMap<u32, RSGMaterialShaderSet>;

#[test]
fn scene_with_2d_first_plus_one_3d_layer() {
    static TRIANGLE2D_BUF_ID: u32 = 1;
    static TRIANGLE3D_BUF_ID: u32 = 2;
    static COLOR_SH_ID: u32 = 1;

    fn make_color_material(shader_sets: &mut ShaderSets) -> RSGMaterial {
        let mvp_name = "mvp".to_owned();
        let color_name = "color".to_owned();

        if !shader_sets.contains_key(&COLOR_SH_ID) {
            let shader_set = RSGMaterialShaderSet {
                vertex_shader: "".to_owned(),
                fragment_shader: "".to_owned(),
                properties: vec![
                    RSGMaterialProperty::Mat4(mvp_name.clone(), glm::one()),
                    RSGMaterialProperty::Vec3(color_name.clone(), glm::zero())
                ]
            };
            shader_sets.insert(COLOR_SH_ID, shader_set);
        }

        let mut material = RSGMaterial {
            shader_set_id: COLOR_SH_ID,
            property_values: Default::default(),
            graphics_state: Default::default()
        };
        material.property_values.insert(mvp_name, RSGMaterialPropertyValue::Builtin(RSGMaterialBuiltinValue::ModelViewProjectionMatrix));
        material.property_values.insert(color_name, RSGMaterialPropertyValue::Custom(RSGMaterialCustomValue::Vec3(glm::vec3(1.0, 0.0, 0.0))));
        material
    }

    fn make_2d_triangle(components: &mut RSGComponentContainer, buffers: &mut MeshBuffers, shader_sets: &mut ShaderSets,
        local_transform: glm::Mat4, opacity: f32) -> RSGNode<RSGComponentLinks>
    {
        if !buffers.contains_key(&TRIANGLE2D_BUF_ID) {
            let buf = RSGMeshBuffer {
                data: vec![
                    -1.0, -1.0,
                    1.0, -1.0,
                    0.5, 1.0,
                ],
                source: Default::default()
            };
            buffers.insert(TRIANGLE2D_BUF_ID, buf);
        }

        let mesh = RSGMesh {
            vertex_views: smallvec::smallvec![RSGMeshBufferView {
                buffer_id: TRIANGLE2D_BUF_ID,
                offset: 0,
                size: 6 * 4,
                stride: 2 * 4
            }],
            submeshes: smallvec::smallvec![RSGSubMesh {
                topology: RSGMeshTopology::Triangles,
                vertex_count: 3,
                inputs: smallvec::smallvec![RSGMeshVertexInput::Position(RSGMeshVertexInputType::Vec2, 0, 0)],
                index_count: None,
                index_view: None
            }],
            bounds_3d: None
        };

        let material = make_color_material(shader_sets);

        RSGNode::with_component_links(
            RSGComponentBuilder::new(components)
            .transform(local_transform)
            .opacity(opacity)
            .material(material)
            .mesh(mesh)
            .links())
    }

    fn make_3d_triangle(components: &mut RSGComponentContainer, buffers: &mut MeshBuffers, shader_sets: &mut ShaderSets,
        local_transform: glm::Mat4, opacity: f32) -> RSGNode<RSGComponentLinks>
    {
        if !buffers.contains_key(&TRIANGLE3D_BUF_ID) {
            let buf = RSGMeshBuffer {
                data: vec![
                    -1.0, -1.0, 0.0,
                    1.0, -1.0, 0.0,
                    0.5, 1.0, 0.0,
                ],
                source: Default::default()
            };
            buffers.insert(TRIANGLE3D_BUF_ID, buf);
        }

        let mesh = RSGMesh {
            vertex_views: smallvec::smallvec![RSGMeshBufferView {
                buffer_id: TRIANGLE3D_BUF_ID,
                offset: 0,
                size: 9 * 4,
                stride: 3 * 4
            }],
            submeshes: smallvec::smallvec![RSGSubMesh {
                topology: RSGMeshTopology::Triangles,
                vertex_count: 3,
                inputs: smallvec::smallvec![RSGMeshVertexInput::Position(RSGMeshVertexInputType::Vec3, 0, 0)],
                index_count: None,
                index_view: None
            }],
            bounds_3d: Some(RSGAabb {
                minimum: glm::vec3(-1.0, -1.0, 0.0),
                maximum: glm::vec3(1.0, 1.0, 0.0)
            }),
        };

        let material = make_color_material(shader_sets);

        RSGNode::with_component_links(
            RSGComponentBuilder::new(components)
            .transform(local_transform)
            .opacity(opacity)
            .material(material)
            .mesh(mesh)
            .links())
    }

    #[derive(Default)]
    struct Data {
        components: RSGComponentContainer,
        mesh_buffers: MeshBuffers,
        shader_sets: ShaderSets,
        opaque_list_2d: RSGRenderList,
        alpha_list_2d: RSGRenderList,
        opaque_list_3d: RSGRenderList,
        alpha_list_3d: RSGRenderList,
        camera_3d: RSGCamera,
        camera_3d_properties: RSGCameraWorldTransformDerivedProperties,
        root_key: RSGNodeKey,
        layer3d_key: RSGNodeKey,
        frame_count: u32,

        // just for the tests's sake
        tri1_key: RSGNodeKey,
        tri2_key: RSGNodeKey,
        tri3_key: RSGNodeKey,
        tri_alpha1_key: RSGNodeKey,
        tri_alpha2_key: RSGNodeKey,
        tri_3d1_key: RSGNodeKey,
        tri_3d2_key: RSGNodeKey,
        tri_3d_alpha1_key: RSGNodeKey,
        tri_3d_alpha2_key: RSGNodeKey,
    }

    fn sync(d: &mut Data, scene: &mut Scene) {
        println!("Frame {} sync", d.frame_count);
        if d.frame_count == 0 {
            let mut transaction = RSGSubtreeAddTransaction::new();
            // 2D, opaque
            d.tri1_key = scene.append_with_transaction(d.root_key, make_2d_triangle(&mut d.components, &mut d.mesh_buffers, &mut d.shader_sets,
                glm::translation(&glm::vec3(50.0, 100.0, 0.0)), 1.0),
                &mut transaction);
            d.tri2_key = scene.append_with_transaction(d.tri1_key, make_2d_triangle(&mut d.components, &mut d.mesh_buffers, &mut d.shader_sets,
                glm::translation(&glm::vec3(10.0, 20.0, 0.0)), 1.0),
                &mut transaction);
            d.tri3_key = scene.append_with_transaction(d.tri2_key, make_2d_triangle(&mut d.components, &mut d.mesh_buffers, &mut d.shader_sets,
                glm::translation(&glm::vec3(-5.0, 0.0, 0.0)), 1.0),
                &mut transaction);
            // 2D, alpha
            d.tri_alpha1_key = scene.append_with_transaction(d.tri1_key, make_2d_triangle(&mut d.components, &mut d.mesh_buffers, &mut d.shader_sets,
                glm::translation(&glm::vec3(25.0, 32.0, 0.0)), 0.8),
                &mut transaction);
            d.tri_alpha2_key = scene.append_with_transaction(d.tri_alpha1_key, make_2d_triangle(&mut d.components, &mut d.mesh_buffers, &mut d.shader_sets,
                glm::translation(&glm::vec3(50.0, 100.0, 0.0)), 1.0),
                &mut transaction);
            // throw in some 3D stuff, with a layer component only node acting as the "barrier"
            d.layer3d_key = scene.append_with_transaction(d.tri_alpha1_key,
                RSGNode::with_component_links(RSGComponentBuilder::new(&mut d.components).layer().links()),
                &mut transaction);
            d.tri_3d1_key = scene.append_with_transaction(d.layer3d_key, make_3d_triangle(&mut d.components, &mut d.mesh_buffers, &mut d.shader_sets,
                glm::translation(&glm::vec3(0.0, 0.0, -1.0)), 1.0),
                &mut transaction);
            d.tri_3d2_key = scene.append_with_transaction(d.tri_3d1_key, make_3d_triangle(&mut d.components, &mut d.mesh_buffers, &mut d.shader_sets,
                glm::translation(&glm::vec3(0.5, 0.5, -5.0)), 1.0),
                &mut transaction);
            d.tri_3d_alpha1_key = scene.append_with_transaction(d.tri_3d1_key, make_3d_triangle(&mut d.components, &mut d.mesh_buffers, &mut d.shader_sets,
                glm::translation(&glm::vec3(-1.5, 0.0, -2.0)), 0.5),
                &mut transaction);
            d.tri_3d_alpha2_key = scene.append_with_transaction(d.tri_3d_alpha1_key, make_3d_triangle(&mut d.components, &mut d.mesh_buffers, &mut d.shader_sets,
                glm::translation(&glm::vec3(0.0, 1.0, 1.0)), 0.2),
                &mut transaction);
            scene.commit(transaction);

            d.camera_3d = RSGCamera::Perspective(RSGPerspectiveProjection {
                aspect_ratio: 1.777,
                fov: 45.0,
                near: 0.01,
                far: 1000.0
            });
            d.camera_3d_properties = RSGCameraWorldTransformDerivedProperties::new(&glm::translation(&glm::vec3(0.0, 0.0, 600.0)));
        }
    }

    fn prepare(d: &mut Data, scene: &Scene, observer: &RSGSceneObserver, pool: &scoped_pool::Pool) {
        println!("Frame {} prepare, changes={:?}", d.frame_count, observer);
        if observer.changed {
            build_render_lists(&mut d.components, &scene, d.root_key, None,
                &observer.dirty_world_roots, &observer.dirty_opacity_roots,
                &mut d.opaque_list_2d, &mut d.alpha_list_2d,
                &pool);
            build_render_lists(&mut d.components, &scene, d.layer3d_key, Some(d.camera_3d_properties),
                &[], &[],
                &mut d.opaque_list_3d, &mut d.alpha_list_3d,
                &pool);
        }
    }

    fn render(d: &mut Data, scene: &Scene) {
        println!("Frame {} render", d.frame_count);
        d.components.print_scene(scene, d.root_key, Some(10));
        println!("  2D opaque list={:?}", d.opaque_list_2d);
        println!("  2D alpha list={:?}", d.alpha_list_2d);
        println!("  3D camera={:?} {:?}", d.camera_3d, d.camera_3d_properties);
        println!("  3D opaque list={:?}", d.opaque_list_3d);
        println!("  3D alpha list={:?}", d.alpha_list_3d);

        assert!(d.opaque_list_2d == vec![
            (d.tri3_key, 2.0),
            (d.tri2_key, 1.0),
            (d.tri1_key, 0.0)
        ]);
        assert!(d.alpha_list_2d == vec![
            (d.tri_alpha1_key, 3.0),
            (d.tri_alpha2_key, 4.0),
        ]);
        assert!(d.opaque_list_3d == vec![
            (d.tri_3d1_key, 601.0),
            (d.tri_3d2_key, 606.0)
        ]);
        assert!(d.alpha_list_3d == vec![
            (d.tri_3d_alpha1_key, 603.0),
            (d.tri_3d_alpha2_key, 602.0),
        ]);
    }

    fn frame(d: &mut Data, scene: &mut Scene, pool: &scoped_pool::Pool) {
        let mut observer = RSGSceneObserver::new();
        scene.set_observer(observer);
        sync(d, scene);
        observer = scene.take_observer().unwrap();
        prepare(d, scene, &observer, pool);
        render(d, scene);
        d.frame_count += 1;
    }

    let pool = scoped_pool::Pool::new(4);
    let mut scene = Scene::new();
    let mut d: Data = Default::default();
    d.root_key = d.components.add_default_root(&mut scene);

    frame(&mut d, &mut scene, &pool);

    pool.shutdown();
}
