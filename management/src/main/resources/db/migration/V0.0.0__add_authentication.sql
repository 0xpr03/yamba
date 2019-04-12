CREATE TABLE users
(
  id       SMALLSERIAL                         NOT NULL,
  username VARCHAR(255) UNIQUE                 NOT NULL,
  enabled  BOOLEAN   DEFAULT FALSE              NOT NULL,
  password CHAR(60)                            NOT NULL,
  created  TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
  PRIMARY KEY (id)
);

CREATE TABLE groups
(
  id   SMALLSERIAL NOT NULL,
  name VARCHAR(255),
  PRIMARY KEY (id)
);

CREATE TABLE group_members
(
  user_id  SMALLINT NOT NULL,
  group_id SMALLINT NOT NULL,
  PRIMARY KEY (user_id, group_id),
  FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
  FOREIGN KEY (group_id) REFERENCES groups (id) ON DELETE CASCADE
);

CREATE TABLE authorities
(
  id        SMALLSERIAL NOT NULL,
  authority VARCHAR(63) NOT NULL,
  PRIMARY KEY (id)
);

CREATE TABLE user_authorities
(
  user_id      SMALLINT NOT NULL,
  authority_id SMALLINT NOT NULL,
  PRIMARY KEY (user_id, authority_id),
  FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
  FOREIGN KEY (authority_id) REFERENCES authorities (id) ON DELETE CASCADE
);

CREATE TABLE group_authorities
(
  group_id     SMALLINT NOT NULL,
  authority_id SMALLINT NOT NULL,
  PRIMARY KEY (group_id, authority_id),
  FOREIGN KEY (group_id) REFERENCES groups (id) ON DELETE CASCADE,
  FOREIGN KEY (authority_id) REFERENCES authorities (id) ON DELETE CASCADE
);
