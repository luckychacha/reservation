-- Add up migration script here
-- ALTER TABLE rsvp.reservation ALTER COLUMN id TYPE integer;

-- CREATE SEQUENCE rsvp.reservation_id_seq
--     AS integer
--     START WITH 1
--     INCREMENT BY 1
--     NO MINVALUE
--     NO MAXVALUE
--     CACHE 1;

-- ALTER SEQUENCE rsvp.reservation_id_seq OWNED BY rsvp.reservation.id;
