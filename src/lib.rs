use std::collections::HashMap;
use std::ptr;

use kperf_sys::constants::KPC_CLASS_CONFIGURABLE_MASK;
use kperf_sys::functions::*;
use kperf_sys::structs::*;

mod error;
mod event;

pub use error::KPerfError;
pub use event::Event;

/// A wrapper around the macOS private kpc_* APIs.
pub struct KPerf {
    classes: u32,
    counter_map: [usize; Self::MAX_COUNTERS],
    regs: [u64; Self::MAX_COUNTERS],
    start_counters: [u64; Self::MAX_COUNTERS],
    stop_counters: [u64; Self::MAX_COUNTERS],
    kpep_db: *mut kpep_db,
    kpep_config: *mut kpep_config,
    events: Vec<Event>,
}

impl KPerf {
    /// The maximum number of counters KPerf can support.
    pub const MAX_COUNTERS: usize = kperf_sys::constants::KPC_MAX_COUNTERS as _;

    /// Create a new KPerf instance.
    pub fn new() -> Result<Self, KPerfError> {
        Self::check_permission()?;
        let mut kperf = KPerf {
            classes: 0,
            counter_map: [0; Self::MAX_COUNTERS],
            regs: [0; Self::MAX_COUNTERS],
            start_counters: [0; Self::MAX_COUNTERS],
            stop_counters: [0; Self::MAX_COUNTERS],
            kpep_db: ptr::null_mut(),
            kpep_config: ptr::null_mut(),
            events: Vec::new(),
        };
        unsafe { kperf.init()? };
        Ok(kperf)
    }

    fn check_permission() -> Result<(), KPerfError> {
        let mut val_out: i32 = 0;
        let res = unsafe { kpc_force_all_ctrs_get(&mut val_out) };
        if res != 0 {
            return Err(KPerfError::PermissionDenied);
        }
        Ok(())
    }

    unsafe fn init(&mut self) -> Result<(), KPerfError> {
        // Create kpep db
        let ret = kpep_db_create(ptr::null(), &mut self.kpep_db);
        if ret != 0 {
            return Err(KPerfError::InitError);
        }
        // Create kpep config
        let ret = kpep_config_create(self.kpep_db, &mut self.kpep_config);
        if ret != 0 {
            return Err(KPerfError::InitError);
        }
        kpep_config_force_counters(self.kpep_config);
        Ok(())
    }

    /// Add one perf event.
    pub fn add_event(&mut self, user_only: bool, e: Event) -> Result<(), KPerfError> {
        let event_name = e.get_internal_name();
        unsafe {
            let event_name_cstr: *const i8 = event_name.as_ptr() as *const i8;
            let mut event: *mut kpep_event = ptr::null_mut();
            kpep_db_event(self.kpep_db, event_name_cstr, &mut event);
            if event.is_null() {
                return Err(KPerfError::InvalidEvent);
            }
            let ret = kpep_config_add_event(
                self.kpep_config,
                &mut event,
                if !user_only { 0 } else { 1 },
                ptr::null_mut(),
            );
            if ret != 0 {
                return Err(KPerfError::InvalidEvent);
            }
            self.events.push(e);
        }
        Ok(())
    }

    /// Add multiple perf events.
    pub fn add_events(&mut self, user_only: bool, events: &[Event]) -> Result<(), KPerfError> {
        for e in events {
            self.add_event(user_only, *e)?;
        }
        Ok(())
    }

    fn release(&mut self) {
        unsafe {
            kpep_config_free(self.kpep_config);
            kpep_db_free(self.kpep_db);
        }
    }

    unsafe fn get_counter_values(buf: &mut [u64; Self::MAX_COUNTERS]) -> Result<(), KPerfError> {
        if kpc_get_thread_counters(0, Self::MAX_COUNTERS as _, buf.as_mut_ptr()) != 0 {
            return Err(KPerfError::FetchCountersFailed);
        }
        Ok(())
    }

    unsafe fn set_counting(classes: u32, e: KPerfError) -> Result<(), KPerfError> {
        if kpc_set_counting(classes) != 0 {
            return Err(e);
        }
        if kpc_set_thread_counting(classes) != 0 {
            return Err(e);
        }
        Ok(())
    }

    /// Start counting the events.
    pub fn start(&mut self) -> Result<(), KPerfError> {
        if self.events.is_empty() {
            return Ok(());
        }
        unsafe {
            if kpep_config_kpc_classes(self.kpep_config, &mut self.classes) != 0 {
                return Err(KPerfError::InitError);
            }
            let mut kpc_reg_count = 0;
            if kpep_config_kpc_count(self.kpep_config, &mut kpc_reg_count) != 0 {
                return Err(KPerfError::InitError);
            }
            if kpep_config_kpc_map(
                self.kpep_config,
                self.counter_map.as_mut_ptr(),
                self.counter_map.len() * std::mem::size_of::<usize>(),
            ) != 0
            {
                return Err(KPerfError::InitError);
            }
            if kpep_config_kpc(
                self.kpep_config,
                self.regs.as_mut_ptr(),
                self.regs.len() * std::mem::size_of::<u64>(),
            ) != 0
            {
                return Err(KPerfError::InitError);
            }
            if kpc_force_all_ctrs_set(1) != 0 {
                return Err(KPerfError::InitError);
            }
            if (self.classes & KPC_CLASS_CONFIGURABLE_MASK) != 0 && kpc_reg_count != 0 {
                let res = kpc_set_config(self.classes, self.regs.as_mut_ptr());
                if res != 0 {
                    return Err(KPerfError::InitError);
                }
            }
            // start counting
            Self::set_counting(self.classes, KPerfError::InitError)?;
            Self::get_counter_values(&mut self.start_counters)?;
        }
        Ok(())
    }

    /// Stop counting the events.
    pub fn stop(&mut self) -> Result<HashMap<Event, u64>, KPerfError> {
        if self.events.is_empty() {
            return Ok(HashMap::new());
        }
        unsafe {
            Self::get_counter_values(&mut self.stop_counters)?;
            Self::set_counting(0, KPerfError::DeinitError)?;
        }
        let mut values = HashMap::new();
        for (i, e) in self.events.iter().enumerate() {
            let idx = self.counter_map[i];
            let value = self.stop_counters[idx] - self.start_counters[idx];
            values.insert(*e, value);
        }
        Ok(values)
    }

    /// Get event counter values.
    pub fn get_results(&self) -> HashMap<Event, u64> {
        let mut values = HashMap::new();
        for (i, e) in self.events.iter().enumerate() {
            let idx = self.counter_map[i];
            let value = self.stop_counters[idx] - self.start_counters[idx];
            values.insert(*e, value);
        }
        values
    }
}

impl Drop for KPerf {
    fn drop(&mut self) {
        self.release();
    }
}
