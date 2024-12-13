use std::borrow::{Borrow, BorrowMut};
use std::rc::{Rc, Weak};
use std::str::FromStr;
use std::sync::{Arc, Mutex, MutexGuard};

use bytemuck::Contiguous;
use cgmath::Vector4;
use editor_state::{EditorState, PolygonEdit, RecordState};
use floem::common::{nav_button, option_button, rgb_to_wgpu, small_button};
use floem::kurbo::Size;
use floem::window::WindowConfig;
use floem_renderer::gpu_resources::{self, GpuResources};
use floem_winit::dpi::{LogicalSize, PhysicalSize};
use floem_winit::event::{ElementState, KeyEvent, Modifiers, MouseButton, MouseScrollDelta};
use helpers::utilities::load_ground_truth_state;
use stunts_engine::camera::{Camera, CameraBinding};
use stunts_engine::dot::draw_dot;
use stunts_engine::editor::{point_to_ndc, Editor, Point, Viewport, WindowSize, WindowSizeShader};
use stunts_engine::polygon::Polygon;
use stunts_engine::vertex::Vertex;
use uuid::Uuid;
use views::app::app_view;
// use winit::{event_loop, window};
use wgpu::util::DeviceExt;

use floem::context::PaintState;
use floem::EngineHandle;
use floem::{Application, CustomRenderCallback};
use floem::{GpuHelper, View, WindowHandle};
use undo::{Edit, Record};

use std::ops::Not;

use cgmath::InnerSpace;
use cgmath::SquareMatrix;
use cgmath::Transform;
use cgmath::{Matrix4, Point3, Vector3};

mod editor_state;
mod helpers;
mod views;

// Usage in render pass:
pub fn render_ray_intersection(
    render_pass: &mut wgpu::RenderPass,
    device: &wgpu::Device,
    window_size: &WindowSize,
    editor: &Editor,
    camera: &Camera,
) {
    // if let ray = visualize_ray_intersection(window_size, editor.last_x, editor.last_y, camera) {
    let (vertices, indices, vertex_buffer, index_buffer) = draw_dot(
        device,
        window_size,
        Point {
            x: editor.ds_ndc_pos.x,
            y: editor.ds_ndc_pos.y,
        },
        rgb_to_wgpu(47, 131, 222, 1.0), // Blue dot
        camera,
    );

    // println!("render ray");
    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
    render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
    render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
    // }
}

pub fn get_sensor_editor(handle: &EngineHandle) -> Option<Arc<Mutex<Editor>>> {
    handle.user_editor.as_ref().and_then(|e| {
        // let guard = e.lock().ok()?;
        let cloned = e.downcast_ref::<Arc<Mutex<Editor>>>().cloned();
        // drop(guard);
        cloned
    })
}

type RenderCallback<'a> = dyn for<'b> Fn(
        wgpu::CommandEncoder,
        wgpu::SurfaceTexture,
        Arc<wgpu::TextureView>,
        Arc<wgpu::TextureView>,
        // &WindowHandle,
        &Arc<GpuResources>,
        &EngineHandle,
    ) -> (
        Option<wgpu::CommandEncoder>,
        Option<wgpu::SurfaceTexture>,
        Option<Arc<wgpu::TextureView>>,
        Option<Arc<wgpu::TextureView>>,
    ) + 'a;

