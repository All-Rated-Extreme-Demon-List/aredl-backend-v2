CREATE TABLE public.user_badge_progress (
    user_id UUID PRIMARY KEY REFERENCES public.users(id) ON DELETE CASCADE,
    statistics JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
