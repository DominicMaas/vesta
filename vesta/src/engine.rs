use winit::{event::{Event, WindowEvent}, event_loop::{ControlFlow, EventLoop}, window::{Window, WindowBuilder}};
use crate::{VestaApp, config::Config, renderer::Renderer, texture};

pub struct Engine {
    pub renderer: Renderer,
}

impl Engine {
    pub async fn run<V: VestaApp + 'static>(config: Config)  {        
        // Loop that will run all the events
        let event_loop = EventLoop::new();
         
        // Build the window with specified config
        let window = WindowBuilder::new()
            .with_title(config.window_title)
            .build(&event_loop)
            .unwrap();
        
        // Determined window size
        let size = window.inner_size();
        
        // New WGPU instance and surface to render on
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(&window) };
        
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();
            
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None, // Trace path
            )
            .await
            .unwrap();
            
        let swap_chain_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter.get_swap_chain_preferred_format(&surface),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
    
        let swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);
        
        // Create a depth texture
        let depth_texture =
            texture::Texture::create_depth(&device, &swap_chain_desc, Some("Depth Texture")).unwrap();
        
        // Renderer information, this will be sent to the app implementation so it can access resources
        let renderer = Renderer { surface, device, queue, swap_chain_desc, swap_chain, size, depth_texture };
        
        let mut engine = Engine { renderer };
        
        // First initllize all the apps resources (shaders, pipelines etc.)
        let mut app = V::init(&engine);
                
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            
            match event {
                Event::RedrawRequested(_) => {
                    match Self::render(&window, &engine, &mut app) {
                        Ok(_) => {}
                        // Recreate the swap_chain if lost
                        Err(wgpu::SwapChainError::Lost) => {
                            let size = engine.renderer.size;
                            Self::resize(&mut engine,size)
                        },
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        // All other errors (Outdated, Timeout) should be resolved by the next frame
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
                Event::MainEventsCleared => {
                    window.request_redraw();
                }
                Event::WindowEvent { ref event, .. } => {
                    //if !state.input(event) {
                        match event {
                            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                            WindowEvent::Resized(physical_size) => {
                                Self::resize(&mut engine, *physical_size);
                            }
                            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                                Self::resize(&mut engine, **new_inner_size);
                            }
                            _ => {}
                        }
                    //}
                }
                _ => {}
            }
        });
    }
    
    fn resize(engine: &mut Engine, new_size: winit::dpi::PhysicalSize<u32>) {
        engine.renderer.size = new_size;
        
        engine.renderer.swap_chain_desc.width = new_size.width;
        engine.renderer.swap_chain_desc.height = new_size.height;
        
        engine.renderer.swap_chain = engine.renderer.device.create_swap_chain(&engine.renderer.surface, &engine.renderer.swap_chain_desc);
        
        engine.renderer.depth_texture = engine.renderer.create_depth_texture(Some("Depth Texture")).unwrap();        
    }
    
    fn render<V: VestaApp>(_window: &Window, engine: &Engine, app: &mut V) -> Result<(), wgpu::SwapChainError> {
        // Get a frame
        let frame = engine.renderer.swap_chain.get_current_frame()?.output;
        let mut encoder = engine.renderer.device.create_command_encoder(&Default::default());
        
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &engine.renderer.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
            
            app.render(engine, &mut render_pass)
        }
        
        // Finished with the frame
        engine.renderer.queue.submit(std::iter::once(encoder.finish()));  
        Ok(())
    }
}
