-- This file is part of yamba which is released under <GPL3>. See file LICENSE or go to https://www.gnu.org/licenses/gpl.html for full license details.

CREATE TABLE `permissions` (
  `id` char(36) NOT NULL,
  `permission` varchar(50) NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE `permission_groups` (
  `id` char(36) NOT NULL,
  `name` varchar(50) NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE `permissions_to_permission_groups` (
  `permission_id` char(36) NOT NULL,
  `permission_group_id` char(36) NOT NULL,
  PRIMARY KEY (`permission_id`, `permission_group_id`),
  FOREIGN KEY (`permission_id`) REFERENCES `permissions`(`id`),
  FOREIGN KEY (`permission_group_id`) REFERENCES `permission_groups`(`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE `users_to_permission_groups` (
  `user_id` char(36) NOT NULL,
  `permission_group_id` char(36) NOT NULL,
  PRIMARY KEY (`user_id`, `permission_group_id`),
  FOREIGN KEY (`user_id`) REFERENCES `users`(`id`),
  FOREIGN KEY (`permission_group_id`) REFERENCES `permission_groups`(`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE `ts3_groups` (
  `id` char(32) NOT NULL,
  `name` varchar(50) NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE `ts3_groups_to_permission_groups` (
  `ts3_group_id` char(32) NOT NULL,
  `permission_group_id` char(36) NOT NULL,
  PRIMARY KEY (`ts3_group_id`, `permission_group_id`),
  FOREIGN KEY (`ts3_group_id`) REFERENCES `ts3_groups`(`id`),
  FOREIGN KEY (`permission_group_id`) REFERENCES `permission_groups`(`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;