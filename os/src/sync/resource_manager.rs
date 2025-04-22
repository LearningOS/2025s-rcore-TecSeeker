use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;

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
    /// Create a new empty ResourceManager.
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

    /// Check request is safe
    pub fn safe_request_resource(
        &mut self,
        tid: usize,
        r_type: ResourceType,
        id: usize,
        count: usize,
    ) -> bool {
        self.add_need(tid, id, r_type, count);
        if !self.is_safe() {
            self.minus_need(tid, id, r_type, count);
            return false;
        }
        self.alloc_resource(tid, id, r_type, count);
        true
    }
    /// Release resource
    pub fn release_resource(&mut self, tid: usize, id: usize, r_type: ResourceType, count: usize) {
        if let Some(rid) = self.get_rid(r_type, id) {
            if tid > self.max_tid || rid >= self.resource_num {
                return;
            }
            self.records[tid].allocation[rid] -= count;
            self.available[rid] += count;
        }
    }
    /// Alloc resource
    pub fn alloc_resource(&mut self, tid: usize, id: usize, r_type: ResourceType, count: usize) {
        if let Some(rid) = self.get_rid(r_type, id) {
            if tid > self.max_tid || rid >= self.resource_num || count > self.available[rid]{
                return;
            }
            self.records[tid].need[rid] -= count;
            self.records[tid].allocation[rid] += count;
            self.available[rid] -= count;
        }
    }
    fn add_need(&mut self, tid: usize, id: usize, r_type: ResourceType, count: usize) {
        if let Some(rid) = self.get_rid(r_type, id) {
            if tid > self.max_tid || rid >= self.resource_num {
                return;
            }
            self.records[tid].need[rid] += count;
        }
    }

    fn minus_need(&mut self, tid: usize, id: usize, r_type: ResourceType, count: usize) {
        if let Some(rid) = self.get_rid(r_type, id) {
            if tid > self.max_tid || rid >= self.resource_num {
                return;
            }
            self.records[tid].need[rid] -= count;
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
