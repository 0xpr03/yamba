/*
 * This file is generated by jOOQ.
 */
package tech.yamba.db.jooq.tables.daos;


import java.util.List;

import javax.annotation.Generated;

import org.jooq.Configuration;
import org.jooq.impl.DAOImpl;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Repository;

import tech.yamba.db.jooq.tables.Groups;
import tech.yamba.db.jooq.tables.pojos.Group;
import tech.yamba.db.jooq.tables.records.GroupsRecord;


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
public class GroupsDao extends DAOImpl<GroupsRecord, Group, Short> {

    /**
     * Create a new GroupsDao without any configuration
     */
    public GroupsDao() {
        super(Groups.GROUPS, Group.class);
    }

    /**
     * Create a new GroupsDao with an attached configuration
     */
    @Autowired
    public GroupsDao(Configuration configuration) {
        super(Groups.GROUPS, Group.class, configuration);
    }

    /**
     * {@inheritDoc}
     */
    @Override
    protected Short getId(Group object) {
        return object.getId();
    }

    /**
     * Fetch records that have <code>id IN (values)</code>
     */
    public List<Group> fetchById(Short... values) {
        return fetch(Groups.GROUPS.ID, values);
    }

    /**
     * Fetch a unique record that has <code>id = value</code>
     */
    public Group fetchOneById(Short value) {
        return fetchOne(Groups.GROUPS.ID, value);
    }

    /**
     * Fetch records that have <code>name IN (values)</code>
     */
    public List<Group> fetchByName(String... values) {
        return fetch(Groups.GROUPS.NAME, values);
    }
}