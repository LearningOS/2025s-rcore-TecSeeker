use alloc::vec;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

/// Resource type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    /// Mutex
    Mutex,
    /// Semaphore with a specific count
    Semaphore(usize),
}

/// Resource record for each thread
pub struct ResourceManagerRecord {
    /// allocation[rid]: Number of resources rid currently allocated to the thread
    pub allocation: Vec<usize>,
    /// need[rid]: Number of resources rid the thread still needs
    pub need: Vec<usize>,
}

/// Resource Manager
pub struct ResourceManager {
    /// Current number of resource types
    pub resource_num: usize,
    /// Current maximum thread ID
    pub max_tid: usize,
    /// Available quantity of each resource
    pub available: Vec<usize>,
    /// Mapping between mutex_id and resource_id
    pub mutex_id_mapping: BTreeMap<usize, usize>,
    /// Mapping between sem_id and resource_id
    pub sem_id_mapping: BTreeMap<usize, usize>,
    /// Type of each resource
    pub resource_types: Vec<ResourceType>,
    /// Resource records for each thread
    pub records: Vec<ResourceManagerRecord>,
}

impl ResourceManager {
    /// inilize a ResourceManager
    pub fn new() -> Self {
        Self {
            resource_num: 0,
            max_tid: usize::MAX, // Initial value as invalid
            available: Vec::new(),
            mutex_id_mapping: BTreeMap::new(),
            sem_id_mapping: BTreeMap::new(),
            resource_types: Vec::new(),
            records: Vec::new(),
        }
    }

    /// Register a new resource
    pub fn register_resource(&mut self, r_type: ResourceType, id: usize) {
        let available_count = match r_type {
            ResourceType::Mutex => 1,
            ResourceType::Semaphore(count) => count,
        };

        self.available.push(available_count);
        self.resource_types.push(r_type);

        // Extend allocation and need vectors for each thread
        for record in &mut self.records {
            record.allocation.push(0);
            record.need.push(0);
        }

        // Map id to resource_id (vector index)
        match r_type {
            ResourceType::Mutex => {
                self.mutex_id_mapping.insert(id, self.resource_num);
            }
            ResourceType::Semaphore(_) => {
                self.sem_id_mapping.insert(id, self.resource_num);
            }
        }
        self.resource_num += 1;
    }

    /// Add a new thread
    pub fn add_thread(&mut self) -> usize {
        let new_tid = if self.max_tid == usize::MAX {
            0
        } else {
            self.max_tid + 1
        };
        self.max_tid = new_tid;

        let record = ResourceManagerRecord {
            allocation: vec![0; self.resource_num],
            need: vec![0; self.resource_num],
        };
        self.records.push(record);

        new_tid
    }

    /// Request resources for a thread
    pub fn request_resource(
        &mut self,
        tid: usize,
        r_type: ResourceType,
        id: usize,
        count: usize,
    ) -> bool {
        if let Some(rid) = self.get_rid(r_type, id) {
            if tid > self.max_tid || rid >= self.resource_num {
                return false; // Illegal request
            }
            if self.available[rid] >= count {
                self.available[rid] -= count;
                self.records[tid].allocation[rid] += count;
                self.records[tid].need[rid] = self.records[tid].need[rid].saturating_sub(count);
                return true;
            } else {
                return false;
            }
        }
        false
    }

    /// Release resources for a thread
    pub fn release_resource(
        &mut self,
        tid: usize,
        r_type: ResourceType,
        id: usize,
        count: usize,
    ) -> bool {
        if let Some(rid) = self.get_rid(r_type, id) {
            if tid > self.max_tid || rid >= self.resource_num {
                return false;
            }
            if self.records[tid].allocation[rid] >= count {
                self.records[tid].allocation[rid] -= count;
                self.available[rid] += count;
                self.records[tid].need[rid] += count;
                return true;
            } else {
                return false;
            }
        }
        false
    }

    /// Request resource safely: allocate resource and check if system remains safe.
    /// If not safe, the allocation will be rolled back automatically.
    pub fn safe_request_resource(&mut self, tid: usize, r_type: ResourceType, id: usize, count: usize) -> bool {
        if !self.request_resource(tid, r_type, id, count) {
            return false;
        }
        if self.is_safe() {
            true
        } else {
            self.release_resource(tid, r_type, id, count);
            false
        }
    }

    /// Helper method: return the real rid
    fn get_rid(&self, r_type: ResourceType, id: usize) -> Option<usize> {
        match r_type {
            ResourceType::Mutex => self.mutex_id_mapping.get(&id).copied(),
            ResourceType::Semaphore(_) => self.sem_id_mapping.get(&id).copied(),
        }
    }

    /// Banker's algorithm: check if the current state is safe
    fn is_safe(&self) -> bool {
        let mut work = self.available.clone();
        let mut finish = vec![false; self.records.len()];

        loop {
            let mut progress = false;
            for (i, record) in self.records.iter().enumerate() {
                if !finish[i] && record.need.iter().zip(&work).all(|(n, w)| *n <= *w) {
                    // This thread can complete
                    for (w, alloc) in work.iter_mut().zip(&record.allocation) {
                        *w += *alloc;
                    }
                    finish[i] = true;
                    progress = true;
                }
            }
            if !progress {
                break;
            }
        }

        finish.iter().all(|&f| f)
    }
}
