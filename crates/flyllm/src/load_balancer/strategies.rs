use log::debug;
use rand::Rng;

use crate::load_balancer::tracker::InstanceTracker;

/// Trait defining the interface for load balancing strategies
///
/// Implementations of this trait determine how to select which LLM instance
/// will handle a particular request based on instance metrics.
pub trait LoadBalancingStrategy {
    /// Select an instance from available candidates
    ///
    /// # Parameters
    /// * `trackers` - Array of (id, tracker) tuples for available instances
    ///
    /// # Returns
    /// * Index into the trackers array of the selected instance
    fn select_instance(&mut self, trackers: &[(usize, &InstanceTracker)]) -> usize;
}

/// Strategy that selects the instance that was used least recently
///
/// This strategy helps distribute load by prioritizing instances
/// that haven't been used in the longest time.
pub struct LeastRecentlyUsedStrategy;

impl LeastRecentlyUsedStrategy {
    /// Creates a new LeastRecentlyUsedStrategy
    pub fn new() -> Self {
        Self {}
    }
}

impl LoadBalancingStrategy for LeastRecentlyUsedStrategy {
    /// Select the instance with the oldest last_used timestamp
    ///
    /// # Parameters
    /// * `trackers` - Array of (id, tracker) tuples for available instances
    ///
    /// # Returns
    /// * Index into the trackers array of the least recently used instance
    ///
    /// # Panics
    /// Panics if `trackers` is empty
    fn select_instance(&mut self, trackers: &[(usize, &InstanceTracker)]) -> usize {
        if trackers.is_empty() {
            panic!("LoadBalancingStrategy::select_instance called with empty trackers slice");
        }

        let mut oldest_index = 0;
        let mut oldest_time = trackers[0].1.last_used;

        for (i, (_id, tracker)) in trackers.iter().enumerate().skip(1) {
            if tracker.last_used < oldest_time {
                oldest_index = i;
                oldest_time = tracker.last_used;
            }
        }

        debug!(
            "LeastRecentlyUsedStrategy: Selected index {} (ID: {}) from {} eligible trackers with last_used: {:?}",
            oldest_index, trackers[oldest_index].0, trackers.len(), oldest_time
        );

        oldest_index
    }
}

/// Strategy that selects the instance with the lowest average response time.
#[derive(Debug, Default)]
pub struct LowestLatencyStrategy;

impl LowestLatencyStrategy {
    /// Creates a new LowestLatencyStrategy
    pub fn new() -> Self {
        Self {}
    }
}

impl LoadBalancingStrategy for LowestLatencyStrategy {
    /// Select the instance with the minimum `avg_response_time`.
    ///
    /// # Parameters
    /// * `trackers` - Array of (id, tracker) tuples for available instances.
    ///
    /// # Returns
    /// * Index into the trackers array of the fastest instance.
    ///
    /// # Panics
    /// * Panics if `trackers` is empty.
    fn select_instance(&mut self, trackers: &[(usize, &InstanceTracker)]) -> usize {
        if trackers.is_empty() {
            panic!("LowestLatencyStrategy::select_instance called with empty trackers slice");
        }

        let mut best_index = 0;
        let mut lowest_time = trackers[0].1.avg_response_time();

        for (i, (_id, tracker)) in trackers.iter().enumerate().skip(1) {
            let avg_time = tracker.avg_response_time();
            if avg_time < lowest_time {
                best_index = i;
                lowest_time = avg_time;
            }
        }

        debug!(
            "LowestLatencyStrategy: Selected index {} (ID: {}) from {} eligible trackers with avg_response_time: {:?}",
            best_index, trackers[best_index].0, trackers.len(), lowest_time
        );

        best_index
    }
}

/// Strategy that selects a random instance from the available pool.
#[derive(Debug, Default)]
pub struct RandomStrategy;

impl RandomStrategy {
    /// Creates a new RandomStrategy
    pub fn new() -> Self {
        Self {}
    }
}

impl LoadBalancingStrategy for RandomStrategy {
    /// Select a random instance.
    ///
    /// # Parameters
    /// * `trackers` - Array of (id, tracker) tuples for available instances.
    ///
    /// # Returns
    /// * Index into the trackers array of a randomly chosen instance.
    ///
    /// # Panics
    /// * Panics if `trackers` is empty.
    fn select_instance(&mut self, trackers: &[(usize, &InstanceTracker)]) -> usize {
        if trackers.is_empty() {
            panic!("RandomStrategy::select_instance called with empty trackers slice");
        }

        let index = rand::rng().random_range(0..trackers.len());

        debug!(
            "RandomStrategy: Selected random index {} (ID: {}) from {} eligible trackers",
            index,
            trackers[index].0,
            trackers.len()
        );

        index
    }
}
