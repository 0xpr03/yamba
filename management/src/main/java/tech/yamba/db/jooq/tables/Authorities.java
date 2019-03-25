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
import tech.yamba.db.jooq.tables.records.AuthoritiesRecord;


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
public class Authorities extends TableImpl<AuthoritiesRecord> {

    private static final long serialVersionUID = -2143414064;

    /**
     * The reference instance of <code>public.authorities</code>
     */
    public static final Authorities AUTHORITIES = new Authorities();

    /**
     * The class holding records for this type
     */
    @Override
    public Class<AuthoritiesRecord> getRecordType() {
        return AuthoritiesRecord.class;
    }

    /**
     * The column <code>public.authorities.id</code>.
     */
    public final TableField<AuthoritiesRecord, Short> ID = createField("id", org.jooq.impl.SQLDataType.SMALLINT.nullable(false).defaultValue(org.jooq.impl.DSL.field("nextval('authorities_id_seq'::regclass)", org.jooq.impl.SQLDataType.SMALLINT)), this, "");

    /**
     * The column <code>public.authorities.authority</code>.
     */
    public final TableField<AuthoritiesRecord, String> AUTHORITY = createField("authority", org.jooq.impl.SQLDataType.VARCHAR(63).nullable(false), this, "");

    /**
     * Create a <code>public.authorities</code> table reference
     */
    public Authorities() {
        this(DSL.name("authorities"), null);
    }

    /**
     * Create an aliased <code>public.authorities</code> table reference
     */
    public Authorities(String alias) {
        this(DSL.name(alias), AUTHORITIES);
    }

    /**
     * Create an aliased <code>public.authorities</code> table reference
     */
    public Authorities(Name alias) {
        this(alias, AUTHORITIES);
    }

    private Authorities(Name alias, Table<AuthoritiesRecord> aliased) {
        this(alias, aliased, null);
    }

    private Authorities(Name alias, Table<AuthoritiesRecord> aliased, Field<?>[] parameters) {
        super(alias, null, aliased, parameters, DSL.comment(""));
    }

    public <O extends Record> Authorities(Table<O> child, ForeignKey<O, AuthoritiesRecord> key) {
        super(child, key, AUTHORITIES);
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
        return Arrays.<Index>asList(Indexes.AUTHORITIES_PKEY);
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public Identity<AuthoritiesRecord, Short> getIdentity() {
        return Keys.IDENTITY_AUTHORITIES;
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public UniqueKey<AuthoritiesRecord> getPrimaryKey() {
        return Keys.AUTHORITIES_PKEY;
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public List<UniqueKey<AuthoritiesRecord>> getKeys() {
        return Arrays.<UniqueKey<AuthoritiesRecord>>asList(Keys.AUTHORITIES_PKEY);
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public Authorities as(String alias) {
        return new Authorities(DSL.name(alias), this);
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public Authorities as(Name alias) {
        return new Authorities(alias, this);
    }

    /**
     * Rename this table
     */
    @Override
    public Authorities rename(String name) {
        return new Authorities(DSL.name(name), null);
    }

    /**
     * Rename this table
     */
    @Override
    public Authorities rename(Name name) {
        return new Authorities(name, null);
    }
}
