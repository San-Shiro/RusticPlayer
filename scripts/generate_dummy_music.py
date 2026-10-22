import os
import sqlite3
import subprocess
import hashlib
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor, as_completed
from PIL import Image, ImageDraw, ImageFont

BASE_DIR = Path("f:/VSCodium/Github/RusticPlayer")
MUSIC_DIR = BASE_DIR / "dummy_library"
ARTWORK_DIR = BASE_DIR / "assets" / "artwork"
CACHE_ARTWORK_DIR = BASE_DIR / "cache" / "artwork"
DB_PATH = BASE_DIR / "rustic_library.db"

MUSIC_DIR.mkdir(parents=True, exist_ok=True)
ARTWORK_DIR.mkdir(parents=True, exist_ok=True)
CACHE_ARTWORK_DIR.mkdir(parents=True, exist_ok=True)

# 12 Diverse Albums totaling 101 songs
ALBUMS = [
    {
        "id": "neon_horizon",
        "title": "Neon Horizon",
        "artist": "Vektor Prime",
        "genre": "Synthwave",
        "year": 2024,
        "colors": [(18, 12, 38), (255, 41, 116), (41, 234, 255)],
        "tracks": [
            "Midnight Circuit", "Chrome Sunset", "Cyber Drift", "Neon Boulevard",
            "Analog Horizon", "Laser Grid", "Retrofutura", "Grid Runner", "Starlight Overdrive"
        ]
    },
    {
        "id": "rainy_afternoon",
        "title": "Rainy Afternoon",
        "artist": "Komorebi",
        "genre": "Lo-Fi Beats",
        "year": 2023,
        "colors": [(35, 25, 20), (198, 114, 69), (235, 190, 140)],
        "tracks": [
            "Coffee & Raindrops", "Window Fog", "Late Study Session", "Warm Blanket",
            "Cat on the Sill", "Sunday Solitude", "Paper Planes", "Lo-Fi Dusk", "Sleeping City"
        ]
    },
    {
        "id": "velvet_midnight",
        "title": "Velvet Midnight",
        "artist": "The Silk Quartet",
        "genre": "Jazz & Soul",
        "year": 2022,
        "colors": [(10, 15, 30), (180, 140, 60), (220, 200, 130)],
        "tracks": [
            "Bourbon & Brass", "Autumn in Montmartre", "Blue Moon Serenade", "Smoky Tavern",
            "Late Night Walk", "Silk & Satin", "Dimly Lit Room", "Harlem Rain"
        ]
    },
    {
        "id": "nordic_echoes",
        "title": "Nordic Echoes",
        "artist": "Astrid Lind",
        "genre": "Ambient Piano",
        "year": 2024,
        "colors": [(15, 28, 36), (64, 128, 140), (200, 230, 240)],
        "tracks": [
            "Fjord Mist", "Northern Lights", "Glacier Reverie", "Silent Pines",
            "Winter Solstice", "Frozen Lake", "First Snowfall", "Echoes of the North"
        ]
    },
    {
        "id": "tokyo_1984",
        "title": "Tokyo 1984",
        "artist": "Hiroshi & The Waves",
        "genre": "City Pop",
        "year": 2023,
        "colors": [(20, 20, 45), (255, 90, 120), (255, 210, 80)],
        "tracks": [
            "Shinjuku Twilight", "Bay Area Freeway", "Cassette Memories", "Plastic Romance",
            "Weekend Drive", "City Lights Romance", "Telephone Line", "Seaside Boulevard", "Midnight Train"
        ]
    },
    {
        "id": "solar_flare",
        "title": "Solar Flare",
        "artist": "Stellar Odyssey",
        "genre": "Space Ambient",
        "year": 2024,
        "colors": [(8, 8, 24), (110, 40, 180), (255, 120, 50)],
        "tracks": [
            "Interstellar Drift", "Nebula Glow", "Event Horizon", "Cosmic Dust",
            "Orbital Resonance", "Pulsar Beats", "Dark Matter", "Voyager Beyond"
        ]
    },
    {
        "id": "dust_and_timber",
        "title": "Dust & Timber",
        "artist": "The Mountain Pines",
        "genre": "Acoustic Folk",
        "year": 2021,
        "colors": [(40, 30, 20), (140, 90, 50), (210, 180, 140)],
        "tracks": [
            "Campfire Embers", "Winding Trail", "Appalachian River", "Cedar Breeze",
            "Old Wooden Porch", "Morning Dew", "Hollow Valley", "Mountain Hymn", "Homeward Bound"
        ]
    },
    {
        "id": "analog_overdrive",
        "title": "Analog Overdrive",
        "artist": "Red Distortion",
        "genre": "Indie Rock",
        "year": 2023,
        "colors": [(30, 10, 10), (220, 40, 40), (240, 200, 100)],
        "tracks": [
            "Garage Anthem", "Fuzzbox Summer", "Broken Strings", "Electric Heartbeat",
            "Basement Tapes", "Feedback Loop", "Neon Youth", "Last Exit Tonight"
        ]
    },
    {
        "id": "subterranean_groove",
        "title": "Subterranean Groove",
        "artist": "Deep Pulse",
        "genre": "Minimal Techno",
        "year": 2024,
        "colors": [(10, 10, 15), (50, 180, 120), (160, 240, 200)],
        "tracks": [
            "Warehouse 04", "Hypnotic State", "Low Frequency", "Berlin Dawn",
            "Monochrome Rhythm", "Sub-Bass Meditation", "Echo Chamber", "Modular Dreams", "Industrial Soul"
        ]
    },
    {
        "id": "golden_hour",
        "title": "Golden Hour",
        "artist": "Maya Cruz",
        "genre": "Neo-Soul",
        "year": 2022,
        "colors": [(35, 15, 25), (200, 100, 130), (250, 210, 160)],
        "tracks": [
            "Honey & Velvet", "Sunset Confessions", "Warm Breeze", "Slow Dance",
            "Sunday Glow", "Soulmate Frequency", "Lavender Tea", "After Midnight"
        ]
    },
    {
        "id": "electric_odyssey",
        "title": "Electric Odyssey",
        "artist": "Hyperion",
        "genre": "French House",
        "year": 2024,
        "colors": [(15, 10, 35), (140, 50, 220), (255, 60, 180)],
        "tracks": [
            "Disco Rebirth", "Starlight Touch", "Filter Paradise", "Champs-Élysées",
            "Midnight Express", "Groove Machine", "French Touch Anthem", "Forever Dancing"
        ]
    },
    {
        "id": "quiet_mind",
        "title": "Quiet Mind",
        "artist": "Zen Garden Ensemble",
        "genre": "Meditation",
        "year": 2023,
        "colors": [(20, 30, 25), (70, 140, 100), (190, 225, 200)],
        "tracks": [
            "Bamboo Water", "Morning Lotus", "Breath of Clarity", "Tibetan Singing Bowl",
            "Zen Stones", "Tranquil Stream", "Mindful Horizon", "Inner Peace"
        ]
    }
]

