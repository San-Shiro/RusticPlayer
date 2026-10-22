use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use crate::api::Track;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueueSource {
    Context,
    User,
    Radio,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueueLane {
    History,
    Current,
    User,
    Context,
    Radio,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PreviousAction {
    RestartTrack,
    PlayTrack(Track),
    None,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueManager {
    /// Linear playback list of explicit tracks (for backwards compatibility):
    /// - 0..cursor: Past tracks (history, preserved in memory, never removed on next)
    /// - cursor: Currently playing track
    /// - cursor+1..len(): Explicit upcoming tracks (from playlist/album or manually queued)
    #[serde(default)]
    pub actual_queue: Vec<Track>,

    /// Index of the active track within actual_queue
    #[serde(default)]
    pub cursor: usize,

    /// Lane 1: User Priority Queue (manual additions via Play Next / Add to Queue)
    #[serde(default)]
    pub user_queue: VecDeque<Track>,

    /// Lane 2: Context Queue (Album / Playlist tracks)
    #[serde(default)]
    pub context_queue: Vec<Track>,
    #[serde(default)]
    pub context_cursor: usize,

    /// Lane 3: Separate suggestion layer (auto-recommendations / Radio stream)
    #[serde(default)]
    pub suggested_queue: VecDeque<Track>,

    /// Session History (Capped at 50 tracks)
    #[serde(default)]
    pub history_list: Vec<Track>,

    /// Context metadata (if started from an album or playlist)
    #[serde(default)]
    pub context_id: Option<String>,
    #[serde(default)]
    pub context_title: Option<String>,

    /// Endless autoplay toggle
    #[serde(default = "default_true")]
    pub autoplay_enabled: bool,

    /// Shuffle state & preserved original order for unshuffle
    #[serde(default)]
    pub is_shuffled: bool,
    #[serde(default)]
    pub original_upcoming_order: Option<Vec<Track>>,

    /// Reactive generation counter for UI synchronization
    #[serde(default)]
    pub generation: u64,
}

impl Default for QueueManager {
    fn default() -> Self {
        Self {
            actual_queue: Vec::new(),
            cursor: 0,
            user_queue: VecDeque::new(),
            context_queue: Vec::new(),
            context_cursor: 0,
            suggested_queue: VecDeque::new(),
            history_list: Vec::new(),
            context_id: None,
            context_title: None,
            autoplay_enabled: true,
            is_shuffled: false,
            original_upcoming_order: None,
            generation: 0,
        }
    }
}

impl QueueManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Currently playing track (reference).
    #[inline]
    pub fn current(&self) -> Option<&Track> {
        self.actual_queue.get(self.cursor)
    }

    /// Currently playing track (cloned).
    #[inline]
    pub fn current_cloned(&self) -> Option<Track> {
        self.actual_queue.get(self.cursor).cloned()
    }

    /// Past played tracks in the current session (slice).
    #[inline]
    pub fn history(&self) -> &[Track] {
        let limit = self.cursor.min(self.actual_queue.len());
        &self.actual_queue[..limit]
    }

    /// Explicit upcoming tracks in the current queue (slice).
    #[inline]
    pub fn upcoming(&self) -> &[Track] {
        if self.cursor + 1 < self.actual_queue.len() {
            &self.actual_queue[self.cursor + 1..]
        } else {
            &[]
        }
    }

    /// Suggested / radio recommendation tracks.
    #[inline]
    pub fn suggestions(&self) -> &VecDeque<Track> {
        &self.suggested_queue
    }

    /// Returns true if either actual_queue has upcoming tracks or autoplay has suggestions ready.
    #[inline]
    pub fn has_next(&self) -> bool {
        self.cursor + 1 < self.actual_queue.len()
            || (!self.suggested_queue.is_empty() && self.autoplay_enabled)
    }

    /// Returns true when suggested_queue is running low and autoplay is enabled.
    #[inline]
    pub fn needs_prefetch(&self) -> bool {
        self.autoplay_enabled && self.suggested_queue.len() < 5
    }

    /// Advances to the next track:
    /// 1. If explicit upcoming songs exist in `actual_queue`, advances `cursor += 1`.
    /// 2. If `actual_queue` is exhausted and `autoplay_enabled` is true, pops 1 suggestion from
    ///    `suggested_queue`, appends it to `actual_queue`, and advances cursor onto it.
    /// Invariant: Past tracks in `actual_queue[..cursor]` are NEVER erased.
    pub fn advance(&mut self) -> Option<Track> {
        self.generation += 1;

        // Priority 1: User Priority Queue (Lane 1)
        if let Some(user_track) = self.user_queue.pop_front() {
            // If already at next pos in actual_queue, advance onto it
            if self.cursor + 1 < self.actual_queue.len() && self.actual_queue[self.cursor + 1].id == user_track.id {
                self.cursor += 1;
                self.history_list = self.actual_queue[..self.cursor].to_vec();
                return Some(user_track);
            } else {
                let insert_pos = (self.cursor + 1).min(self.actual_queue.len());
                if let Some(existing_pos) = self.actual_queue[insert_pos..].iter().position(|t| t.id == user_track.id) {
                    self.actual_queue.remove(insert_pos + existing_pos);
                }
                self.actual_queue.insert(insert_pos, user_track.clone());
                self.cursor = insert_pos;
                self.history_list = self.actual_queue[..self.cursor].to_vec();
                return Some(user_track);
            }
        }

        // Priority 2: Context Queue in actual_queue (Lane 2)
        if self.cursor + 1 < self.actual_queue.len() {
            self.cursor += 1;
            self.history_list = self.actual_queue[..self.cursor].to_vec();
            return self.current_cloned();
        }

        // Priority 3: Radio / Suggestions (Lane 3)
        if self.autoplay_enabled {
            if let Some(next_sugg) = self.suggested_queue.pop_front() {
                self.actual_queue.push(next_sugg.clone());
                self.cursor = self.actual_queue.len() - 1;
                self.history_list = self.actual_queue[..self.cursor].to_vec();
                return Some(next_sugg);
            }
        }

        None
    }

    /// Alias to advance() for compatibility.
    #[inline]
    pub fn next(&mut self) -> Option<Track> {
        self.advance()
    }

    /// Navigates back:
    /// - If position > 3.0s: restarts current track from 0.0s.
    /// - Else if `cursor > 0`: steps cursor back (`cursor -= 1`) and returns previous track.
    /// - Else if at track 0: restarts current track (never cycles).
    pub fn previous(&mut self, current_position_secs: f64) -> PreviousAction {
        self.generation += 1;
        if current_position_secs > 3.0 {
            return PreviousAction::RestartTrack;
        }

        if self.cursor > 0 {
            self.cursor -= 1;
            self.history_list = self.actual_queue[..self.cursor].to_vec();
            if let Some(track) = self.current_cloned() {
                return PreviousAction::PlayTrack(track);
            }
        }

        self.history_list.clear();

        if !self.actual_queue.is_empty() {
            PreviousAction::RestartTrack
        } else {
            PreviousAction::None
        }
    }

    /// Sets an album or playlist context, resetting the queue to the selected collection.
    /// `start_index` is the initially selected track.
    /// Invariant: `suggested_queue` is cleared; fresh suggestions can be fetched for when this album ends.
    pub fn set_context(
        &mut self,
        id: impl Into<String>,
        title: impl Into<String>,
        tracks: Vec<Track>,
        start_index: usize,
        shuffle: bool,
    ) -> Option<Track> {
        self.generation += 1;
        self.context_id = Some(id.into());
        self.context_title = Some(title.into());
        self.context_queue = tracks.clone();
        self.actual_queue = tracks;
        self.cursor = if self.actual_queue.is_empty() {
            0
        } else {
            start_index.min(self.actual_queue.len() - 1)
        };
        self.context_cursor = self.cursor;
        self.user_queue.clear();
        self.suggested_queue.clear();
        self.is_shuffled = shuffle;
        self.original_upcoming_order = None;

        if shuffle && self.cursor + 1 < self.actual_queue.len() {
            self.original_upcoming_order = Some(self.actual_queue.clone());
            use rand::seq::SliceRandom;
            let mut rng = rand::thread_rng();
            self.actual_queue[self.cursor + 1..].shuffle(&mut rng);
            self.context_queue = self.actual_queue.clone();
        }

        self.current_cloned()
    }

    /// Plays a single standalone track (e.g. from search result or direct song click).
    /// Invariant: Starts a fresh single-item `actual_queue`; suggestions are cleared to await fresh radio recommendations.
    pub fn play_single(&mut self, track: Track) -> Track {
        self.context_id = None;
        self.context_title = None;
        self.actual_queue = vec![track.clone()];
        self.cursor = 0;
        self.suggested_queue.clear();
        self.is_shuffled = false;
        self.original_upcoming_order = None;
        track
    }

    /// Alias for play_single.
    #[inline]
    pub fn play_track_directly(&mut self, track: Track) {
        self.play_single(track);
    }

    /// Appends a track to the end of Lane 1 (User Priority Queue).
    /// Invariant: Guaranteed to play before any songs from `suggested_queue`.
    pub fn enqueue_end(&mut self, track: Track) {
        self.generation += 1;
        self.user_queue.push_back(track.clone());
        self.actual_queue.push(track);
    }

    /// Alias for enqueue_end.
    #[inline]
    pub fn add_to_queue(&mut self, track: Track) {
        self.enqueue_end(track);
    }

    /// Inserts a track to play immediately after the current song (`cursor + 1`) in Lane 1.
    pub fn enqueue_next(&mut self, track: Track) {
        self.generation += 1;
        self.user_queue.push_front(track.clone());
        let insert_pos = (self.cursor + 1).min(self.actual_queue.len());
        self.actual_queue.insert(insert_pos, track);
    }

    /// Alias for enqueue_next.
    #[inline]
    pub fn play_next(&mut self, track: Track) {
        self.enqueue_next(track);
    }

    /// Clears only Lane 1 (User Queue), leaving context queue and active track intact.
    pub fn clear_user_queue(&mut self) {
        self.generation += 1;
        let user_ids: std::collections::HashSet<String> = self.user_queue.iter().map(|t| t.id.clone()).collect();
        self.user_queue.clear();
        if self.cursor + 1 < self.actual_queue.len() {
            let mut i = self.cursor + 1;
            while i < self.actual_queue.len() {
                if user_ids.contains(&self.actual_queue[i].id) {
                    self.actual_queue.remove(i);
                } else {
                    i += 1;
                }
            }
        }
    }

    /// Removes an item from Lane 1 (User Queue) by index.
    pub fn remove_user_queue_item(&mut self, index: usize) -> Option<Track> {
        if index < self.user_queue.len() {
            self.generation += 1;
            let removed = self.user_queue.remove(index)?;
            if let Some(pos) = self.actual_queue[self.cursor + 1..].iter().position(|t| t.id == removed.id) {
                self.actual_queue.remove(self.cursor + 1 + pos);
            }
            Some(removed)
        } else {
            None
        }
    }

    /// Reorders items within Lane 1 (User Queue).
    pub fn move_user_queue_item(&mut self, from: usize, to: usize) -> bool {
        if from < self.user_queue.len() && to < self.user_queue.len() && from != to {
            self.generation += 1;
            if let Some(t) = self.user_queue.remove(from) {
                self.user_queue.insert(to, t);
                return true;
            }
        }
        false
    }

    /// Promotes an item from Lane 3 (Radio) into the bottom of `actual_queue` (Lane 2).
    pub fn promote_radio_item(&mut self, index: usize) -> Option<Track> {
        if let Some(track) = self.suggested_queue.remove(index) {
            self.generation += 1;
            self.actual_queue.push(track.clone());
            Some(track)
        } else {
            None
        }
    }

    /// Sets or replaces upcoming tracks in `actual_queue`, preserving history and current track.
    pub fn set_upcoming_tracks(&mut self, tracks: Vec<Track>) {
        self.generation += 1;
        let keep_len = (self.cursor + 1).min(self.actual_queue.len());
        self.actual_queue.truncate(keep_len);
        self.actual_queue.extend(tracks);
    }

    /// Jumps cursor directly to an absolute index in `actual_queue` without wiping the queue.
    pub fn jump_to(&mut self, index: usize) -> Option<Track> {
        if index < self.actual_queue.len() {
            self.cursor = index;
            self.history_list = self.actual_queue[..self.cursor].to_vec();
            return self.current_cloned();
        }
        None
    }

    /// Jumps cursor directly to a track with `track_id` in `actual_queue` or promotes from `suggested_queue`.
    pub fn jump_to_track_id(&mut self, track_id: &str) -> Option<Track> {
        if let Some(pos) = self.actual_queue.iter().position(|t| t.id == track_id) {
            self.generation += 1;
            self.cursor = pos;
            self.history_list = self.actual_queue[..self.cursor].to_vec();
            return self.current_cloned();
        }
        if let Some(pos) = self.suggested_queue.iter().position(|t| t.id == track_id) {
            return self.promote_suggestion(pos);
        }
        None
    }

    /// Promotes a track from `suggested_queue` into `actual_queue` right after current,
    /// advancing cursor onto it immediately.
    pub fn promote_suggestion(&mut self, suggestion_idx: usize) -> Option<Track> {
        if let Some(track) = self.suggested_queue.remove(suggestion_idx) {
            self.generation += 1;
            let insert_pos = (self.cursor + 1).min(self.actual_queue.len());
            self.actual_queue.insert(insert_pos, track.clone());
            self.cursor = insert_pos;
            self.history_list = self.actual_queue[..self.cursor].to_vec();
            return Some(track);
        }
        None
    }

    /// Generates a comprehensive MultiLaneQueueSnapshot for reactive UI presentation.
    pub fn snapshot(&self) -> crate::api::MultiLaneQueueSnapshot {
        let user_ids: std::collections::HashSet<String> = self.user_queue.iter().map(|t| t.id.clone()).collect();
        let context_upcoming: Vec<Track> = if self.cursor + 1 < self.actual_queue.len() {
            self.actual_queue[self.cursor + 1..]
                .iter()
                .filter(|t| !user_ids.contains(&t.id))
                .cloned()
                .collect()
        } else {
            Vec::new()
        };

        crate::api::MultiLaneQueueSnapshot {
            current_track: self.current_cloned(),
            actual_queue: self.actual_queue.clone(),
            cursor: self.cursor,
            user_queue: self.user_queue.iter().cloned().collect(),
            context_title: self.context_title.clone(),
            context_id: self.context_id.clone(),
            context_upcoming,
            radio_upcoming: self.suggested_queue.iter().cloned().collect(),
            history: self.history().to_vec(),
            autoplay_enabled: self.autoplay_enabled,
            is_shuffled: self.is_shuffled,
            generation: self.generation,
        }
    }

    /// Ingests auto-recommendations into `suggested_queue`.
    /// Invariant: Dedupes against both `actual_queue` and existing `suggested_queue`.
    /// Never mutates `cursor` or `actual_queue`.
    pub fn ingest_suggestions(&mut self, tracks: Vec<Track>) {
        for t in tracks {
            let tid = &t.id;
            let in_actual = self.actual_queue.iter().any(|it| &it.id == tid);
            let in_sugg = self.suggested_queue.iter().any(|it| &it.id == tid);

            if !in_actual && !in_sugg {
                self.suggested_queue.push_back(t);
            }
        }
    }

    /// Alias to ingest_suggestions for compatibility.
    #[inline]
    pub fn ingest_radio(&mut self, tracks: Vec<Track>) {
        self.ingest_suggestions(tracks);
    }

    /// Removes an item from explicit upcoming tracks (`actual_queue[cursor + 1 ..]`).
    /// `upcoming_idx` is 0-based relative to the start of upcoming tracks.
    pub fn remove_upcoming(&mut self, upcoming_idx: usize) -> Option<Track> {
        let actual_idx = self.cursor + 1 + upcoming_idx;
        if actual_idx < self.actual_queue.len() {
            Some(self.actual_queue.remove(actual_idx))
        } else {
            None
        }
    }

    /// Reorders explicit upcoming tracks (`actual_queue[cursor + 1 ..]`).
    /// `from_upcoming` and `to_upcoming` are 0-based relative to the upcoming slice.
    pub fn reorder_upcoming(&mut self, from_upcoming: usize, to_upcoming: usize) -> bool {
        let actual_from = self.cursor + 1 + from_upcoming;
        let actual_to = self.cursor + 1 + to_upcoming;
        let len = self.actual_queue.len();

        if actual_from < len && actual_to < len && actual_from != actual_to {
            let t = self.actual_queue.remove(actual_from);
            self.actual_queue.insert(actual_to, t);
            true
        } else {
            false
        }
    }

    /// Returns upcoming tracks formatted with lane categorization for UI compatibility.
    pub fn upcoming_tracks(&self) -> Vec<(QueueLane, Track)> {
        let mut list = Vec::new();
        // Lane 1: User Priority Queue
        for t in &self.user_queue {
            list.push((QueueLane::User, t.clone()));
        }
        // Lane 2: Context Upcoming
        let user_ids: std::collections::HashSet<String> = self.user_queue.iter().map(|t| t.id.clone()).collect();
        for t in self.upcoming() {
            if !user_ids.contains(&t.id) {
                list.push((QueueLane::Context, t.clone()));
            }
        }
        // Lane 3: Radio / Suggestions
        for t in &self.suggested_queue {
            list.push((QueueLane::Radio, t.clone()));
        }
        list
    }

    /// Produces a capped snapshot of `actual_queue` for persistence so history does not grow unbounded over days.
    pub fn prepare_for_persistence(&self, max_history: usize) -> (Vec<Track>, usize) {
        if self.cursor <= max_history {
            (self.actual_queue.clone(), self.cursor)
        } else {
            let drop_count = self.cursor - max_history;
            let bounded_queue = self.actual_queue[drop_count..].to_vec();
            let bounded_cursor = max_history;
            (bounded_queue, bounded_cursor)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::TrackSource;

    fn mock_track(id: &str, title: &str) -> Track {
        Track {
            id: id.to_string(),
            title: title.to_string(),
            artists: vec!["Artist".to_string()],
            album: "Album".to_string(),
            duration_secs: 200,
            source: TrackSource::Local(format!("/path/{id}.flac").into()),
            isrc: None,
            artwork_url: None,
            content_hash: None,
        }
    }

    #[test]
    fn test_cursor_progression_and_history_retention() {
        let mut qm = QueueManager::new();
        let t1 = mock_track("1", "Track 1");
        let t2 = mock_track("2", "Track 2");
        let t3 = mock_track("3", "Track 3");

        qm.set_context("album1", "Album 1", vec![t1.clone(), t2.clone(), t3.clone()], 0, false);

        assert_eq!(qm.current().unwrap().id, "1");
        assert_eq!(qm.history().len(), 0);
        assert_eq!(qm.upcoming().len(), 2);

        // Advance to track 2
        let next1 = qm.advance();
        assert_eq!(next1.unwrap().id, "2");
        assert_eq!(qm.cursor, 1);
        assert_eq!(qm.history().len(), 1);
        assert_eq!(qm.history()[0].id, "1"); // History stays in actual_queue!
        assert_eq!(qm.upcoming().len(), 1);

        // Advance to track 3
        let next2 = qm.advance();
        assert_eq!(next2.unwrap().id, "3");
        assert_eq!(qm.cursor, 2);
        assert_eq!(qm.history().len(), 2);
        assert_eq!(qm.history()[0].id, "1");
        assert_eq!(qm.history()[1].id, "2");
        assert_eq!(qm.upcoming().len(), 0);
    }

    #[test]
    fn test_suggestions_pulled_only_when_explicit_queue_exhausted() {
        let mut qm = QueueManager::new();
        qm.autoplay_enabled = true;
        let t1 = mock_track("1", "Track 1");
        let t2 = mock_track("2", "Track 2");
        let s1 = mock_track("s1", "Suggestion 1");
        let s2 = mock_track("s2", "Suggestion 2");

        qm.set_context("playlist", "Playlist", vec![t1, t2], 0, false);
        qm.ingest_suggestions(vec![s1.clone(), s2.clone()]);

        assert_eq!(qm.actual_queue.len(), 2);
        assert_eq!(qm.suggested_queue.len(), 2);

        // First advance goes to t2 (explicit track)
        let n1 = qm.advance().unwrap();
        assert_eq!(n1.id, "2");
        assert_eq!(qm.actual_queue.len(), 2);
        assert_eq!(qm.suggested_queue.len(), 2); // Suggestions still untouched!

        // Second advance exhausts explicit queue -> pops s1 from suggestions into actual_queue
        let n2 = qm.advance().unwrap();
        assert_eq!(n2.id, "s1");
        assert_eq!(qm.actual_queue.len(), 3);
        assert_eq!(qm.actual_queue[2].id, "s1");
        assert_eq!(qm.suggested_queue.len(), 1);
        assert_eq!(qm.suggested_queue[0].id, "s2");
    }

    #[test]
    fn test_add_to_queue_plays_before_suggestions() {
        let mut qm = QueueManager::new();
        qm.autoplay_enabled = true;
        let t1 = mock_track("1", "Track 1");
        let s1 = mock_track("s1", "Suggestion 1");
        let manual = mock_track("manual", "Manually Queued Track");

        qm.play_single(t1);
        qm.ingest_suggestions(vec![s1]);

        // User clicks "Add to Queue"
        qm.add_to_queue(manual.clone());

        assert_eq!(qm.actual_queue.len(), 2);
        assert_eq!(qm.suggested_queue.len(), 1);

        // Next song MUST be manual track, NOT suggestion!
        let next_track = qm.advance().unwrap();
        assert_eq!(next_track.id, "manual");

        // Next song after manual track is suggestion 1
        let after_manual = qm.advance().unwrap();
        assert_eq!(after_manual.id, "s1");
    }

    #[test]
    fn test_jump_to_preserves_entire_queue() {
        let mut qm = QueueManager::new();
        let tracks = vec![
            mock_track("1", "1"),
            mock_track("2", "2"),
            mock_track("3", "3"),
            mock_track("4", "4"),
        ];
        qm.set_context("album", "Album", tracks, 0, false);

        // Jump to track 3 (index 2)
        let jumped = qm.jump_to(2).unwrap();
        assert_eq!(jumped.id, "3");
        assert_eq!(qm.cursor, 2);
        assert_eq!(qm.history().len(), 2);
        assert_eq!(qm.history()[0].id, "1");
        assert_eq!(qm.history()[1].id, "2");
        assert_eq!(qm.upcoming().len(), 1);
        assert_eq!(qm.upcoming()[0].id, "4");

        // Jump back to track 1 (index 0)
        let back = qm.jump_to(0).unwrap();
        assert_eq!(back.id, "1");
        assert_eq!(qm.cursor, 0);
        assert_eq!(qm.history().len(), 0);
        assert_eq!(qm.upcoming().len(), 3);
    }

    #[test]
    fn test_previous_navigation() {
        let mut qm = QueueManager::new();
        let tracks = vec![mock_track("1", "1"), mock_track("2", "2")];
        qm.set_context("album", "Album", tracks, 1, false);

        // If position > 3.0s, restarts
        assert_eq!(qm.previous(4.5), PreviousAction::RestartTrack);
        assert_eq!(qm.cursor, 1);

        // If position <= 3.0s, steps back to track 1
        match qm.previous(1.2) {
            PreviousAction::PlayTrack(t) => assert_eq!(t.id, "1"),
            _ => panic!("Expected PlayTrack"),
        }
        assert_eq!(qm.cursor, 0);

        // Calling previous at track 0 restarts track 1
        assert_eq!(qm.previous(1.0), PreviousAction::RestartTrack);
        assert_eq!(qm.cursor, 0);
    }

    #[test]
    fn test_reorder_and_remove_upcoming() {
        let mut qm = QueueManager::new();
        let tracks = vec![
            mock_track("1", "1"),
            mock_track("2", "2"),
            mock_track("3", "3"),
            mock_track("4", "4"),
        ];
        qm.set_context("album", "Album", tracks, 0, false);

        // Upcoming is [2, 3, 4]
        assert_eq!(qm.upcoming()[0].id, "2");
        assert_eq!(qm.upcoming()[1].id, "3");
        assert_eq!(qm.upcoming()[2].id, "4");

        // Reorder upcoming: move 4 (upcoming idx 2) to upcoming idx 0
        assert!(qm.reorder_upcoming(2, 0));
        assert_eq!(qm.upcoming()[0].id, "4");
        assert_eq!(qm.upcoming()[1].id, "2");
        assert_eq!(qm.upcoming()[2].id, "3");

        // Remove upcoming 2 (upcoming idx 1)
        let removed = qm.remove_upcoming(1).unwrap();
        assert_eq!(removed.id, "2");
        assert_eq!(qm.upcoming().len(), 2);
        assert_eq!(qm.upcoming()[0].id, "4");
        assert_eq!(qm.upcoming()[1].id, "3");
    }

    #[test]
    fn test_3_lane_priority_user_queue_over_context_and_radio() {
        let mut qm = QueueManager::new();
        qm.autoplay_enabled = true;
        let c1 = mock_track("c1", "Context 1");
        let c2 = mock_track("c2", "Context 2");
        let u1 = mock_track("u1", "User 1");
        let r1 = mock_track("r1", "Radio 1");

        // Start album with c1, c2
        qm.set_context("album", "Album", vec![c1.clone(), c2.clone()], 0, false);
        // Radio stream has r1
        qm.ingest_radio(vec![r1.clone()]);
        // User explicitly queues u1 via "Play Next"
        qm.enqueue_next(u1.clone());

        // First advance MUST play User Queue (u1), not c2 or r1
        let next1 = qm.advance().unwrap();
        assert_eq!(next1.id, "u1");

        // Second advance MUST play Context Queue (c2), not r1
        let next2 = qm.advance().unwrap();
        assert_eq!(next2.id, "c2");

        // Third advance MUST play Radio Queue (r1) because user & context are exhausted
        let next3 = qm.advance().unwrap();
        assert_eq!(next3.id, "r1");

        // Fourth advance is None (empty)
        assert!(qm.advance().is_none());
    }

    #[test]
    fn test_clear_user_queue_preserves_context() {
        let mut qm = QueueManager::new();
        let c1 = mock_track("c1", "Context 1");
        let c2 = mock_track("c2", "Context 2");
        let u1 = mock_track("u1", "User 1");

        qm.set_context("album", "Album", vec![c1, c2], 0, false);
        qm.enqueue_next(u1);

        // Clear user queue
        qm.clear_user_queue();
        assert_eq!(qm.user_queue.len(), 0);

        // Next song MUST be c2 (context song is intact!)
        let next = qm.advance().unwrap();
        assert_eq!(next.id, "c2");
    }

    #[test]
    fn test_snapshot_separates_lanes() {
        let mut qm = QueueManager::new();
        let c1 = mock_track("c1", "Context 1");
        let c2 = mock_track("c2", "Context 2");
        let u1 = mock_track("u1", "User 1");
        let r1 = mock_track("r1", "Radio 1");

        qm.set_context("album", "Album", vec![c1, c2], 0, false);
        qm.enqueue_next(u1);
        qm.ingest_radio(vec![r1]);

        let snap = qm.snapshot();
        assert_eq!(snap.current_track.unwrap().id, "c1");
        assert_eq!(snap.user_queue.len(), 1);
        assert_eq!(snap.user_queue[0].id, "u1");
        assert_eq!(snap.context_upcoming.len(), 1);
        assert_eq!(snap.context_upcoming[0].id, "c2");
        assert_eq!(snap.radio_upcoming.len(), 1);
        assert_eq!(snap.radio_upcoming[0].id, "r1");
    }

    #[test]
    fn test_previous_clamps_at_top_no_cycle() {
        let mut qm = QueueManager::new();
        let t1 = mock_track("t1", "Track 1");
        let t2 = mock_track("t2", "Track 2");
        let t3 = mock_track("t3", "Track 3");
        qm.set_context("album", "Album", vec![t1, t2, t3], 0, false);

        // Advance to t2, then t3
        qm.advance();
        assert_eq!(qm.cursor, 1);
        qm.advance();
        assert_eq!(qm.cursor, 2);

        // Previous steps to t2
        match qm.previous(1.0) {
            PreviousAction::PlayTrack(t) => assert_eq!(t.id, "t2"),
            _ => panic!("Expected PlayTrack t2"),
        }
        assert_eq!(qm.cursor, 1);
        assert_eq!(qm.history_list.len(), 1);

        // Previous steps to t1
        match qm.previous(1.0) {
            PreviousAction::PlayTrack(t) => assert_eq!(t.id, "t1"),
            _ => panic!("Expected PlayTrack t1"),
        }
        assert_eq!(qm.cursor, 0);
        assert_eq!(qm.history_list.len(), 0);

        // Previous at track 0 restarts track and NEVER cycles
        match qm.previous(1.0) {
            PreviousAction::RestartTrack => {}
            _ => panic!("Expected RestartTrack at top of queue"),
        }
        assert_eq!(qm.cursor, 0);
        assert_eq!(qm.history_list.len(), 0);

        // Additional previous calls still stay clamped at track 0
        match qm.previous(0.0) {
            PreviousAction::RestartTrack => {}
            _ => panic!("Expected RestartTrack at top of queue"),
        }
        assert_eq!(qm.cursor, 0);
        assert_eq!(qm.history_list.len(), 0);
    }

    #[test]
    fn test_promote_radio_adds_to_user_queue() {
        let mut qm = QueueManager::new();
        let t1 = mock_track("t1", "Track 1");
        let t2 = mock_track("t2", "Track 2");
        let r1 = mock_track("r1", "Radio 1");
        qm.set_context("album", "Album", vec![t1, t2], 0, false);
        qm.ingest_radio(vec![r1]);

        assert_eq!(qm.suggested_queue.len(), 1);
        assert_eq!(qm.user_queue.len(), 0);

        // Promote radio item
        let promoted = qm.promote_radio_item(0).unwrap();
        assert_eq!(promoted.id, "r1");
        assert_eq!(qm.suggested_queue.len(), 0);

        // Promoted track should now be at the bottom of actual_queue (Lane 2)
        assert_eq!(qm.actual_queue.len(), 3);
        assert_eq!(qm.actual_queue.last().unwrap().id, "r1");
        let snap = qm.snapshot();
        assert_eq!(snap.context_upcoming.len(), 2);
        assert_eq!(snap.context_upcoming.last().unwrap().id, "r1");
    }
}

