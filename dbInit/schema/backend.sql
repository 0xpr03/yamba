-- This file is part of yamba which is released under <GPL3>. See file LICENSE or go to https://www.gnu.org/licenses/gpl.html for full license details.

CREATE TABLE `titles` (
  `id` CHAR(32) NOT NULL,
  `name` VARCHAR(250) NOT NULL,
  `source` VARCHAR(255) NOT NULL,
  `last_used` DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
  `downloaded` BIT NOT NULL,
  `keep` BIT DEFAULT 0 NOT NULL,
  `artist` VARCHAR(50),
  `length` INT,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE `playlists` (
  `id` CHAR(36) NOT NULL,
  `name` VARCHAR(50) NOT NULL,
  `created` DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
  `modified` DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE `titles_to_playlists` (
  `title_id` CHAR(32) NOT NULL,
  `playlist_id` CHAR(36) NOT NULL,
  PRIMARY KEY (`title_id`, `playlist_id`),
  FOREIGN KEY (`title_id`) REFERENCES `titles`(`id`),
  FOREIGN KEY (`playlist_id`) REFERENCES `playlists`(`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE `instances` (
  `id` INT AUTO_INCREMENT NOT NULL,
  `host` VARCHAR(255) NOT NULL,
  `port` INT(16) UNSIGNED,
  `identity` VARCHAR(255) NOT NULL,
  `name` VARCHAR(255) NOT NULL,
  `password` VARCHAR (255),
  `autostart` BIT NOT NULL,
  `cid` INT(32),
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE `instance_store` (
  `id` INT NOT NULL PRIMARY KEY,
  `volume` INT NOT NULL,
  `index` INT NOT NULL,
  `position` INT NOT NULL,
  `random` BIT NOT NULL,
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE `queues` (
  `index` INT AUTO_INCREMENT NOT NULL,
  `instance_id` INT NOT NULL,
  `title_id` CHAR(32) NOT NULL,
  PRIMARY KEY (`index`, `instance_id`),
  FOREIGN KEY (`instance_id`) REFERENCES `instances`(`id`),
  FOREIGN KEY (`title_id`) REFERENCES `titles`(`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
