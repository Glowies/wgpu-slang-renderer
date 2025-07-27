pub trait AsBindGroup {
    fn init_bind_group_layout(&mut self, device: &wgpu::Device);
    fn init_bind_group(&mut self, device: &wgpu::Device);
    fn init_uniform_buffer(&mut self, device: &wgpu::Device);
    fn init_uniform_bind_group(&mut self, device: &wgpu::Device) {
        self.init_uniform_buffer(device);
        self.init_bind_group_layout(device);
        self.init_bind_group(device);
    }
    fn bind_group_layout(&self) -> &wgpu::BindGroupLayout;
    fn bind_group(&self) -> &wgpu::BindGroup;
    fn uniform_buffer(&self) -> &wgpu::Buffer;
    fn update_uniform(&mut self);
    fn queue_write_buffer(&mut self, queue: &wgpu::Queue);
}
