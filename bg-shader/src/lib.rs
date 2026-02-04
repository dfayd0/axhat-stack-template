use wasm_bindgen::prelude::*;
use wgpu::util::DeviceExt;

#[wasm_bindgen]
pub async fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info).ok();

    log::info!("bg-shader: starting");

    let window = web_sys::window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;
    let canvas = document
        .get_element_by_id("bg-canvas")
        .ok_or("no bg-canvas element")?;
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into()
        .map_err(|_| "element is not a canvas")?;

    let width = window.inner_width()?.as_f64().unwrap_or(800.0) as u32;
    let height = window.inner_height()?.as_f64().unwrap_or(600.0) as u32;
    canvas.set_width(width);
    canvas.set_height(height);

    log::info!("bg-shader: canvas {}x{}", width, height);

    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::BROWSER_WEBGPU | wgpu::Backends::GL,
        ..Default::default()
    });

    let surface = instance
        .create_surface(wgpu::SurfaceTarget::Canvas(canvas.clone()))
        .map_err(|e| JsValue::from_str(&format!("surface error: {e}")))?;

    log::info!("bg-shader: requesting adapter");

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .ok_or("no adapter found — WebGPU/WebGL2 not available")?;

    log::info!("bg-shader: adapter found: {:?}", adapter.get_info().backend);

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("bg-device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
            memory_hints: wgpu::MemoryHints::MemoryUsage,
        }, None)
        .await
        .map_err(|e| JsValue::from_str(&format!("device error: {e}")))?;

    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps
        .formats
        .iter()
        .find(|f| f.is_srgb())
        .copied()
        .unwrap_or(surface_caps.formats[0]);

    log::info!("bg-shader: format {:?}", surface_format);

    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width,
        height,
        present_mode: wgpu::PresentMode::AutoVsync,
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(&device, &config);

    // Shader
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("bg-shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });

    // Uniforms
    #[repr(C)]
    #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
    struct Uniforms {
        resolution: [f32; 2],
        time: f32,
        _padding: f32,
    }

    let mut uniforms = Uniforms {
        resolution: [width as f32, height as f32],
        time: 0.0,
        _padding: 0.0,
    };

    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("uniform-buffer"),
        contents: bytemuck::cast_slice(&[uniforms]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("bg-bind-group-layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("bg-bind-group"),
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("bg-pipeline-layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("bg-pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    log::info!("bg-shader: pipeline created, starting render loop");

    // Animation loop
    let render_loop = Closure::<dyn FnMut()>::new(move || {
        let window = web_sys::window().unwrap();
        let now = js_sys::Date::now() / 1000.0;
        uniforms.time = now as f32;

        // Handle resize
        let w = window.inner_width().unwrap().as_f64().unwrap_or(800.0) as u32;
        let h = window.inner_height().unwrap().as_f64().unwrap_or(600.0) as u32;
        if w != config.width || h != config.height {
            config.width = w.max(1);
            config.height = h.max(1);
            canvas.set_width(config.width);
            canvas.set_height(config.height);
            surface.configure(&device, &config);
            uniforms.resolution = [config.width as f32, config.height as f32];
        }

        queue.write_buffer(&uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        let output = match surface.get_current_texture() {
            Ok(t) => t,
            Err(_) => return,
        };
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("bg-encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("bg-render-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&render_pipeline);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        queue.submit(std::iter::once(encoder.finish()));
        output.present();
    });

    // Start the loop using setInterval at ~30fps for low power usage
    window
        .set_interval_with_callback_and_timeout_and_arguments_0(
            render_loop.as_ref().unchecked_ref(),
            33, // ~30fps
        )
        .map_err(|_| "failed to set interval")?;

    render_loop.forget(); // Don't drop the closure

    log::info!("bg-shader: running");
    Ok(())
}
