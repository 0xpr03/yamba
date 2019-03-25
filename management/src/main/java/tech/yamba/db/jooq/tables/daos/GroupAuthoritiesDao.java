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

import tech.yamba.db.jooq.tables.GroupAuthorities;
import tech.yamba.db.jooq.tables.pojos.GroupAuthority;
import tech.yamba.db.jooq.tables.records.GroupAuthoritiesRecord;


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
public class GroupAuthoritiesDao extends DAOImpl<GroupAuthoritiesRecord, GroupAuthority, Record2<Short, Short>> {

    /**
     * Create a new GroupAuthoritiesDao without any configuration
     */
    public GroupAuthoritiesDao() {
        super(GroupAuthorities.GROUP_AUTHORITIES, GroupAuthority.class);
    }

    /**
     * Create a new GroupAuthoritiesDao with an attached configuration
     */
    @Autowired
    public GroupAuthoritiesDao(Configuration configuration) {
        super(GroupAuthorities.GROUP_AUTHORITIES, GroupAuthority.class, configuration);
    }

    /**
     * {@inheritDoc}
     */
    @Override
    protected Record2<Short, Short> getId(GroupAuthority object) {
        return compositeKeyRecord(object.getGroupId(), object.getAuthorityId());
    }

    /**
     * Fetch records that have <code>group_id IN (values)</code>
     */
    public List<GroupAuthority> fetchByGroupId(Short... values) {
        return fetch(GroupAuthorities.GROUP_AUTHORITIES.GROUP_ID, values);
    }

    /**
     * Fetch records that have <code>authority_id IN (values)</code>
     */
    public List<GroupAuthority> fetchByAuthorityId(Short... values) {
        return fetch(GroupAuthorities.GROUP_AUTHORITIES.AUTHORITY_ID, values);
    }
}
