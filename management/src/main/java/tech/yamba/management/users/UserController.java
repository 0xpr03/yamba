package tech.yamba.management.users;

import java.util.List;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.http.HttpStatus;
import org.springframework.http.MediaType;
import org.springframework.http.ResponseEntity;
import org.springframework.stereotype.Controller;
import org.springframework.web.bind.annotation.DeleteMapping;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.PutMapping;
import org.springframework.web.bind.annotation.RequestMapping;

import tech.yamba.db.jooq.tables.pojos.User;


@Controller
@RequestMapping(value = "/api/user", produces = MediaType.APPLICATION_JSON_VALUE)
public class UserController {

	private final UserService userService;


	@Autowired
	public UserController(UserService userService) {
		this.userService = userService;
	}


	@PostMapping("/add")
	public ResponseEntity<User> addUser(User user) {
		return new ResponseEntity<>(userService.addUser(user), HttpStatus.CREATED);
	}


	@GetMapping("/get")
	public ResponseEntity<List<User>> getUser() {
		return new ResponseEntity<>(userService.getUsers(), HttpStatus.OK);
	}


	@PutMapping("/update")
	public ResponseEntity<User> updateUser(User user) {
		return new ResponseEntity<>(userService.updateUser(user), HttpStatus.OK);
	}


	@DeleteMapping("/delete")
	public ResponseEntity deleteUser(User user) {
		userService.deleteUser(user.getId());
		return new ResponseEntity(HttpStatus.NO_CONTENT);
	}
}
