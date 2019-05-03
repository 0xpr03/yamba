package tech.yamba.management.users;

import java.util.List;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.http.HttpStatus;
import org.springframework.http.MediaType;
import org.springframework.http.ResponseEntity;
import org.springframework.stereotype.Controller;
import org.springframework.web.bind.annotation.DeleteMapping;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.PutMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;

import lombok.extern.slf4j.Slf4j;
import tech.yamba.db.jooq.tables.pojos.User;
import tech.yamba.management.websocket.WebSocketService;


@Controller
@RequestMapping(value = "/api/user", produces = MediaType.APPLICATION_JSON_VALUE)
@Slf4j
public class UserController {

	private final UserService userService;
	private final WebSocketService webSocketService;


	@Autowired
	public UserController(UserService userService, WebSocketService webSocketService) {
		this.userService = userService;
		this.webSocketService = webSocketService;
	}


	@PostMapping()
	public ResponseEntity<User> addUser(@RequestBody User user) {
		User result = userService.addUser(user);
		notifyGetUser();
		return new ResponseEntity<>(result, HttpStatus.CREATED);
	}


	@GetMapping()
	public ResponseEntity<List<User>> getUsers() {
		return new ResponseEntity<>(userService.getUsers(), HttpStatus.OK);
	}


	@GetMapping("/{id}")
	public ResponseEntity<User> getUserById(@PathVariable short id) {
		return userService.getUsers()
				.stream()
				.filter(user -> user.getId().equals(id))
				.findAny()
				.map(user -> new ResponseEntity<>(user, HttpStatus.OK))
				.orElse(new ResponseEntity<>(HttpStatus.NOT_FOUND));
	}


	@PutMapping("/{id}")
	public ResponseEntity<User> updateUser(@PathVariable short id, @RequestBody User user) {
		User result = userService.updateUser(user.setId(id));
		notifyGetUser();
		return new ResponseEntity<>(result, HttpStatus.OK);
	}


	@DeleteMapping("/{id}")
	public ResponseEntity deleteUser(@PathVariable short id) {
		userService.deleteUser(id);
		notifyGetUser();
		return new ResponseEntity(HttpStatus.NO_CONTENT);
	}


	private void notifyGetUser() {
		webSocketService.notifySubscribers(UserController.class, methodName -> methodName.startsWith("getUsers"));
	}
}
