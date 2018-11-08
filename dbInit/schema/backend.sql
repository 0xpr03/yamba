-- This file is part of yamba which is released under <GPL3>. See file LICENSE or go to https://www.gnu.org/licenses/gpl.html for full license details.

CREATE TABLE `titles` (
  `id` char(32) NOT NULL,
  `name` varchar(150) NOT NULL,
  `source` varchar(150) NOT NULL,
  `last_used` DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
  `artist` varchar(50),
  `length` INT,
  `downloaded` bit,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE `playlists` (
  `id` char(36) NOT NULL,
  `name` varchar(50) NOT NULL,
  `keep` bit DEFAULT 0 NOT NULL,
  `created` DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
  `modified` DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE `titles_to_playlists` (
  `title_id` char(32) NOT NULL,
  `playlist_id` char(36) NOT NULL,
  PRIMARY KEY (`title_id`, `playlist_id`),
  FOREIGN KEY (`title_id`) REFERENCES `titles`(`id`),
  FOREIGN KEY (`playlist_id`) REFERENCES `playlists`(`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;