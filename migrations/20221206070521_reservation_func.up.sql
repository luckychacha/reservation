CREATE OR REPLACE FUNCTION rsvp.query(
    uid text,
    rid text,
    _start timestamp with time zone,
    _end timestamp with time zone,
    status rsvp.reservation_status DEFAULT 'pending',
    is_desc bool DEFAULT FALSE
) RETURNS TABLE (LIKE rsvp.reservation) AS $$
DECLARE
    _during tstzrange;
    _sql text;
BEGIN
    -- if start or end is null, use infinity
    _during := tstzrange(
        COALESCE(_start, '-infinity'),
        COALESCE(_end, 'infinity'),
        '[)]'
    );
    -- format the query based on parameters
    -- quote_literal 可以防注入，让输入的字符串按照字符串做处理（转义）
    _sql := format(
        'SELECT * FROM rsvp.reservation WHERE %L @> timespan AND status = %L AND %s ORDER BY lower(timespan) %s',
        _during,
        status,
        CASE
            WHEN uid IS NULL AND rid IS NULL THEN 'TRUE'
            WHEN uid IS NULL THEN 'resource_id = ' || quote_literal(rid)
            WHEN rid IS NULL THEN 'user_id = ' || quote_literal(uid)
            ELSE 'user_id = ' || quote_literal(uid) || ' AND resource_id = ' || quote_literal(rid)
        END,
        CASE
            WHEN is_desc THEN 'DESC'
            ELSE 'ASC'
        END
    );
    -- log the sql 暂时保留、未来可以删除
    RAISE NOTICE '%', _sql;

    -- execute
    RETURN QUERY EXECUTE _sql;
END;
$$ LANGUAGE plpgsql;

-- we filter 2 more items one for starting, one for ending
-- If starting existing, then we have previous page,
-- If ending existing, then we have next page.
CREATE OR REPLACE FUNCTION rsvp.filter(
    uid text,
    rid text,
    status rsvp.reservation_status,
    cursor bigint DEFAULT NULL,
    is_desc bool DEFAULT FALSE,
    page_size bigint DEFAULT 10
) RETURNS TABLE (LIKE rsvp.reservation) AS $$
DECLARE
    _sql text;
    _offset bigint;
BEGIN
    -- if page_size is not between 10 and 100, set it to 10
    IF page_size < 10 OR page_size > 100 THEN
        page_size := 10;
    END IF;
    -- if cursor is NULL or less than 0, set it to 0 if is_desc is false, or to 2^63 -1 if is_desc is true
    IF cursor IS NULL OR cursor < 0 THEN
        IF is_desc THEN
            cursor := 9223372036854775807;
        ELSE
            cursor := 0;
        END IF;
    END IF;
    _sql := format(
        'SELECT * FROM rsvp.reservation WHERE %s AND status = %L AND %s ORDER BY id %s LIMIT %L::integer',
        CASE
            WHEN is_desc THEN 'id <= ' || cursor
            ELSE 'id >= ' || cursor
        END,
        status,
        CASE
            WHEN uid IS NULL AND rid IS NULL THEN 'TRUE'
            WHEN uid IS NULL THEN 'resource_id = ' || quote_literal(rid)
            WHEN rid IS NULL THEN 'user_id = ' || quote_literal(uid)
            ELSE 'user_id = ' || quote_literal(uid) || ' AND resource_id = ' || quote_literal(rid)
        END,
        CASE
            WHEN is_desc THEN 'DESC'
            ELSE 'ASC'
        END,
        page_size + 1
    );
    -- log the sql 暂时保留、未来可以删除
    RAISE NOTICE '%', _sql;

    -- execute
    RETURN QUERY EXECUTE _sql;
END;
$$ LANGUAGE plpgsql;
