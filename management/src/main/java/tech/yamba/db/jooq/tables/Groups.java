/*
 * This file is generated by jOOQ.
 */
package tech.yamba.db.jooq.tables;


import java.util.Arrays;
import java.util.List;

import javax.annotation.Generated;

import org.jooq.Field;
import org.jooq.ForeignKey;
import org.jooq.Identity;
import org.jooq.Index;
import org.jooq.Name;
import org.jooq.Record;
import org.jooq.Schema;
import org.jooq.Table;
import org.jooq.TableField;
import org.jooq.UniqueKey;
import org.jooq.impl.DSL;
import org.jooq.impl.TableImpl;

import tech.yamba.db.jooq.Indexes;
import tech.yamba.db.jooq.Keys;
import tech.yamba.db.jooq.Public;
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
public class Groups extends TableImpl<GroupsRecord> {

    private static final long serialVersionUID = -543291348;

    /**
     * The reference instance of <code>public.groups</code>
     */
    public static final Groups GROUPS = new Groups();

    /**
     * The class holding records for this type
     */
    @Override
    public Class<GroupsRecord> getRecordType() {
        return GroupsRecord.class;
    }

    /**
     * The column <code>public.groups.id</code>.
     */
    public final TableField<GroupsRecord, Short> ID = createField("id", org.jooq.impl.SQLDataType.SMALLINT.nullable(false).defaultValue(org.jooq.impl.DSL.field("nextval('groups_id_seq'::regclass)", org.jooq.impl.SQLDataType.SMALLINT)), this, "");

    /**
     * The column <code>public.groups.name</code>.
     */
    public final TableField<GroupsRecord, String> NAME = createField("name", org.jooq.impl.SQLDataType.VARCHAR(255), this, "");

    /**
     * Create a <code>public.groups</code> table reference
     */
    public Groups() {
        this(DSL.name("groups"), null);
    }

    /**
     * Create an aliased <code>public.groups</code> table reference
     */
    public Groups(String alias) {
        this(DSL.name(alias), GROUPS);
    }

    /**
     * Create an aliased <code>public.groups</code> table reference
     */
    public Groups(Name alias) {
        this(alias, GROUPS);
    }

    private Groups(Name alias, Table<GroupsRecord> aliased) {
        this(alias, aliased, null);
    }

    private Groups(Name alias, Table<GroupsRecord> aliased, Field<?>[] parameters) {
        super(alias, null, aliased, parameters, DSL.comment(""));
    }

    public <O extends Record> Groups(Table<O> child, ForeignKey<O, GroupsRecord> key) {
        super(child, key, GROUPS);
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public Schema getSchema() {
        return Public.PUBLIC;
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public List<Index> getIndexes() {
        return Arrays.<Index>asList(Indexes.GROUPS_PKEY);
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public Identity<GroupsRecord, Short> getIdentity() {
        return Keys.IDENTITY_GROUPS;
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public UniqueKey<GroupsRecord> getPrimaryKey() {
        return Keys.GROUPS_PKEY;
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public List<UniqueKey<GroupsRecord>> getKeys() {
        return Arrays.<UniqueKey<GroupsRecord>>asList(Keys.GROUPS_PKEY);
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public Groups as(String alias) {
        return new Groups(DSL.name(alias), this);
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public Groups as(Name alias) {
        return new Groups(alias, this);
    }

    /**
     * Rename this table
     */
    @Override
    public Groups rename(String name) {
        return new Groups(DSL.name(name), null);
    }

    /**
     * Rename this table
     */
    @Override
    public Groups rename(Name name) {
        return new Groups(name, null);
    }
}
