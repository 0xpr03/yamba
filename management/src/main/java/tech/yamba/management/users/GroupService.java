package tech.yamba.management.users;

import org.jooq.DSLContext;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Service;
import tech.yamba.db.jooq.tables.daos.GroupMembersDao;
import tech.yamba.db.jooq.tables.daos.GroupsDao;
import tech.yamba.db.jooq.tables.pojos.Group;
import tech.yamba.db.jooq.tables.pojos.GroupMember;

import java.util.List;

import static tech.yamba.db.jooq.tables.Groups.GROUPS;

@Service
public class GroupService {

    DSLContext create;
    GroupsDao groupsDao;
    GroupMembersDao groupMembersDao;

    @Autowired
    public GroupService(DSLContext create) {
        this.create = create;
        groupsDao = new GroupsDao(create.configuration());
        groupMembersDao = new GroupMembersDao(create. configuration());
    }


    public Group addGroup(Group group) {
        Group result = create.insertInto(GROUPS)
                .columns(GROUPS.NAME)
                .values(group.getName())
                .returning(GROUPS.fields())
                .fetchOne()
                .into(Group.class);

        return result;
    }

    public List<Group> getGroups() {
        return groupsDao.findAll();
    }

    public Group updateGroup(Group Group) {
        groupsDao.update(Group);

        return groupsDao.fetchOneById(Group.getId());
    }

    public void deleteGroup(short id) {
        groupsDao.deleteById(id);
    }

    // Group members
    public void addMember(short user_id, short group_id) {
        groupMembersDao
                .insert(new GroupMember(user_id, group_id));
    }

    public void removeMember(short user_id, short group_id) {
        groupMembersDao
                .delete(new GroupMember(user_id, group_id));
    }
}
