/*
 * This file is generated by jOOQ.
 */
package tech.yamba.db.jooq.tables.daos;


import java.util.List;

import javax.annotation.Generated;

import org.jooq.Configuration;
import org.jooq.Record2;
import org.jooq.impl.DAOImpl;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Repository;

import tech.yamba.db.jooq.tables.GroupMembers;
import tech.yamba.db.jooq.tables.pojos.GroupMember;
import tech.yamba.db.jooq.tables.records.GroupMembersRecord;


/**
 * This class is generated by jOOQ.
 */
@Generated(
    value = {
        "http://www.jooq.org",
        "jOOQ version:3.11.9"
    },
    comments = "This class is generated by jOOQ"
)
@SuppressWarnings({ "all", "unchecked", "rawtypes" })
@Repository
public class GroupMembersDao extends DAOImpl<GroupMembersRecord, GroupMember, Record2<Short, Short>> {

    /**
     * Create a new GroupMembersDao without any configuration
     */
    public GroupMembersDao() {
        super(GroupMembers.GROUP_MEMBERS, GroupMember.class);
    }

    /**
     * Create a new GroupMembersDao with an attached configuration
     */
    @Autowired
    public GroupMembersDao(Configuration configuration) {
        super(GroupMembers.GROUP_MEMBERS, GroupMember.class, configuration);
    }

    /**
     * {@inheritDoc}
     */
    @Override
    protected Record2<Short, Short> getId(GroupMember object) {
        return compositeKeyRecord(object.getUserId(), object.getGroupId());
    }

    /**
     * Fetch records that have <code>user_id IN (values)</code>
     */
    public List<GroupMember> fetchByUserId(Short... values) {
        return fetch(GroupMembers.GROUP_MEMBERS.USER_ID, values);
    }

    /**
     * Fetch records that have <code>group_id IN (values)</code>
     */
    public List<GroupMember> fetchByGroupId(Short... values) {
        return fetch(GroupMembers.GROUP_MEMBERS.GROUP_ID, values);
    }
}
