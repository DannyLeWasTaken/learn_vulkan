use ash::vk;

// Effectively functions as a free list allocator
#[derive(Default, Debug)]
pub struct DescriptorTable<T> {
    free_ids: Vec<u32>,
    resources: Vec<Option<T>>,

    /// Indices that need to be updated/written to
    writes: Vec<u32>,
}

impl<T> DescriptorTable<T> {
    pub fn new() -> Self {
        Self {
            free_ids: Vec::with_capacity(u16::MAX as usize),
            resources: Vec::with_capacity(u16::MAX as usize),
            writes: Vec::with_capacity(u16::MAX as usize),
        }
    }

    pub fn get_writes(&self) -> &[u32] {
        self.writes.as_slice()
    }

    pub fn get_resources(&self) -> &[Option<T>] {
        self.resources.as_slice()
    }

    pub fn clear_writes(&mut self) {
        self.writes.clear();
    }

    pub fn get_resource(&self, index: usize) -> &Option<T> {
        self.resources.get(index).unwrap()
    }

    /// Grab the next free id available
    pub fn get_free_id(&mut self) -> u32 {
        if self.free_ids.is_empty() {
            self.resources.len() as u32
        } else {
            self.free_ids.pop().unwrap()
        }
    }

    pub fn allocate_resource(&mut self, resource: T) -> u32 {
        let mut id: u32 = 0;
        if self.free_ids.is_empty() {
            id = self.resources.len() as u32;
            self.resources.push(Some(resource));
            self.writes.push(id);
        } else {
            id = self.free_ids.pop().unwrap();
            self.resources.insert(id as usize, Some(resource));
            self.writes.push(id);
        }
        id
    }

    pub fn free_resource(&mut self, index: u32) {
        self.resources.insert(index as usize, None);
        self.free_ids.push(index);
    }
}
