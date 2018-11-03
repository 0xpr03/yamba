-- This file is part of yamba which is released under <GPL3>. See file LICENSE or go to https://www.gnu.org/licenses/gpl.html for full license details.

CREATE TABLE `users` (
  `id` char(36) NOT NULL,
  `email` varchar(150) UNIQUE NOT NULL,
  `password` varchar(150) NOT NULL,
  `created` DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
  `modified` DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE `users_not_confirmed` (
  `confirmationToken` char(40) NOT NULL,
  `user_id` char(36) NOT NULL,
  PRIMARY KEY (`confirmationToken`),
  FOREIGN KEY (`user_id`) REFERENCES `users`(`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE `add_songs_jobs` (
  `backend_token` INT UNSIGNED NOT NULL,
  `playlist_id` char(36) NOT NULL,
  `user_id` char(36) NOT NULL,
  PRIMARY KEY (`backend_token`),
  FOREIGN KEY (`playlist_id`) REFERENCES `playlists`(`id`),
  FOREIGN KEY (`user_id`) REFERENCES `users`(`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;