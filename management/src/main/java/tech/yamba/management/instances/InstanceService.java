package tech.yamba.management.instances;

import static tech.yamba.db.jooq.tables.Instances.INSTANCES;
import static tech.yamba.db.jooq.tables.Users.USERS;

import java.util.List;

import org.jooq.DSLContext;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.security.crypto.password.PasswordEncoder;
import org.springframework.stereotype.Service;

import tech.yamba.db.jooq.tables.Instances;
import tech.yamba.db.jooq.tables.daos.InstancesDao;
import tech.yamba.db.jooq.tables.daos.UserAuthoritiesDao;
import tech.yamba.db.jooq.tables.daos.UsersDao;
import tech.yamba.db.jooq.tables.pojos.Instance;
import tech.yamba.db.jooq.tables.pojos.User;
import tech.yamba.db.jooq.tables.pojos.UserAuthority;

@Service
public class InstanceService {

    private final DSLContext create;
    private final InstancesDao instancesDao;

    @Autowired
    public InstanceService(DSLContext create) {
        this.create = create;
        instancesDao = new InstancesDao(create.configuration());
    }

    public Instance addInstance(Instance instance) {
        Instance result = create.insertInto(INSTANCES)
                .columns(INSTANCES.AUTOSTART, INSTANCES.CID,INSTANCES.HOST,INSTANCES.IDENTITY,INSTANCES.NAME,INSTANCES.PORT,INSTANCES.PASSWORD)
                .values(instance.getAutostart(), instance.getCid(),instance.getHost(),instance.getIdentity(),instance.getName(),instance.getPort(),instance.getPassword())
                .returning(INSTANCES.fields())
                .fetchOne()
                .into(Instance.class);
        return result;
    }

    public List<Instance> getInstances() {
        return instancesDao.findAll();
    }

    public Instance getInstanceById(int id) {
        return instancesDao.fetchOneById(id);
    }

    public Instance updateInstance(Instance instance) {
        instancesDao.update(instance);

        return instancesDao.fetchOneById(instance.getId());
    }

    public void deleteInstance(int id) {
        instancesDao.deleteById(id);
    }
}
