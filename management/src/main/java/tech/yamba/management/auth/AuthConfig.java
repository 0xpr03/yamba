package tech.yamba.management.auth;

import java.security.SecureRandom;
import java.sql.Timestamp;
import java.time.LocalDateTime;
import java.util.Collections;

import javax.sql.DataSource;

import org.apache.commons.lang3.RandomStringUtils;
import org.jooq.DSLContext;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.context.event.ApplicationReadyEvent;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.context.event.EventListener;
import org.springframework.security.config.annotation.authentication.builders.AuthenticationManagerBuilder;
import org.springframework.security.config.annotation.web.builders.HttpSecurity;
import org.springframework.security.config.annotation.web.configuration.WebSecurityConfigurerAdapter;
import org.springframework.security.crypto.bcrypt.BCryptPasswordEncoder;
import org.springframework.web.cors.CorsConfiguration;
import org.springframework.web.cors.CorsConfigurationSource;
import org.springframework.web.cors.UrlBasedCorsConfigurationSource;

import lombok.extern.slf4j.Slf4j;
import tech.yamba.db.jooq.tables.daos.UserAuthoritiesDao;
import tech.yamba.db.jooq.tables.daos.UsersDao;
import tech.yamba.db.jooq.tables.pojos.User;
import tech.yamba.db.jooq.tables.pojos.UserAuthority;


@Configuration
@Slf4j
public class AuthConfig extends WebSecurityConfigurerAdapter {

	private final DataSource dataSource;
	private final DSLContext create;
	private final UsersDao usersDao;
	private final UserAuthoritiesDao userAuthoritiesDao;


	@Autowired public AuthConfig(DataSource dataSource, DSLContext dslContext) {
		this.dataSource = dataSource;
		this.create = dslContext;
		this.usersDao = new UsersDao(dslContext.configuration());
		this.userAuthoritiesDao = new UserAuthoritiesDao(dslContext.configuration());
	}


	@Override protected void configure(AuthenticationManagerBuilder auth) throws Exception {
		auth
				.jdbcAuthentication()
				.passwordEncoder(new BCryptPasswordEncoder())
				.dataSource(this.dataSource)
				.usersByUsernameQuery("SELECT username, password, enabled "
						+ "FROM users "
						+ "WHERE username = ?")
				.authoritiesByUsernameQuery("SELECT username, authority "
						+ "FROM users, authorities, user_authorities "
						+ "WHERE users.id = user_authorities.user_id "
						+ "AND authorities.id = user_authorities.authority_id "
						+ "AND users.username = ?")
				.groupAuthoritiesByUsername("SELECT groups.id, groups.name, authorities.authority "
						+ "FROM groups, group_authorities, group_members, users, authorities "
						+ "WHERE groups.id = group_authorities.group_id "
						+ "AND authorities.id = group_authorities.authority_id "
						+ "AND users.id = group_members.user_id "
						+ "AND groups.id = group_members.group_id "
						+ "AND users.username = ?");
	}


	@Override
	protected void configure(HttpSecurity http) throws Exception {
		http
				.authorizeRequests()
				.anyRequest()
				.authenticated()
				.and()
				.formLogin()
				//.successForwardUrl("/") // TODO: implement login forward
				.and()
				.logout()
				.logoutSuccessUrl("/login?logout")
				.and()
				.csrf()
				.disable()
				.cors();
	}


	@Bean
	CorsConfigurationSource corsConfigurationSource() {
		UrlBasedCorsConfigurationSource source = new UrlBasedCorsConfigurationSource();
		CorsConfiguration corsConfiguration = new CorsConfiguration().applyPermitDefaultValues();
		corsConfiguration.setAllowedOrigins(Collections.singletonList("http://localhost:3000"));
		corsConfiguration.setAllowCredentials(true);
		source.registerCorsConfiguration("/**", corsConfiguration);
		return source;
	}

	@EventListener(ApplicationReadyEvent.class)
	public void initRootUser() {
		final String rootUsername = "root";
		if (this.usersDao.fetchByUsername(rootUsername).isEmpty()) {
			// No root user, create with random password
			log.info("Creating root user with random password");
			final String uppers = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
			final String lowers = uppers.toLowerCase();
			final String digits = "0123456789";
			final char[] chars = (uppers + lowers + digits).toCharArray();

			String randomPassword = RandomStringUtils.random(20, 0, chars.length, true, true, chars, new SecureRandom());

			this.usersDao.insert(new User(
					(short) 0,
					rootUsername,
					true,
					new BCryptPasswordEncoder().encode(randomPassword),
					Timestamp.valueOf(LocalDateTime.now())
			));

			this.userAuthoritiesDao.insert(new UserAuthority((short) 0, (short) 0));

			log.warn("\nroot user with password '{}' was created\nYou should change the generated password for your own security!", randomPassword);
		}
	}
}