def generate_cover(album, output_path):
    size = (600, 600)
    img = Image.new("RGB", size, album["colors"][0])
    draw = ImageDraw.Draw(img)

    c0, c1, c2 = album["colors"]
    for y in range(size[1]):
        t = y / size[1]
        if t < 0.5:
            factor = t * 2.0
            r = int(c0[0] + (c1[0] - c0[0]) * factor)
            g = int(c0[1] + (c1[1] - c0[1]) * factor)
            b = int(c0[2] + (c1[2] - c0[2]) * factor)
        else:
            factor = (t - 0.5) * 2.0
            r = int(c1[0] + (c2[0] - c1[0]) * factor)
            g = int(c1[1] + (c2[1] - c1[1]) * factor)
            b = int(c1[2] + (c2[2] - c1[2]) * factor)
        draw.line([(0, y), (size[0], y)], fill=(r, g, b))

    # Artistic geometric and modern border accents
    draw.ellipse([70, 70, 530, 530], outline=(255, 255, 255, 60), width=2)
    draw.ellipse([130, 130, 470, 470], outline=(c2[0], c2[1], c2[2], 100), width=2)
    draw.line([(50, 485), (550, 485)], fill=(255, 255, 255, 190), width=2)

    try:
        font_large = ImageFont.truetype("arial.ttf", 34)
        font_small = ImageFont.truetype("arial.ttf", 22)
        font_genre = ImageFont.truetype("arial.ttf", 18)
    except:
        font_large = ImageFont.load_default()
        font_small = font_large
        font_genre = font_large

    draw.text((55, 500), album["title"].upper(), fill=(255, 255, 255), font=font_large)
    draw.text((55, 545), f"{album['artist']} • {album['year']}", fill=(225, 225, 225), font=font_small)
    draw.text((55, 45), album["genre"].upper(), fill=c2, font=font_genre)

    img.save(output_path, "JPEG", quality=95)
    return img

