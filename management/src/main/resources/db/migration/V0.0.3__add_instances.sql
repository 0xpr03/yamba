CREATE TABLE instances
(
  id SERIAL NOT NULL,
  autostart BOOLEAN NOT NULL,
  host VARCHAR(255) NOT NULL,
  port INTEGER,
  identity TEXT,
  cid INTEGER,
  name VARCHAR(30) NOT NULL,
  password TEXT,
  PRIMARY KEY (id)
);

CREATE INDEX autostart_index ON instances USING btree(autostart);