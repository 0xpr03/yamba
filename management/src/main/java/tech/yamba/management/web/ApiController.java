package tech.yamba.management.web;

import org.springframework.http.HttpStatus;
import org.springframework.http.MediaType;
import org.springframework.http.ResponseEntity;
import org.springframework.stereotype.Controller;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RequestMapping;


@Controller
@RequestMapping("/api/")
public class ApiController {

	@GetMapping(value = "hello", produces = MediaType.TEXT_PLAIN_VALUE)
	public ResponseEntity<String> hello() {
		return new ResponseEntity<>("hello", HttpStatus.OK);
	}
}