fn create_render_callback<'a>() -> Box<RenderCallback<'a>> {
    Box::new(
        move |mut encoder: wgpu::CommandEncoder,
              frame: wgpu::SurfaceTexture,
              view: Arc<wgpu::TextureView>,
              resolve_view: Arc<wgpu::TextureView>,
              //   window_handle: &WindowHandle
              gpu_resources: &Arc<GpuResources>,
              engine_handle: &EngineHandle| {
            // let mut handle = window_handle.borrow();
            let mut editor = get_sensor_editor(engine_handle);
            // let mut engine = editor
            //     .as_mut()
            //     .expect("Couldn't get user engine")
            //     .lock()
            //     .unwrap();

            // if let Some(gpu_resources) = &handle.gpu_resources {
            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: Some(&resolve_view),
                        ops: wgpu::Operations {
                            // load: wgpu::LoadOp::Clear(wgpu::Color {
                            //     // grey background
                            //     r: 0.15,
                            //     g: 0.15,
                            //     b: 0.15,
                            //     // white background
                            //     // r: 1.0,
                            //     // g: 1.0,
                            //     // b: 1.0,
                            //     a: 1.0,
                            // }),
                            // load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                            // load: wgpu::LoadOp::Load,
                            // store: wgpu::StoreOp::Store,
                            load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    // depth_stencil_attachment: None,
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &engine_handle
                            .gpu_helper
                            .as_ref()
                            .expect("Couldn't get gpu helper")
                            .lock()
                            .unwrap()
                            .depth_view
                            .as_ref()
                            .expect("Couldn't fetch depth view"), // This is the depth texture view
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0), // Clear to max depth
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None, // Set this if using stencil
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                // println!("Render frame...");

                // Render partial screen content
                // render_pass.set_viewport(100.0, 100.0, 200.0, 200.0, 0.0, 1.0);
                // render_pass.set_scissor_rect(100, 100, 200, 200);

                render_pass.set_pipeline(
                    &engine_handle
                        .render_pipeline
                        .as_ref()
                        .expect("Couldn't fetch render pipeline"),
                );

                // let editor = handle
                //     .user_editor
                //     .as_ref()
                //     .expect("Couldn't get user editor")
                //     .lock()
                //     .unwrap();
                let editor = get_sensor_editor(engine_handle);
                // does this freeze?
                let mut editor = editor
                    .as_ref()
                    .expect("Couldn't get user engine")
                    .lock()
                    .unwrap();

                let camera = editor.camera.expect("Couldn't get camera");

                editor.step_motion_path_animations(&camera);

                let camera_binding = editor
                    .camera_binding
                    .as_ref()
                    .expect("Couldn't get camera binding");

                // camera_binding.update(&gpu_resources.queue, &editor.camera);
                // editor.update_camera_binding(&gpu_resources.queue);

                render_pass.set_bind_group(0, &camera_binding.bind_group, &[]);
                render_pass.set_bind_group(
                    2,
                    editor
                        .window_size_bind_group
                        .as_ref()
                        .expect("Couldn't get window size group"),
                    &[],
                );

                for (poly_index, polygon) in editor.polygons.iter().enumerate() {
                    polygon
                        .transform
                        .update_uniform_buffer(&gpu_resources.queue, &camera.window_size);
                    render_pass.set_bind_group(1, &polygon.bind_group, &[]);
                    render_pass.set_vertex_buffer(0, polygon.vertex_buffer.slice(..));
                    render_pass.set_index_buffer(
                        polygon.index_buffer.slice(..),
                        wgpu::IndexFormat::Uint32,
                    );
                    render_pass.draw_indexed(0..polygon.indices.len() as u32, 0, 0..1);
                }

                let viewport = editor.viewport.lock().unwrap();
                let window_size = WindowSize {
                    width: viewport.width as u32,
                    height: viewport.height as u32,
                };

                // println!("Render size {:?}", window_size);

                // let camera = editor.camera.expect("Couldn't get camera");

                // let ndc_position = point_to_ndc(editor.last_top_left, &window_size);
                // let (vertices, indices, vertex_buffer, index_buffer) = draw_dot(
                //     &gpu_resources.device,
                //     &window_size,
                //     Point {
                //         x: ndc_position.x,
                //         y: ndc_position.y,
                //     },
                //     rgb_to_wgpu(47, 131, 222, 1.0),
                //     &camera,
                // ); // Green dot

                // render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                // render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                // render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
            }

            (Some(encoder), Some(frame), Some(view), Some(resolve_view))
        },
    )
}

