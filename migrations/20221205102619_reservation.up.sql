CREATE TYPE rsvp.reservation_status AS ENUM('unknown', 'pending', 'confirmed', 'blocked');

CREATE TABLE rsvp.reservation (
                                  id uuid NOT NULL DEFAULT uuid_generate_v4(),
                                  user_id VARCHAR(64) NOT NULL,
                                  status rsvp.reservation_status NOT NULL DEFAULT 'pending',

                                  resource_id VARCHAR(64) NOT NULL,
                                  timespan TSTZRANGE NOT NULL,

                                  note TEXT,

                                  CONSTRAINT reservation_pkey PRIMARY KEY (id),
                                  CONSTRAINT reservation_conflict EXCLUDE USING gist (resource_id WITH =, timespan WITH &&)
);

CREATE INDEX reservations_resource_id_idx ON rsvp.reservation (resource_id);
CREATE INDEX reservations_user_id_idx ON rsvp.reservation (user_id);
