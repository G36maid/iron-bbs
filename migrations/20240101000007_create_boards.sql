CREATE TABLE IF NOT EXISTS boards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    slug VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

CREATE INDEX idx_boards_slug ON boards(slug);

ALTER TABLE posts ADD COLUMN board_id UUID REFERENCES boards(id) ON DELETE SET NULL;
CREATE INDEX idx_posts_board_id ON posts(board_id);
