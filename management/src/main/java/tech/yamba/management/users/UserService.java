package tech.yamba.management.users;

import org.jooq.DSLContext;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.security.crypto.password.PasswordEncoder;
import org.springframework.stereotype.Service;
import tech.yamba.db.jooq.tables.daos.UserAuthoritiesDao;
import tech.yamba.db.jooq.tables.daos.UsersDao;
import tech.yamba.db.jooq.tables.pojos.User;
import tech.yamba.db.jooq.tables.pojos.UserAuthority;

import java.util.List;

import static tech.yamba.db.jooq.tables.Users.USERS;

@Service
public class UserService {

    DSLContext create;
    UsersDao usersDao;
    PasswordEncoder passwordEncoder;
    UserAuthoritiesDao userAuthoritiesDao;

    @Autowired
    public UserService(DSLContext create, PasswordEncoder passwordEncoder) {
        this.create = create;
        this.usersDao = new UsersDao(create.configuration());
        this.userAuthoritiesDao = new UserAuthoritiesDao(create.configuration());
        this.passwordEncoder = passwordEncoder;
    }

    public User addUser(User user) {
        User result = create.insertInto(USERS)
                .columns(USERS.USERNAME, USERS.PASSWORD)
                .values(user.getUsername(), passwordEncoder.encode(user.getPassword()))
                .returning(USERS.fields())
                .fetchOne()
                .into(User.class);

        // Add default role to user
        userAuthoritiesDao.insert(new UserAuthority(result.getId(), (short)-1));

        return result;
    }

    public List<User> getUsers() {
        return usersDao.findAll();
    }

    public User updateUser(User user) {
        usersDao.update(user);

        return usersDao.fetchOneById(user.getId());
    }

    public void deleteUser(short id) {
        usersDao.deleteById(id);
    }
}
