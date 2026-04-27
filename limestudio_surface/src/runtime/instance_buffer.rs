use wgpu::*;

pub struct InstanceBuffer<T: bytemuck::Pod> {
    buffer: Buffer,
    capacity: usize,
    _marker: std::marker::PhantomData<T>,
}

impl<T: bytemuck::Pod> InstanceBuffer<T> {
    pub fn new(device: &Device, label: &str, capacity: usize) -> Self {
        let buffer = device.create_buffer(&BufferDescriptor {
            label: Some(label),
            size: (std::mem::size_of::<T>() * capacity) as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            capacity,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn write(&self, queue: &Queue, instances: &[T]) {
        let size = std::mem::size_of_val(instances).min(self.capacity * std::mem::size_of::<T>());
        queue.write_buffer(&self.buffer, 0, &bytemuck::cast_slice(instances)[..size]);
    }

    pub fn slice(&self) -> BufferSlice<'_> {
        self.buffer.slice(..)
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}
