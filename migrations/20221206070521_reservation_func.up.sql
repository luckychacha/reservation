CREATE OR REPLACE FUNCTION rsvp.query(
    uid text,
    rid text,
    during TSTZRANGE,
    status rsvp.reservation_status,
    page integer DEFAULT 1,
    is_desc bool DEFAULT FALSE,
    page_size integer DEFAULT 10
) RETURNS TABLE (LIKE rsvp.reservation) AS $$
DECLARE
    _sql text;
BEGIN
    -- format the query based on parameters
    -- quote_literal 可以防注入，让输入的字符串按照字符串做处理（转义）
    _sql := format(
        'SELECT * FROM rsvp.reservation WHERE %L @> timespan AND status = %L AND %s ORDER BY lower(timespan) %s LIMIT %L::integer OFFSET %L::integer',
        during,
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
        page_size,
        (page - 1) * page_size
    );
    -- log the sql 暂时保留、未来可以删除
    RAISE NOTICE '%', _sql;

    -- execute
    RETURN QUERY EXECUTE _sql;
END;
$$ LANGUAGE plpgsql;
