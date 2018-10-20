CREATE TABLE `users` (
  `id` char(36) NOT NULL,
  `email` varchar(255) UNIQUE NOT NULL,
  `password` varchar(255) NOT NULL,
  `created` DATETIME NOT NULL,
  `modified` DATETIME NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE `permissions` (
  `user_id` char(36) NOT NULL,
  `privilege` varchar(50) NOT NULL,
  PRIMARY KEY (`user_id`, `privilege`),
  FOREIGN KEY (`user_id`) REFERENCES `users`(`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE `streams` (
  `id` char(32) NOT NULL,
  `name` varchar(50) NOT NULL,
  `url` varchar(255) NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE `songs` (
  `id` char(32) NOT NULL,
  `name` varchar(255) NOT NULL,
  `url` varchar(255) NOT NULL,
  `artist` varchar(50) NOT NULL,
  `length` varchar(50) NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE `playlists` (
  `id` char(36) NOT NULL,
  `name` varchar(50) NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE `songs2playlists` (
  `song_id` char(36) NOT NULL,
  `playlist_id` char(36) NOT NULL,
  PRIMARY KEY (`song_id`, `playlist_id`),
  FOREIGN KEY (`song_id`) REFERENCES `songs`(`id`),
  FOREIGN KEY (`playlist_id`) REFERENCES `playlists`(`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;