use std::{
    collections::{HashSet, VecDeque}, hash::Hash, time::{Duration, Instant}
};

pub struct RateLimiter {
    moments: VecDeque<Instant>,
    time: Duration,
    limit: usize,
}

impl RateLimiter {
    pub fn new(limit: usize, time: Duration) -> Self {
        Self {
            moments: VecDeque::with_capacity(limit),
            time,
            limit,
        }
    }
    pub fn get_token(&mut self) -> bool {
        let now = Instant::now();
        while self
            .moments
            .front()
            .map_or(false, |moment| now - *moment > self.time)
        {
            self.moments.pop_front();
        }
        if self.moments.len() < self.limit {
            self.moments.push_back(now);
            true
        } else {
            false
        }
    }
}

pub struct RingSet<Key: Hash + Eq + Clone> {
    set: HashSet<Key>,
    ring: VecDeque<Key>,
    limit: usize,
}

impl<Key: Hash + Eq + Clone> RingSet<Key> {
    pub fn new(limit: usize) -> Self {
        assert!(limit > 0);
        Self {
            set: HashSet::new(),
            ring: VecDeque::with_capacity(limit),
            limit,
        }
    }

    pub fn add(&mut self, key: Key) {
        if !self.contains(&key) {
            if self.ring.len() >= self.limit {
                let element = self.ring.pop_front().expect("Deque has elements");
                self.set.remove(&element);
            }
            self.set.insert(key.clone());
            self.ring.push_back(key);
        }
    }

    pub fn contains(&self, key: &Key) -> bool {
        self.set.contains(key)
    }
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use super::{RateLimiter, RingSet};

    #[test]
    fn rate_limit() {
        let duration = Duration::from_micros(100);
        let mut limiter = RateLimiter::new(4, duration);

        for _ in 0..4 {
            assert!(limiter.get_token())
        }
        assert!(!limiter.get_token());
        thread::sleep(duration * 2);
        assert!(limiter.get_token());
    }

    #[test]
    fn ring_set() {
        let mut set = RingSet::new(3);
        set.add(1);
        assert!(set.contains(&1));
        set.add(2);
        assert!(set.contains(&1));
        assert!(set.contains(&2));
        assert!(!set.contains(&3));
        set.add(3);
        set.add(3);
        set.add(4);
        assert!(!set.contains(&1));
        assert!(set.contains(&4));
    }
}
