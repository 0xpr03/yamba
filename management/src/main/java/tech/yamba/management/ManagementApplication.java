package tech.yamba.management;

import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;


@SpringBootApplication
public class ManagementApplication {

	static {
		System.getProperties().setProperty("org.jooq.no-logo", "true");
	}

	public static void main(String[] args) {
		SpringApplication.run(ManagementApplication.class, args);
	}

}

