CREATE TABLE `streams` (
  `id` char(32) NOT NULL,
  `name` varchar(50) NOT NULL,
  `url` varchar(150) NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE `songs` (
  `id` char(32) NOT NULL,
  `name` varchar(150) NOT NULL,
  `source` varchar(150) NOT NULL,
  `artist` varchar(50) NOT NULL,
  `length` varchar(50) NOT NULL,
  `keep` bit NOT NULL,
  `downloaded` bit NOT NULL,
  `last_used` DATETIME NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE `playlists` (
  `id` char(32) NOT NULL,
  `name` varchar(50) NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE `songs2playlists` (
  `song_id` char(32) NOT NULL,
  `playlist_id` char(32) NOT NULL,
  PRIMARY KEY (`song_id`, `playlist_id`),
  FOREIGN KEY (`song_id`) REFERENCES `songs`(`id`),
  FOREIGN KEY (`playlist_id`) REFERENCES `playlists`(`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;