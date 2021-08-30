use wgpu::util::BufferInitDescriptor;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ModelUniform {
    pub model: cgmath::Matrix4<f32>,  // 4x4 matrix
    pub normal: cgmath::Matrix3<f32>, // 3x3 matrix
}

unsafe impl bytemuck::Zeroable for ModelUniform {}
unsafe impl bytemuck::Pod for ModelUniform {}

/// A holder for a uniform buffer, contains the data and raw buffer
pub struct UniformBuffer<T>
where
    T: Copy + bytemuck::Pod + bytemuck::Zeroable,
{
    pub data: T,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl crate::Renderer {
    /// Write the specified uniform buffer to the GPU
    pub fn write_uniform_buffer<T: Copy + bytemuck::Pod + bytemuck::Zeroable>(
        &self,
        uniform_buffer: &UniformBuffer<T>,
    ) {
        self.queue.write_buffer(
            &uniform_buffer.buffer,
            0,
            bytemuck::cast_slice(&[uniform_buffer.data]),
        );
    }
}

impl<T: Copy + bytemuck::Pod + bytemuck::Zeroable> UniformBuffer<T> {
    #[deprecated(note = "Please use renderer.write_uniform_buffer() instead")]
    pub fn write_buffer(&self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.data]));
    }

    //noinspection RsBorrowChecker
    /// Crate a new uniform buffer to store data of type
    pub fn new(name: &str, visibility: wgpu::ShaderStages, data: T, device: &wgpu::Device) -> Self {
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some(name),
            contents: bytemuck::cast_slice(&[data]),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &UniformBufferUtils::create_bind_group_layout(visibility, device),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("uniform_bind_group"),
        });

        Self {
            data,
            buffer,
            bind_group,
        }
    }
}

pub struct UniformBufferUtils {}
impl UniformBufferUtils {
    pub fn create_bind_group_layout(
        visibility: wgpu::ShaderStages,
        device: &wgpu::Device,
    ) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("uniform_bind_group_layout"),
        })
    }
}