fn handle_cursor_moved(
    editor: std::sync::Arc<Mutex<Editor>>,
    gpu_resources: std::sync::Arc<GpuResources>,
    // window_size: WindowSize,
    viewport: std::sync::Arc<Mutex<Viewport>>,
) -> Option<Box<dyn Fn(f64, f64, f64, f64)>> {
    Some(Box::new(
        move |positionX: f64, positionY: f64, logPosX: f64, logPoxY: f64| {
            let mut editor = editor.lock().unwrap();
            let viewport = viewport.lock().unwrap();
            let window_size = WindowSize {
                width: viewport.width as u32,
                height: viewport.height as u32,
            };
            // println!("window size {:?}", window_size);
            // println!("positions {:?} {:?}", positionX, positionY);
            editor.handle_mouse_move(
                &window_size,
                &gpu_resources.device,
                positionX as f32,
                positionY as f32,
            );
            // TODO: need callback for when cursor is done moving, then add translation to undo stack
        },
    ))
}

fn handle_mouse_input(
    mut editor_state: Arc<Mutex<EditorState>>,
    editor: std::sync::Arc<Mutex<Editor>>,
    gpu_resources: std::sync::Arc<GpuResources>,
    // window_size: WindowSize,
    viewport: std::sync::Arc<Mutex<Viewport>>,
    record: Arc<Mutex<Record<PolygonEdit>>>,
) -> Option<Box<dyn Fn(MouseButton, ElementState)>> {
    Some(Box::new(move |button, state| {
        let mut editor_orig = Arc::clone(&editor);
        let mut editor = editor.lock().unwrap();
        let viewport = viewport.lock().unwrap();
        let window_size = WindowSize {
            width: viewport.width as u32,
            height: viewport.height as u32,
        };
        if button == MouseButton::Left {
            let edit_config = match state {
                ElementState::Pressed => editor.handle_mouse_down(
                    // mouse_position.0,
                    // mouse_position.1,
                    &window_size,
                    &gpu_resources.device,
                ),
                ElementState::Released => editor.handle_mouse_up(),
            };

            drop(editor);

            // if (edit_config.is_some()) {
            //     let edit_config = edit_config.expect("Couldn't get polygon edit config");

            //     let mut editor_state = editor_state.lock().unwrap();

            //     let edit = PolygonEdit {
            //         polygon_id: edit_config.polygon_id,
            //         old_value: edit_config.old_value,
            //         new_value: edit_config.new_value,
            //         field_name: edit_config.field_name,
            //         signal: None,
            //     };

            //     let mut record_state = RecordState {
            //         editor: editor_orig,
            //         // record: Arc::clone(&record),
            //     };

            //     let mut record = record.lock().unwrap();
            //     record.edit(&mut record_state, edit);
            // }
        }
    }))
}

fn handle_window_resize(
    editor: std::sync::Arc<Mutex<Editor>>,
    gpu_resources: std::sync::Arc<GpuResources>,
    // window_size: WindowSize, // need newest window size
    gpu_helper: std::sync::Arc<Mutex<GpuHelper>>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
) -> Option<Box<dyn FnMut(PhysicalSize<u32>, LogicalSize<f64>)>> {
    Some(Box::new(move |size, logical_size| {
        let mut editor = editor.lock().unwrap();

        let window_size = WindowSize {
            width: size.width,
            height: size.height,
        };

        let mut viewport = viewport.lock().unwrap();

        viewport.width = size.width as f32;
        viewport.height = size.height as f32;

        let mut camera = editor.camera.expect("Couldn't get camera on resize");

        camera.window_size.width = size.width;
        camera.window_size.height = size.height;

        // editor.update_date_from_window_resize(&window_size, &gpu_resources.device);

        gpu_helper
            .lock()
            .unwrap()
            .recreate_depth_view(&gpu_resources, size.width, size.height);
    }))
}