def encode_single_track(track_title, track_num, total_tracks, album, cover_path, output_mp4):
    dur = 12 + (track_num % 7) # 12s to 18s duration
    base_freq = 220 + (track_num * 37) % 300
    f0, f1, f2, f3 = base_freq, int(base_freq * 1.25), int(base_freq * 1.5), int(base_freq * 2.0)
    
    synth_filter = (
        f"aevalsrc=0.18*sin(2*PI*{f0}*t)+0.14*sin(2*PI*{f1}*t)+"
        f"0.11*sin(2*PI*{f2}*t)+0.08*sin(2*PI*{f3}*t):s=44100:d={dur},"
        f"afade=t=in:ss=0:d=1.0,afade=t=out:st={dur-1.0}:d=1.0"
    )

    cmd = [
        "ffmpeg", "-y",
        "-f", "lavfi", "-i", synth_filter,
        "-loop", "1", "-i", str(cover_path),
        "-c:a", "aac", "-b:a", "256k",
        "-c:v", "libx264", "-tune", "stillimage", "-pix_fmt", "yuv420p", "-shortest",
        "-metadata", f"title={track_title}",
        "-metadata", f"artist={album['artist']}",
        "-metadata", f"album_artist={album['artist']}",
        "-metadata", f"album={album['title']}",
        "-metadata", f"genre={album['genre']}",
        "-metadata", f"date={album['year']}",
        "-metadata", f"track={track_num}/{total_tracks}",
        str(output_mp4)
    ]
    
    res = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    if res.returncode != 0:
        cmd_fallback = [
            "ffmpeg", "-y",
            "-f", "lavfi", "-i", synth_filter,
            "-i", str(cover_path),
            "-c:a", "aac", "-b:a", "256k",
            "-metadata", f"title={track_title}",
            "-metadata", f"artist={album['artist']}",
            "-metadata", f"album={album['title']}",
            str(output_mp4)
        ]
        subprocess.run(cmd_fallback, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=True)

    file_stat = output_mp4.stat()
    with open(output_mp4, "rb") as f:
        content_hash = hashlib.blake2b(f.read(65536), digest_size=16).hexdigest()

    return {
        "title": track_title,
        "artist": album["artist"],
        "album": album["title"],
        "genre": album["genre"],
        "year": album["year"],
        "track_num": track_num,
        "dur_ms": dur * 1000,
        "file_size": file_stat.st_size,
        "mtime": int(file_stat.st_mtime),
        "path": str(output_mp4),
        "content_hash": content_hash,
        "track_key": f"local_{content_hash}"
    }

def init_database(db_path):
    conn = sqlite3.connect(db_path)
    cur = conn.cursor()
    cur.execute("PRAGMA journal_mode = WAL;")
    cur.execute("PRAGMA synchronous = NORMAL;")
    cur.execute("PRAGMA foreign_keys = ON;")

    cur.execute("""
    CREATE TABLE IF NOT EXISTS tracks (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        track_key TEXT UNIQUE NOT NULL,
        path TEXT NOT NULL,
        title TEXT NOT NULL,
        artist TEXT NOT NULL,
        album TEXT NOT NULL,
        album_artist TEXT,
        composer TEXT,
        genre TEXT,
        year INTEGER,
        track_number INTEGER,
        disc_number INTEGER,
        duration_ms INTEGER NOT NULL,
        file_size INTEGER NOT NULL,
        mtime INTEGER NOT NULL,
        content_hash TEXT NOT NULL,
        isrc TEXT,
        has_artwork INTEGER NOT NULL DEFAULT 0
    );
    """)

    cur.execute("""
    CREATE TABLE IF NOT EXISTS albums (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        title TEXT NOT NULL,
        artist TEXT NOT NULL,
        year INTEGER,
        track_count INTEGER NOT NULL DEFAULT 0,
        artwork_hash TEXT
    );
    """)

    cur.execute("""
    CREATE TABLE IF NOT EXISTS artists (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        track_count INTEGER NOT NULL DEFAULT 0,
        album_count INTEGER NOT NULL DEFAULT 0
    );
    """)

    cur.execute("""
    CREATE TABLE IF NOT EXISTS playlists (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL
    );
    """)

    cur.execute("""
    CREATE TABLE IF NOT EXISTS playlist_tracks (
        playlist_id INTEGER NOT NULL,
        track_id INTEGER NOT NULL,
        position INTEGER NOT NULL,
        PRIMARY KEY (playlist_id, track_id),
        FOREIGN KEY(playlist_id) REFERENCES playlists(id) ON DELETE CASCADE,
        FOREIGN KEY(track_id) REFERENCES tracks(id) ON DELETE CASCADE
    );
    """)

    cur.execute("""
    CREATE TABLE IF NOT EXISTS app_state (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL
    );
    """)

    # FTS5 Virtual Table
    cur.execute("""
    CREATE VIRTUAL TABLE IF NOT EXISTS tracks_fts USING fts5(
        title, artist, album, content=tracks, content_rowid=id
    );
    """)

    cur.execute("""
    CREATE TRIGGER IF NOT EXISTS tracks_ai AFTER INSERT ON tracks BEGIN
        INSERT INTO tracks_fts(rowid, title, artist, album) 
        VALUES (new.id, new.title, new.artist, new.album);
    END;
    """)

    conn.commit()
    return conn

