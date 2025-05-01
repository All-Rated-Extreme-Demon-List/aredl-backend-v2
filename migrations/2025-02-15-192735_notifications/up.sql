CREATE TYPE notification_type AS ENUM ('Info', 'Success', 'Failure');

CREATE TABLE notifications (
    id uuid DEFAULT uuid_generate_v4(),
	user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    notification_type notification_type NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	PRIMARY KEY(id)
);