fn handle_mouse_wheel(
    editor: std::sync::Arc<Mutex<Editor>>,
    gpu_resources: std::sync::Arc<GpuResources>,
    // window_size: WindowSize, // need newest window size
    // gpu_helper: std::sync::Arc<Mutex<GpuHelper>>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
) -> Option<Box<dyn FnMut(MouseScrollDelta)>> {
    Some(Box::new(move |delta: MouseScrollDelta| {
        let mut editor = editor.lock().unwrap();

        // let mouse_pos = Point {
        //     x: editor.global_top_left.x,
        //     y: editor.global_top_left.y,
        // };

        // match delta {
        //     MouseScrollDelta::LineDelta(_x, y) => {
        //         // y is positive for scrolling up/away from user
        //         // negative for scrolling down/toward user
        //         // let zoom_factor = if y > 0.0 { 1.1 } else { 0.9 };
        //         editor.handle_wheel(y, mouse_pos, &gpu_resources.queue);
        //     }
        //     MouseScrollDelta::PixelDelta(pos) => {
        //         // Convert pixel delta if needed
        //         let y = pos.y as f32;
        //         // let zoom_factor = if y > 0.0 { 1.1 } else { 0.9 };
        //         editor.handle_wheel(y, mouse_pos, &gpu_resources.queue);
        //     }
        // }
    }))
}

fn handle_modifiers_changed(
    // editor: std::sync::Arc<Mutex<common_vector::editor::Editor>>,
    editor_state: std::sync::Arc<Mutex<EditorState>>,
    gpu_resources: std::sync::Arc<GpuResources>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
) -> Option<Box<dyn FnMut(Modifiers)>> {
    Some(Box::new(move |modifiers: Modifiers| {
        let mut editor_state = editor_state.lock().unwrap();
        println!("modifiers changed");
        let modifier_state = modifiers.state();
        editor_state.current_modifiers = modifier_state;
    }))
}

use floem_winit::keyboard::NamedKey;
use floem_winit::keyboard::{Key, SmolStr};

fn handle_keyboard_input(
    // editor: std::sync::Arc<Mutex<common_vector::editor::Editor>>,
    editor_state: std::sync::Arc<Mutex<EditorState>>,
    gpu_resources: std::sync::Arc<GpuResources>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
) -> Option<Box<dyn FnMut(KeyEvent)>> {
    Some(Box::new(move |event: KeyEvent| {
        if event.state != ElementState::Pressed {
            return;
        }

        let mut editor_state = editor_state.lock().unwrap();
        // let editor: MutexGuard<'_, Editor> = editor_state.editor.lock().unwrap();
        // Check for Ctrl+Z (undo)
        let modifiers = editor_state.current_modifiers;

        // match event.logical_key {
        //     Key::Character(c) if c == SmolStr::new("z") => {
        //         if modifiers.control_key() {
        //             if modifiers.shift_key() {
        //                 editor_state.redo(); // Ctrl+Shift+Z
        //             } else {
        //                 println!("undo!");
        //                 editor_state.undo(); // Ctrl+Z
        //             }
        //         }
        //     }
        //     Key::Character(c) if c == SmolStr::new("y") => {
        //         if modifiers.control_key() {
        //             editor_state.redo(); // Ctrl+Y
        //         }
        //     }
        //     _ => {}
        // }
    }))
}

