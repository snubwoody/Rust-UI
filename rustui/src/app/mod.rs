pub mod view;
pub mod events;
use std::fs;
use events::EventManager;
use glium::{
	glutin::surface::WindowSurface, Display, Program,
};
use winit::{
	event::WindowEvent, 
	event_loop::{
		ControlFlow, 
		EventLoop
	}, 
	window::{Window, WindowBuilder}
};
use crate::app::view:: View;



/// This is a singular isolated program. Most projects
/// will only contain one app
pub struct App<'a>{
	event_loop:EventLoop<()>,
	window:&'a Window,
	event_manager:EventManager,
	views:Vec<View>,
	context:RenderContext,
	index:usize,
	surface: wgpu::Surface<'a>,
	device: wgpu::Device,
	queue: wgpu::Queue,
	config: wgpu::SurfaceConfiguration,
}

impl<'a> App<'a>{
	pub async fn new() -> Self {
		let event_loop = EventLoop::new().unwrap();

		// Set the control flow to redraw every frame whether
		// there are events to process or not
		event_loop.set_control_flow(ControlFlow::Poll);
		
		let window = WindowBuilder::new().build(&event_loop).unwrap();
		
		let event_manager = EventManager::new();

		// Handle to wpgu for creating a surface and an adapter
		let instance = wgpu::Instance::new(wgpu::InstanceDescriptor{
			backends: wgpu::Backends::PRIMARY,
			..Default::default()
		});

		// Create the surface to draw on
		let surface = instance.create_surface(&window).unwrap();

		// Handle to the graphics card
		let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions{
			power_preference: Default::default(),
			compatible_surface: Some(&surface),
			force_fallback_adapter:false
		}).await.unwrap();

		// The device is an open connection to the graphics
		// card and the queue is a command buffer
		let (device,queue) = adapter.request_device(&wgpu::DeviceDescriptor{
			label: Some("Device/Queue"),
			required_features: wgpu::Features::empty(),
			..Default::default()
		}, None).await.unwrap();

		let surface_caps = surface.get_capabilities(&adapter);

		// Get an sRGB texture format
		let surface_format = 
			surface_caps
			.formats
			.iter()
			.find(|f|f.is_srgb())
			.copied()
			.unwrap_or(surface_caps.formats[0]);

		// The surface configuration
		let config = wgpu::SurfaceConfiguration{
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: surface_format,
			width: window.inner_size().width,
			height: window.inner_size().height,
			present_mode: surface_caps.present_modes[0],
			alpha_mode:surface_caps.alpha_modes[0],
			view_formats: vec![],
			desired_maximum_frame_latency: 2
		};

		let context = helium_renderer::RenderContext::new(&device, &config);

		Self { 
			event_loop,
			window:window,
			event_manager,
			context,
			views:vec![],
			index:0,
			device,
			queue,
			surface,
			config
		}
	}

	pub fn add_view(mut self,view:View) -> Self{
		self.views.push(view);
		self
	}

	pub fn run(mut self){
		self.event_loop.run(move | event,window_target|{
			match event {
				winit::event::Event::WindowEvent{event,..} => match event {
					WindowEvent::CloseRequested => window_target.exit(),
					WindowEvent::RedrawRequested => {
						// Re-render the page when the window redraws
						self.views[self.index].render(&self.display, &self.window,&self.context)
					},
					event => {
						// Send all other window events to the event manager
						let widget_tree = &mut self.views[0].widget_tree;
						// FIXME error here
						//self.event_manager.handle_events(widget_tree,event)
					}
				}, 
				_ => {}
			}
	
		}).expect("Event loop error occured");
	}


}

fn create_program(display:&Display<WindowSurface>,vertex_shader_src:&str,fragment_shader_src:&str) -> Program {
	let vertex_shader = fs::read_to_string(vertex_shader_src).expect("Cannot locate vertex shader file");
	let fragment_shader = fs::read_to_string(fragment_shader_src).expect("Cannot locate vertex shader file");
	let program = glium::Program::from_source(display, vertex_shader.as_str(), fragment_shader.as_str(), None).unwrap();
	return program
}

/// Holds the compiled shaders
#[derive(Debug)]
pub struct RenderContext{
	pub rect_pipeline: wgpu::RenderPipeline,
	pub text_pipeline: wgpu::RenderPipeline,
	pub image_pipeline: wgpu::RenderPipeline
}

impl RenderContext {
	pub fn new(device:&wgpu::Device,config:&wgpu::SurfaceConfiguration) -> Self{
		Self{
			rect_pipeline: RenderContext::create_rect_pipeline(device, config),
			text_pipeline: RenderContext::create_text_pipeline(device, config),
			image_pipeline: RenderContext::create_image_pipeline(device, config)
		}
	}

	fn create_rect_pipeline(device:&wgpu::Device,config:&wgpu::SurfaceConfiguration) -> wgpu::RenderPipeline {
		// Compiled shader
		let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { 
			label: Some("Shader module"), 
			source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/surface/rect.wgsl").into())
		});

