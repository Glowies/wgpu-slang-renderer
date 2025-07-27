pub trait AsBindGroup {
    fn init_bind_group_layout(&mut self, device: &wgpu::Device);
    fn init_bind_group(&mut self, device: &wgpu::Device);
    fn init_binding_resources(&mut self, device: &wgpu::Device);
    fn init_uniform_bind_group(&mut self, device: &wgpu::Device) {
        self.init_binding_resources(device);
        self.init_bind_group_layout(device);
        self.init_bind_group(device);
    }
    fn bind_group_layout(&self) -> &wgpu::BindGroupLayout;
    fn bind_group(&self) -> &wgpu::BindGroup;
    fn update_binding_resources(&mut self);
    fn queue_write_binding_resources(&mut self, queue: &wgpu::Queue);
}
