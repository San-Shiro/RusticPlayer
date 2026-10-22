
pub struct MockTrack {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_ms: u64,
    pub is_explicit: bool,
    pub source: String, // "spotify" or "ytmusic"
}

pub struct MockAlbum {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub year: u32,
}

pub struct MockArtist {
    pub id: String,
    pub name: String,
}

pub fn get_dummy_tracks() -> Vec<MockTrack> {
    vec![
        MockTrack {
            id: "t1".to_string(),
            title: "Bohemian Rhapsody".to_string(),
            artist: "Queen".to_string(),
            album: "A Night at the Opera".to_string(),
            duration_ms: 354000,
            is_explicit: false,
            source: "spotify".to_string(),
        },
        MockTrack {
            id: "t2".to_string(),
            title: "Hotel California".to_string(),
            artist: "Eagles".to_string(),
            album: "Hotel California".to_string(),
            duration_ms: 390000,
            is_explicit: false,
            source: "ytmusic".to_string(),
        },
        MockTrack {
            id: "t3".to_string(),
            title: "Stairway to Heaven".to_string(),
            artist: "Led Zeppelin".to_string(),
            album: "Led Zeppelin IV".to_string(),
            duration_ms: 482000,
            is_explicit: false,
            source: "spotify".to_string(),
        },
        MockTrack {
            id: "t4".to_string(),
            title: "Smells Like Teen Spirit".to_string(),
            artist: "Nirvana".to_string(),
            album: "Nevermind".to_string(),
            duration_ms: 301000,
            is_explicit: true,
            source: "ytmusic".to_string(),
        },
        MockTrack {
            id: "t5".to_string(),
            title: "Sweet Child O' Mine".to_string(),
            artist: "Guns N' Roses".to_string(),
            album: "Appetite for Destruction".to_string(),
            duration_ms: 356000,
            is_explicit: false,
            source: "spotify".to_string(),
        },
    ]
}

pub fn get_dummy_albums() -> Vec<MockAlbum> {
    vec![
        MockAlbum {
            id: "a1".to_string(),
            title: "A Night at the Opera".to_string(),
            artist: "Queen".to_string(),
            year: 1975,
        },
        MockAlbum {
            id: "a2".to_string(),
            title: "Hotel California".to_string(),
            artist: "Eagles".to_string(),
            year: 1976,
        },
        MockAlbum {
            id: "a3".to_string(),
            title: "Nevermind".to_string(),
            artist: "Nirvana".to_string(),
            year: 1991,
        },
    ]
}

pub fn get_dummy_artists() -> Vec<MockArtist> {
    vec![
        MockArtist {
            id: "ar1".to_string(),
            name: "Queen".to_string(),
        },
        MockArtist {
            id: "ar2".to_string(),
            name: "Eagles".to_string(),
        },
        MockArtist {
            id: "ar3".to_string(),
            name: "Led Zeppelin".to_string(),
        },
        MockArtist {
            id: "ar4".to_string(),
            name: "Nirvana".to_string(),
        },
    ]
}