		let render_pipeline_layout = 
			device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { 
				label: Some("Render pipeline layout"), 
				bind_group_layouts: &[], 
				push_constant_ranges: &[] 
			});

		let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor { 
			label: Some("Render pipeline"), 
			layout: Some(&render_pipeline_layout), 
			vertex: wgpu::VertexState { 
				module: &shader, 
				entry_point: "vs_main", 
				compilation_options: Default::default(), 
				buffers: &[Vertex::decription()] 
			}, 
			fragment: Some(wgpu::FragmentState{
				module:&shader,
				entry_point:"fs_main",
				compilation_options: Default::default(),
				targets:&[Some(wgpu::ColorTargetState { 
					format: config.format, 
					blend: Some(wgpu::BlendState::ALPHA_BLENDING), // TODO check pre-multiplied alpha blending 
					write_mask: wgpu::ColorWrites::ALL 
				})]
			}), 
			primitive: wgpu::PrimitiveState { 
				topology: wgpu::PrimitiveTopology::TriangleList, 
				strip_index_format: None, 
				front_face: wgpu::FrontFace::Ccw, 
				cull_mode: Some(wgpu::Face::Back), 
				unclipped_depth: false, 
				polygon_mode: wgpu::PolygonMode::Fill, 
				conservative: false 
			}, 
			multisample: wgpu::MultisampleState { 
				count: 1, 
				mask: !0, 
				alpha_to_coverage_enabled: false,
			}, 
			multiview: None, 
			cache: None, 
			depth_stencil: None, 
		});

		render_pipeline
	}

	fn create_text_pipeline(device:&wgpu::Device,config:&wgpu::SurfaceConfiguration) -> wgpu::RenderPipeline {
		// TODO replace this with the actual text shader
		// Compiled shader
		let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { 
			label: Some("Shader module"), 
			source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/rect.wgsl").into())
		});

		let render_pipeline_layout = 
			device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { 
				label: Some("Render pipeline layout"), 
				bind_group_layouts: &[], 
				push_constant_ranges: &[] 
			});

		let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor { 
			label: Some("Render pipeline"), 
			layout: Some(&render_pipeline_layout), 
			vertex: wgpu::VertexState { 
				module: &shader, 
				entry_point: "vs_main", 
				compilation_options: Default::default(), 
				buffers: &[Vertex::decription()] 
			}, 
			fragment: Some(wgpu::FragmentState{
				module:&shader,
				entry_point:"fs_main",
				compilation_options: Default::default(),
				targets:&[Some(wgpu::ColorTargetState { 
					format: config.format, 
					blend: Some(wgpu::BlendState::ALPHA_BLENDING), // TODO check pre-multiplied alpha blending 
					write_mask: wgpu::ColorWrites::ALL 
				})]
			}), 
			primitive: wgpu::PrimitiveState { 
				topology: wgpu::PrimitiveTopology::TriangleList, 
				strip_index_format: None, 
				front_face: wgpu::FrontFace::Ccw, 
				cull_mode: Some(wgpu::Face::Back), 
				unclipped_depth: false, 
				polygon_mode: wgpu::PolygonMode::Fill, 
				conservative: false 
			}, 
			multisample: wgpu::MultisampleState { 
				count: 1, 
				mask: !0, 
				alpha_to_coverage_enabled: false,
			}, 
			multiview: None, 
			cache: None, 
			depth_stencil: None, 
		});

		render_pipeline
	}

	fn create_image_pipeline(device:&wgpu::Device,config:&wgpu::SurfaceConfiguration) -> wgpu::RenderPipeline {
		// TODO replace this with the actual text shader
		// Compiled shader
		let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { 
			label: Some("Shader module"), 
			source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/rect.wgsl").into())
		});

		let render_pipeline_layout = 
			device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { 
				label: Some("Render pipeline layout"), 
				bind_group_layouts: &[], 
				push_constant_ranges: &[] 
			});

		let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor { 
			label: Some("Render pipeline"), 
			layout: Some(&render_pipeline_layout), 
			vertex: wgpu::VertexState { 
				module: &shader, 
				entry_point: "vs_main", 
				compilation_options: Default::default(), 
				buffers: &[Vertex::decription()] 
			}, 
			fragment: Some(wgpu::FragmentState{
				module:&shader,
				entry_point:"fs_main",
				compilation_options: Default::default(),
				targets:&[Some(wgpu::ColorTargetState { 
					format: config.format, 
					blend: Some(wgpu::BlendState::ALPHA_BLENDING), // TODO check pre-multiplied alpha blending 
					write_mask: wgpu::ColorWrites::ALL 
				})]
			}), 
			primitive: wgpu::PrimitiveState { 
				topology: wgpu::PrimitiveTopology::TriangleList, 
				strip_index_format: None, 
				front_face: wgpu::FrontFace::Ccw, 
				cull_mode: Some(wgpu::Face::Back), 
				unclipped_depth: false, 
				polygon_mode: wgpu::PolygonMode::Fill, 
				conservative: false 
			}, 
			multisample: wgpu::MultisampleState { 
				count: 1, 
				mask: !0, 
				alpha_to_coverage_enabled: false,
			}, 
			multiview: None, 
			cache: None, 
			depth_stencil: None, 
		});

		render_pipeline
	}
}

#[repr(C)]
#[derive(Debug,Clone,Copy,bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex{
	pub position:[f32;2],
	pub color: [f32;4]	
}

impl Vertex {
	pub fn decription() -> wgpu::VertexBufferLayout<'static> {
		wgpu::VertexBufferLayout { 
			array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress, 
			step_mode: wgpu::VertexStepMode::Vertex, 
			attributes: &[
				wgpu::VertexAttribute{
					offset: 0,
					shader_location: 0,
					format: wgpu::VertexFormat::Float32x3
				},
				wgpu::VertexAttribute{
					offset: size_of::<[f32;3]>() as wgpu::BufferAddress,
					shader_location: 1,
					format: wgpu::VertexFormat::Float32x3
				}
			]
		}
	}
}