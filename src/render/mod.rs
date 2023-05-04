use winit::window::Window;

mod texture;
mod buffer;

use texture::Texture;

use crate::systems::GameState;

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    render_pipeline: wgpu::RenderPipeline,
    diffuse_bind_group: wgpu::BindGroup,
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    index_count: usize
}

impl State {
    async fn new(window: Window) -> Self {
        let size = window.inner_size();
        
        // 'instance' created to handle GPU
        // 'Backends::all()' = Vulkan, Metal, DX12, and WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default()
        });

        // The 'surface' is the part of the window we draw to
        // 'surface' takes a reference to 'window', so 'window' needs to remain alive as long as 'surface'
        // Since 'State' owns 'window' and 'surface', this should be safe
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        // The 'adapter' is a handle to the GPU, can get information on name and backend
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false
            }
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None
            }, 
            None
        ).await.unwrap();

        let surface_capabilities = surface.get_capabilities(&adapter);

        // Surface texture format is SRGB
        let surface_format = surface_capabilities.formats.iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            // 'TextureUsages::RENDER_ATTACHMENT' specifies that the textures will be used to write to the screen
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            // 'present_mode' determines how to sync surface with display
            // Selected is 'PresentMode::Fifo', which is essentially VSync
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![]
        };

        surface.configure(&device, &config);

        let diffuse_bytes = include_bytes!("../img/sprite_sheet.png");
        let diffuse_texture = Texture::from_bytes(&device, &queue, diffuse_bytes, "tile.png").unwrap();

        let texture_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true }
                        },
                        count: None
                    },
                    wgpu:: BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None
                    }
                ],
                label: Some("texture_bind_group_layout")
            }
        );

        let diffuse_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture.view)
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler)
                    }
                ],
                label: Some("diffuse_bind_group")
            }
        );

        let shader = device.create_shader_module(wgpu::include_wgsl!("../shader.wgsl"));
        
        let render_pipeline_layout = device.create_pipeline_layout(
                &wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&texture_bind_group_layout],
                    push_constant_ranges: &[]
                }
            );

            let render_pipeline = device.create_render_pipeline(
                &wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[buffer::Vertex::desc()]
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL
                    })]
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false
                },
                multiview: None
            }
        );

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            diffuse_bind_group,
            vertex_buffer: None,
            index_buffer: None,
            index_count: 0
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.size
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config)
        }
    }

    pub fn update(&mut self, state: &GameState) {
        (self.vertex_buffer, self.index_buffer, self.index_count) = buffer::create_buffers(&self.device, &state);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // 'encoder' creates a command buffer to be sent to the GPU
        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder")
            }
        );

        {
            // 'begin_render_pass' borrows encoder mutably, we need to call 'encoder.finish()'
            // after this, thus the extra scope to drop the borrowed reference
            let mut render_pass = encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor { 
                    label: Some("Render Pass"), 
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment { 
                        view: &view, 
                        resolve_target: None, 
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0
                            }),
                            store: true
                        }
                    })],
                    depth_stencil_attachment: None
                }
            );

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.as_mut().unwrap().slice(..));
            render_pass.set_index_buffer(self.index_buffer.as_mut().unwrap().slice(..), wgpu::IndexFormat::Uint16);

            render_pass.draw_indexed(0..self.index_count as u32, 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

pub async fn new(window: Window) -> State {
    State::new(window).await
}