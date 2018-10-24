CREATE TABLE `users` (
  `id` char(36) NOT NULL,
  `email` varchar(150) UNIQUE NOT NULL,
  `password` varchar(150) NOT NULL,
  `created` DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
  `modified` DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE `permissions` (
  `user_id` char(36) NOT NULL,
  `privilege` varchar(50) NOT NULL,
  PRIMARY KEY (`user_id`, `privilege`),
  FOREIGN KEY (`user_id`) REFERENCES `users`(`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE `users_not_confirmed` (
  `confirmationToken` char(40) NOT NULL,
  `user_id` char(36) NOT NULL,
  PRIMARY KEY (`confirmationToken`),
  FOREIGN KEY (`user_id`) REFERENCES `users`(`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;