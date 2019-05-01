package tech.yamba.management.web;

import org.springframework.messaging.handler.annotation.MessageMapping;
import org.springframework.messaging.handler.annotation.SendTo;
import org.springframework.stereotype.Controller;

import tech.yamba.db.jooq.tables.pojos.User;


@Controller
@MessageMapping("/user")
public class UserController {

	@MessageMapping("/hello")
	@SendTo("/user/add")
	public User add(User user) throws Exception {
		Thread.sleep(1000); // simulated delay
		System.out.println(user);
		return user;
	}
}
