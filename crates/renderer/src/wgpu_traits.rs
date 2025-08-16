use wgpu::{BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry};

pub trait AsBindGroup {
    // Associated Function
    fn bind_group_layout_entries() -> Vec<BindGroupLayoutEntry>;
    fn create_bind_group_layout(device: &wgpu::Device, label: &str) -> BindGroupLayout {
        let entries = Self::bind_group_layout_entries();

        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some(label),
            entries: &entries,
        })
    }

    // Methods
    fn init_bind_group(&mut self, device: &wgpu::Device);
    fn init_binding_resources(&mut self, device: &wgpu::Device);
    fn init_all(&mut self, device: &wgpu::Device) {
        self.init_binding_resources(device);
        self.init_bind_group(device);
    }
    fn bind_group_layout(&self) -> &wgpu::BindGroupLayout;
    fn bind_group(&self) -> &wgpu::BindGroup;
    fn update_binding_resources(&mut self);
    fn queue_write_binding_resources(&mut self, queue: &wgpu::Queue);
}