def main():
    print("=== Generating 100+ Songs across 12 Albums ===")
    conn = init_database(DB_PATH)
    cur = conn.cursor()

    # Clear old records
    cur.execute("DELETE FROM playlist_tracks;")
    cur.execute("DELETE FROM playlists;")
    cur.execute("DELETE FROM tracks;")
    cur.execute("DELETE FROM albums;")
    cur.execute("DELETE FROM artists;")
    cur.execute("DELETE FROM tracks_fts;")
    conn.commit()

    total_songs = sum(len(a["tracks"]) for a in ALBUMS)
    print(f"Total albums to build: {len(ALBUMS)} | Total tracks to encode: {total_songs}")

    tasks = []
    album_art_hashes = {}

    for album in ALBUMS:
        album_dir = MUSIC_DIR / album["artist"] / album["title"]
        album_dir.mkdir(parents=True, exist_ok=True)

        cover_path = album_dir / "cover.jpg"
        img = generate_cover(album, cover_path)

        with open(cover_path, "rb") as f:
            art_hash = hashlib.blake2b(f.read(), digest_size=16).hexdigest()
        album_art_hashes[album["title"]] = art_hash

        # Cache thumbnails
        img_64 = img.resize((64, 64), Image.Resampling.LANCZOS)
        img_64.save(CACHE_ARTWORK_DIR / f"{art_hash}_64.jpg", "JPEG", quality=90)
        img_256 = img.resize((256, 256), Image.Resampling.LANCZOS)
        img_256.save(CACHE_ARTWORK_DIR / f"{art_hash}_256.jpg", "JPEG", quality=92)
        img.save(ARTWORK_DIR / f"{album['id']}.jpg", "JPEG", quality=95)

        for num, track_title in enumerate(album["tracks"], 1):
            track_filename = f"{num:02d} - {track_title}.mp4"
            track_path = album_dir / track_filename
            tasks.append((track_title, num, len(album["tracks"]), album, cover_path, track_path))

    print(f"Parallel encoding {len(tasks)} .mp4 tracks using 8 threads...")
    completed_tracks = []
    with ThreadPoolExecutor(max_workers=8) as executor:
        futures = {executor.submit(encode_single_track, *t): t[0] for t in tasks}
        for future in as_completed(futures):
            title = futures[future]
            try:
                data = future.result()
                completed_tracks.append(data)
                print(f"  [OK] Encoded: {data['title']} ({data['artist']})")
            except Exception as e:
                print(f"  [ERROR] Failed to encode {title}: {e}")

    # Sort tracks deterministically by album and track_num
    completed_tracks.sort(key=lambda t: (t["album"], t["track_num"]))

    print(f"Inserting {len(completed_tracks)} tracks and {len(ALBUMS)} albums into SQLite...")

    # Insert Albums
    for album in ALBUMS:
        art_hash = album_art_hashes[album["title"]]
        cur.execute(
            "INSERT INTO albums (title, artist, year, track_count, artwork_hash) VALUES (?, ?, ?, ?, ?)",
            (album["title"], album["artist"], album["year"], len(album["tracks"]), art_hash)
        )

    # Insert Artists
    for album in ALBUMS:
        cur.execute(
            "INSERT INTO artists (name, track_count, album_count) VALUES (?, ?, ?)",
            (album["artist"], len(album["tracks"]), 1)
        )

    # Insert Tracks
    db_track_ids = []
    for t in completed_tracks:
        art_hash = album_art_hashes[t["album"]]
        cur.execute("""
        INSERT INTO tracks (
            track_key, path, title, artist, album, album_artist, genre, year,
            track_number, disc_number, duration_ms, file_size, mtime,
            content_hash, has_artwork
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """, (
            t["track_key"], t["path"], t["title"], t["artist"],
            t["album"], t["artist"], t["genre"], t["year"],
            t["track_num"], 1, t["dur_ms"], t["file_size"], t["mtime"],
            art_hash, 1
        ))
        db_track_ids.append((cur.lastrowid, t["title"], t["album"]))

    # Insert 4 Playlists
    playlists = [
        ("Favorites", db_track_ids[0:15]),
        ("Night Driving", db_track_ids[15:35]),
        ("Focus Beats", db_track_ids[35:55]),
        ("Chill Morning", db_track_ids[55:75]),
    ]

    for pl_name, tracks in playlists:
        cur.execute("INSERT INTO playlists (name, created_at, updated_at) VALUES (?, 1700000000, 1700000000)", (pl_name,))
        pl_id = cur.lastrowid
        for pos, (tid, _, _) in enumerate(tracks):
            cur.execute("INSERT INTO playlist_tracks (playlist_id, track_id, position) VALUES (?, ?, ?)", (pl_id, tid, pos))

    conn.commit()
    conn.close()

    print(f"=== Successfully Generated {len(completed_tracks)} Songs across {len(ALBUMS)} Albums! ===")

if __name__ == "__main__":
    main()
