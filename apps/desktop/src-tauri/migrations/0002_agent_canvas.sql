-- Phase 2: canvas positions are now first-class.
--
-- The `agents.position_x` / `position_y` columns existed as nullable reals
-- since 0001 (reserved for Phase 2). SQLite cannot retroactively add a
-- NOT NULL constraint without recreating the table, so we enforce
-- non-nullability at the application layer and backfill any NULLs left
-- over from Phase 1 here.
--
-- Idempotent.

UPDATE agents SET position_x = 0 WHERE position_x IS NULL;
UPDATE agents SET position_y = 0 WHERE position_y IS NULL;
