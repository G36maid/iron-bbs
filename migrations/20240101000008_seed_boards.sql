INSERT INTO boards (name, slug, description) VALUES
('General', 'general', 'General discussions and announcements'),
('Technology', 'tech', 'Technology news and discussions'),
('Programming', 'programming', 'Programming languages, frameworks, and tools'),
('Off-topic', 'off-topic', 'Random discussions and off-topic conversations');

UPDATE posts SET board_id = (SELECT id FROM boards WHERE slug = 'general' LIMIT 1) WHERE board_id IS NULL;