#[tokio::main]
async fn main() {
    println!("Initializing Stunts...");

    let app = Application::new();

    // Get the primary monitor's size
    let monitor = app.primary_monitor().expect("Couldn't get primary monitor");
    let monitor_size = monitor.size();

    // Calculate a reasonable window size (e.g., 80% of the screen size)
    let window_width = (monitor_size.width.into_integer() as f32 * 0.8) as u32;
    let window_height = (monitor_size.height.into_integer() as f32 * 0.8) as u32;

    let window_size = WindowSize {
        width: window_width,
        height: window_height,
    };

    let mut gpu_helper = Arc::new(Mutex::new(GpuHelper::new()));

    let gpu_cloned = Arc::clone(&gpu_helper);
    let gpu_clonsed2 = Arc::clone(&gpu_helper);
    let gpu_cloned3 = Arc::clone(&gpu_helper);

    let viewport = Arc::new(Mutex::new(Viewport::new(
        window_size.width as f32,
        window_size.height as f32,
    )));

    let mut editor = Arc::new(Mutex::new(Editor::new(viewport.clone())));

    let cloned_viewport = Arc::clone(&viewport);
    let cloned_viewport2 = Arc::clone(&viewport);
    let cloned_viewport3 = Arc::clone(&viewport);

    // let cloned_handler = Arc::clone(&handler);
    // let cloned_square_handler = Arc::clone(&square_handler);
    // let cloned_square_handler6 = Arc::clone(&square_handler);

    let cloned = Arc::clone(&editor);
    let cloned2 = Arc::clone(&editor);
    let cloned3 = Arc::clone(&editor);
    let cloned4 = Arc::clone(&editor);
    let cloned5 = Arc::clone(&editor);
    // let cloned6 = Arc::clone(&editor);
    let cloned7 = Arc::clone(&editor);
    // let cloned8 = Arc::clone(&editor);
    // let cloned9 = Arc::clone(&editor);
    // let cloned10 = Arc::clone(&editor);
    let cloned11 = Arc::clone(&editor);
    let cloned12 = Arc::clone(&editor);
    let cloned13 = Arc::clone(&editor);

    let record = Arc::new(Mutex::new(Record::new()));

    let record_2 = Arc::clone(&record);

    let editor_state = Arc::new(Mutex::new(EditorState::new(cloned4, record)));

    let state_2 = Arc::clone(&editor_state);
    let state_3 = Arc::clone(&editor_state);
    let state_4 = Arc::clone(&editor_state);
    let state_5 = Arc::clone(&editor_state);

    // load saved state (no projects as Ground Truth)
    println!("Loading saved state...");
    let saved_state = load_ground_truth_state().expect("Couldn't get Saved State");
    let mut state_guard = state_5.lock().unwrap();
    state_guard.saved_state = Some(saved_state.clone());
    drop(state_guard);

    let (mut app, window_id) = app.window(
        move |_| {
            app_view(
                Arc::clone(&editor_state),
                Arc::clone(&editor),
                Arc::clone(&gpu_helper),
                Arc::clone(&viewport),
            )
        },
        Some(
            WindowConfig::default()
                .size(Size::new(
                    window_size.width as f64,
                    window_size.height as f64,
                ))
                .title("CommonOS Stunts"),
        ),
    );

    let window_id = window_id.expect("Couldn't get window id");

    {
        let app_handle = app.handle.as_mut().expect("Couldn't get handle");
        let window_handle = app_handle
            .window_handles
            .get_mut(&window_id)
            .expect("Couldn't get window handle");

        // Create and set the render callback
        let render_callback = create_render_callback();

        // window_handle.set_render_callback(render_callback);
        window_handle.set_encode_callback(render_callback);
        // window_handle.window_size = Some(window_size);
        window_handle.window_width = Some(window_size.width);
        window_handle.window_height = Some(window_size.height);

        println!("Ready...");

        // window_handle.user_editor = Some(Box::new(cloned));

        // Receive and store GPU resources
        // match &mut window_handle.paint_state {
        //     PaintState::PendingGpuResources { rx, .. } => {
        if let PaintState::PendingGpuResources { rx, .. } = &mut window_handle.paint_state {
            async {
                let gpu_resources = Arc::new(rx.recv().unwrap().unwrap());

                println!("Initializing pipeline...");

                // let mut editor = cloned11.lock().unwrap();
                let mut editor = cloned5.lock().unwrap();

                let camera = Camera::new(window_size);
                let camera_binding = CameraBinding::new(&gpu_resources.device);

                editor.camera = Some(camera);
                editor.camera_binding = Some(camera_binding);

                let sampler = gpu_resources
                    .device
                    .create_sampler(&wgpu::SamplerDescriptor {
                        address_mode_u: wgpu::AddressMode::ClampToEdge,
                        address_mode_v: wgpu::AddressMode::ClampToEdge,
                        mag_filter: wgpu::FilterMode::Linear,
                        min_filter: wgpu::FilterMode::Linear,
                        mipmap_filter: wgpu::FilterMode::Nearest,
                        ..Default::default()
                    });

                gpu_cloned.lock().unwrap().recreate_depth_view(
                    &gpu_resources,
                    window_size.width,
                    window_size.height,
                );

                let depth_stencil_state = wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24Plus,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                };

                let camera_binding = editor
                    .camera_binding
                    .as_ref()
                    .expect("Couldn't get camera binding");

                let model_bind_group_layout = gpu_resources.device.create_bind_group_layout(
                    &wgpu::BindGroupLayoutDescriptor {
                        entries: &[wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        }],
                        label: Some("model_bind_group_layout"),
                    },
                );

                let model_bind_group_layout = Arc::new(model_bind_group_layout);

                let window_size_buffer =
                    gpu_resources
                        .device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: None,
                            contents: bytemuck::cast_slice(&[WindowSizeShader {
                                width: window_size.width as f32,
                                height: window_size.height as f32,
                            }]),
                            usage: wgpu::BufferUsages::UNIFORM,
                        });

                let window_size_bind_group_layout = gpu_resources.device.create_bind_group_layout(
                    &wgpu::BindGroupLayoutDescriptor {
                        label: None,
                        entries: &[wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        }],
                    },
                );

                let window_size_bind_group =
                    gpu_resources
                        .device
                        .create_bind_group(&wgpu::BindGroupDescriptor {
                            layout: &window_size_bind_group_layout,
                            entries: &[wgpu::BindGroupEntry {
                                binding: 0,
                                resource: window_size_buffer.as_entire_binding(),
                            }],
                            label: None,
                        });

                // Define the layouts
                let pipeline_layout =
                    gpu_resources
                        .device
                        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                            label: Some("Pipeline Layout"),
                            // bind_group_layouts: &[&bind_group_layout],
                            bind_group_layouts: &[
                                &camera_binding.bind_group_layout,
                                &model_bind_group_layout,
                                &window_size_bind_group_layout,
                            ], // No bind group layouts
                            push_constant_ranges: &[],
                        });

                // Load the shaders
                let shader_module_vert_primary =
                    gpu_resources
                        .device
                        .create_shader_module(wgpu::ShaderModuleDescriptor {
                            label: Some("Primary Vert Shader"),
                            source: wgpu::ShaderSource::Wgsl(
                                include_str!("shaders/vert_primary.wgsl").into(),
                            ),
                        });

                let shader_module_frag_primary =
                    gpu_resources
                        .device
                        .create_shader_module(wgpu::ShaderModuleDescriptor {
                            label: Some("Primary Frag Shader"),
                            source: wgpu::ShaderSource::Wgsl(
                                include_str!("shaders/frag_primary.wgsl").into(),
                            ),
                        });

                // let swapchain_capabilities = gpu_resources
                //     .surface
                //     .get_capabilities(&gpu_resources.adapter);
                // let swapchain_format = swapchain_capabilities.formats[0]; // Choosing the first available format
                let swapchain_format = wgpu::TextureFormat::Bgra8UnormSrgb; // hardcode for now

                // Configure the render pipeline
                let render_pipeline =
                    gpu_resources
                        .device
                        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                            label: Some("Common Vector Primary Render Pipeline"),
                            layout: Some(&pipeline_layout),
                            multiview: None,
                            cache: None,
                            vertex: wgpu::VertexState {
                                module: &shader_module_vert_primary,
                                entry_point: "vs_main", // name of the entry point in your vertex shader
                                buffers: &[Vertex::desc()], // Make sure your Vertex::desc() matches your vertex structure
                                compilation_options: wgpu::PipelineCompilationOptions::default(),
                            },
                            fragment: Some(wgpu::FragmentState {
                                module: &shader_module_frag_primary,
                                entry_point: "fs_main", // name of the entry point in your fragment shader
                                targets: &[Some(wgpu::ColorTargetState {
                                    format: swapchain_format,
                                    // blend: Some(wgpu::BlendState::REPLACE),
                                    blend: Some(wgpu::BlendState {
                                        color: wgpu::BlendComponent {
                                            src_factor: wgpu::BlendFactor::SrcAlpha,
                                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                            operation: wgpu::BlendOperation::Add,
                                        },
                                        alpha: wgpu::BlendComponent {
                                            src_factor: wgpu::BlendFactor::One,
                                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                            operation: wgpu::BlendOperation::Add,
                                        },
                                    }),
                                    write_mask: wgpu::ColorWrites::ALL,
                                })],
                                compilation_options: wgpu::PipelineCompilationOptions::default(),
                            }),
                            // primitive: wgpu::PrimitiveState::default(),
                            // depth_stencil: None,
                            // multisample: wgpu::MultisampleState::default(),
                            primitive: wgpu::PrimitiveState {
                                conservative: false,
                                topology: wgpu::PrimitiveTopology::TriangleList, // how vertices are assembled into geometric primitives
                                // strip_index_format: Some(wgpu::IndexFormat::Uint32),
                                strip_index_format: None,
                                front_face: wgpu::FrontFace::Ccw, // Counter-clockwise is considered the front face
                                // none cull_mode
                                cull_mode: None,
                                polygon_mode: wgpu::PolygonMode::Fill,
                                // Other properties such as conservative rasterization can be set here
                                unclipped_depth: false,
                            },
                            depth_stencil: Some(depth_stencil_state), // Optional, only if you are using depth testing
                            multisample: wgpu::MultisampleState {
                                count: 4, // effect performance
                                mask: !0,
                                alpha_to_coverage_enabled: false,
                            },
                        });

                // window_handle.render_pipeline = Some(render_pipeline);
                // window_handle.depth_view = gpu_helper.depth_view;

                println!("Initialized...");

                // TODO: actually do this when opening a sequence
                // saved_state.sequences.iter().for_each(|s| {
                //     s.active_polygons.iter().for_each(|p| {
                //         let restored_polygon = Polygon::new(
                //             &window_size,
                //             &gpu_resources.device,
                //             &camera,
                //             vec![
                //                 Point { x: 0.0, y: 0.0 },
                //                 Point { x: 1.0, y: 0.0 },
                //                 Point { x: 1.0, y: 1.0 },
                //                 Point { x: 0.0, y: 1.0 },
                //             ],
                //             (p.dimensions.0 as f32, p.dimensions.1 as f32),
                //             Point { x: 600.0, y: 100.0 },
                //             0.0,
                //             [1.0, 1.0, 1.0, 1.0],
                //             p.name.clone(),
                //             Uuid::from_str(&p.id).expect("Couldn't convert string to uuid"),
                //         );

                //         editor.add_polygon(restored_polygon);
                //     });
                // });

                // println!("Polygons restored...");

                window_handle.handle_cursor_moved = handle_cursor_moved(
                    cloned2.clone(),
                    gpu_resources.clone(),
                    cloned_viewport.clone(),
                );
                window_handle.handle_mouse_input = handle_mouse_input(
                    state_4.clone(),
                    cloned3.clone(),
                    gpu_resources.clone(),
                    cloned_viewport2.clone(),
                    record_2.clone(),
                );
                window_handle.handle_window_resized = handle_window_resize(
                    cloned7,
                    gpu_resources.clone(),
                    gpu_cloned3,
                    cloned_viewport3.clone(),
                );
                window_handle.handle_mouse_wheel =
                    handle_mouse_wheel(cloned11, gpu_resources.clone(), cloned_viewport3.clone());
                window_handle.handle_modifiers_changed = handle_modifiers_changed(
                    state_3,
                    gpu_resources.clone(),
                    cloned_viewport3.clone(),
                );
                window_handle.handle_keyboard_input =
                    handle_keyboard_input(state_2, gpu_resources.clone(), cloned_viewport3.clone());

                editor.update_camera_binding(&gpu_resources.queue);

                gpu_clonsed2.lock().unwrap().gpu_resources = Some(Arc::clone(&gpu_resources));
                editor.gpu_resources = Some(Arc::clone(&gpu_resources));
                editor.model_bind_group_layout = Some(model_bind_group_layout);
                editor.window_size_bind_group = Some(window_size_bind_group);
                window_handle.gpu_resources = Some(gpu_resources);
                // window_handle.gpu_helper = Some(gpu_clonsed2);
                editor.window = window_handle.window.clone();
                window_handle.engine_handle = Some(EngineHandle {
                    render_pipeline: Some(render_pipeline),
                    user_editor: Some(Box::new(cloned)),
                    gpu_helper: Some(gpu_cloned),
                    depth_view: None,
                });
            }
            .await;
        }
        //     PaintState::Initialized { .. } => {
        //         println!("Renderer is already initialized");
        //     }
        // }
    }

    app.run();
}
