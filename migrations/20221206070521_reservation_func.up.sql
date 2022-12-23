CREATE OR REPLACE FUNCTION rsvp.query(uid text, rid text, during TSTZRANGE) RETURNS TABLE (LIKE rsvp.reservation) AS $$
BEGIN
    IF uid IS NULL AND rid IS NULL THEN
        RETURN QUERY SELECT * FROM rsvp.reservation WHERE timespan && during;
    ELSIF uid IS NULL THEN
        RETURN QUERY SELECT * FROM rsvp.reservation WHERE resource_id = rid AND during @> timespan;
    ELSEIF rid IS NULL THEN
        RETURN QUERY SELECT * FROM rsvp.reservation WHERE user_id = uid AND during @> timespan;
    ELSE
        RETURN QUERY SELECT * FROM rsvp.reservation WHERE resource_id = rid AND user_id = uid AND during @> timespan;
    END IF;
END;
$$ LANGUAGE plpgsql;
