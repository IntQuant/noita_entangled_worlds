use rustc_hash::FxHashSet;
use shared::des::INTEREST_REQUEST_RADIUS;
use shared::{PeerId, des::InterestRequest};
pub(crate) struct InterestTracker {
    radius_hysteresis: f64,
    x: f64,
    y: f64,
    interested_peers: FxHashSet<PeerId>,
    added_any: Vec<PeerId>,
    lost_interest: Vec<PeerId>,
}

impl InterestTracker {
    pub(crate) fn new(radius_hysteresis: f64) -> Self {
        assert!(radius_hysteresis > 0.0);
        Self {
            radius_hysteresis,
            x: 0.0,
            y: 0.0,
            interested_peers: Default::default(),
            lost_interest: Vec::with_capacity(4),
            added_any: Vec::with_capacity(4),
        }
    }

    pub(crate) fn set_center(&mut self, x: f64, y: f64) {
        self.x = x;
        self.y = y;
    }

    pub(crate) fn handle_interest_request(&mut self, peer: PeerId, request: InterestRequest) {
        let rx = request.pos.x as f64;
        let ry = request.pos.y as f64;
        let radius = INTEREST_REQUEST_RADIUS;

        let dist_sq = (rx - self.x).powi(2) + (ry - self.y).powi(2);
        if dist_sq < (radius as f64).powi(2) && self.interested_peers.insert(peer) {
            self.added_any.push(peer);
        }

        if dist_sq > ((radius as f64) + self.radius_hysteresis).powi(2)
            && self.interested_peers.remove(&peer)
        {
            self.lost_interest.push(peer);
        }
    }

    pub(crate) fn got_any_new_interested(&mut self) -> Vec<PeerId> {
        std::mem::take(&mut self.added_any)
    }

    pub(crate) fn drain_lost_interest(&mut self) -> impl Iterator<Item = PeerId> + '_ {
        self.lost_interest.drain(..)
    }

    pub(crate) fn remove_peer(&mut self, peer: PeerId) {
        self.interested_peers.remove(&peer);
        self.added_any.retain(|p| p != &peer);
        self.lost_interest.retain(|p| p != &peer);
    }

    pub(crate) fn iter_interested(&self) -> impl Iterator<Item = PeerId> + '_ {
        self.interested_peers.iter().copied()
    }

    pub(crate) fn contains(&self, peer_id: PeerId) -> bool {
        self.interested_peers.contains(&peer_id)
    }

    pub(crate) fn reset_interest_for(&mut self, source: PeerId) {
        // No need to count peer as "lost_interest" in this case.
        self.interested_peers.remove(&source);
    }
}
