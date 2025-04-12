-- Up
CREATE TABLE animes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    anilist_id INTEGER UNIQUE NOT NULL,
    media_type TEXT NOT NULL,
    titles TEXT NOT NULL, -- JSON array
    year INTEGER,
    season TEXT,
    start_date TEXT,
    episode_count INTEGER,
    season_number INTEGER,
    episode_number INTEGER,
    absolute_episode_number INTEGER,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE anime_mappings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    anime_id INTEGER NOT NULL,
    platform TEXT NOT NULL,
    platform_id TEXT,
    review_status TEXT NOT NULL DEFAULT 'Unmapped', -- Unmapped, Ready, Approved, Rejected
    review_comment TEXT,
    reviewed_by TEXT,
    reviewed_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (anime_id) REFERENCES animes(id) ON DELETE CASCADE,
    UNIQUE(anime_id, platform)
);

CREATE TABLE review_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    anime_id INTEGER NOT NULL,
    mapping_id INTEGER NOT NULL,
    platform TEXT NOT NULL, 
    previous_status TEXT NOT NULL,
    new_status TEXT NOT NULL,
    comment TEXT,
    reviewed_by TEXT NOT NULL,
    reviewed_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (anime_id) REFERENCES animes(id) ON DELETE CASCADE,
    FOREIGN KEY (mapping_id) REFERENCES anime_mappings(id) ON DELETE CASCADE
);

CREATE INDEX idx_animes_anilist_id ON animes(anilist_id);
CREATE INDEX idx_anime_mappings_anime_id ON anime_mappings(anime_id);
CREATE INDEX idx_anime_mappings_platform_id ON anime_mappings(platform_id);
CREATE INDEX idx_anime_mappings_status ON anime_mappings(review_status);
CREATE INDEX idx_review_history_mapping_id ON review_history(mapping_id);

-- Down
DROP TABLE review_history;
DROP TABLE anime_mappings;
DROP TABLE animes; 