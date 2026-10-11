use std::{collections::HashMap, ops::Add, sync::{LazyLock, RwLock}, time::Duration};

pub use track_macros::track;

///Contains timing info and statistics for a tracked function. Does not keep track of individual function calls.
#[derive(Clone, Copy, Debug)]
pub struct TrackInfo {
    ///Name of function given from either attribute in macro or signature function name. Remains constant even after reset.
    pub function_name: &'static str,
    ///All total calls made to a function since last reset. Default value is zero.
    pub calls: usize,
    ///Total duration of all calls to a function since last reset. Default value is [`Duration::ZERO`]
    pub total_time: Duration,
    ///Longest recorded time of a function since last reset. Default value is [`Duration::ZERO`]
    pub max_time: Duration,
    ///Shortest recorded time of a function since last reset. Default value is [`Duration::MAX`]
    pub min_time: Duration,
}

impl TrackInfo {
    pub fn new(function_name: &'static str) -> TrackInfo{
        TrackInfo {function_name, calls: 0, total_time: Duration::ZERO, max_time: Duration::ZERO, min_time: Duration::MAX }
    }

    ///Total Time / Number of Calls. Returns [`Duration::ZERO`] if calls are 0.
    pub fn get_mean_time(&self) -> Duration {
        if self.calls == 0 {return Duration::ZERO}
        self.total_time.div_f32(self.calls as f32)
    }

    pub fn reset(&mut self) {
        self.calls = 0;
        self.total_time = Duration::ZERO;
        self.max_time = Duration::ZERO;
        self.min_time = Duration::MAX;
    }

    pub fn push(&mut self, d: Duration) {
        self.calls += 1;
        self.total_time = self.total_time.add(d);
        self.max_time = self.max_time.max(d);
        self.min_time = self.min_time.min(d);
    }
}

impl Default for TrackInfo {
    fn default() -> Self {
        TrackInfo { function_name: "", calls: 0, total_time: Duration::ZERO, max_time: Duration::ZERO, min_time: Duration::MAX }
    }
}

static TRACKERS: LazyLock<RwLock<HashMap<&'static str, TrackInfo>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

///Inserts or updates given function name and duration into Trackers. Done automatically with #\[track\] attribute macro
pub fn record(name: &'static str, duration: Duration) {
    let new = &mut TrackInfo::new(name);
    let trackers = TRACKERS.read().unwrap();
    let mut record =  *trackers.get(name).unwrap_or(new);
    record.push(duration);
    drop(trackers);
    TRACKERS.write().unwrap().insert(name, record);
}

///Returns the ['TrackInfo] of a tracked function, or None if no such function name exists.
pub fn get(name: &'static str) -> Option<TrackInfo> {
    let t = TRACKERS.read().unwrap();
    t.get(name).copied()
}

///Gets cloned values of all TrackInfo's in HashMap
pub fn get_all() -> Vec<TrackInfo> {
    TRACKERS.read().unwrap().values().cloned().collect()
}

pub fn contains(name: &'static str) -> bool {
    TRACKERS.read().unwrap().contains_key(name)
}

///Returns the [`TrackInfo`] of a tracker, or [`TrackInfo::Default`] if no such function name exists.
pub fn get_or_default(name: &'static str) -> TrackInfo {
    let t = TRACKERS.read().unwrap();
    t.get(name).copied().unwrap_or_default()
}

///Resets the TrackInfo of given name to default values.
pub fn reset(name: &'static str) {
    TRACKERS.write().unwrap().get_mut(name).and_then(|t| Some(t.reset())).unwrap();
}

///Resets all trackers to default values. Function names remain in HashMap as keys.
pub fn reset_all() {
    TRACKERS.write().unwrap().values_mut().for_each(|t| t.reset());
}

///Erases all mapped trackers.
/// 
/// NOTE: Key-values will be re-added to the HashMap next time a tracked function is called. 
pub fn clear_all() {
    TRACKERS.write().unwrap().clear();
}