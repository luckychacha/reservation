DROP TRIGGER IF EXISTS reservation_trigger ON rsvp.reservation;
DROP FUNCTION IF EXISTS rsvp.reservation_trigger();
DROP TABLE IF EXISTS reservation_change CASCADE;
