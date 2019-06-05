/*
 * This file is generated by jOOQ.
 */
package tech.yamba.db.jooq.tables.records;


import javax.annotation.Generated;
import javax.validation.constraints.NotNull;
import javax.validation.constraints.Size;

import org.jooq.Field;
import org.jooq.Record1;
import org.jooq.Record8;
import org.jooq.Row8;
import org.jooq.impl.UpdatableRecordImpl;

import tech.yamba.db.jooq.tables.Instances;


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
public class InstancesRecord extends UpdatableRecordImpl<InstancesRecord> implements Record8<Integer, Boolean, String, Integer, String, Integer, String, String> {

    private static final long serialVersionUID = -359188337;

    /**
     * Setter for <code>public.instances.id</code>.
     */
    public InstancesRecord setId(Integer value) {
        set(0, value);
        return this;
    }

    /**
     * Getter for <code>public.instances.id</code>.
     */
    public Integer getId() {
        return (Integer) get(0);
    }

    /**
     * Setter for <code>public.instances.autostart</code>.
     */
    public InstancesRecord setAutostart(Boolean value) {
        set(1, value);
        return this;
    }

    /**
     * Getter for <code>public.instances.autostart</code>.
     */
    @NotNull
    public Boolean getAutostart() {
        return (Boolean) get(1);
    }

    /**
     * Setter for <code>public.instances.host</code>.
     */
    public InstancesRecord setHost(String value) {
        set(2, value);
        return this;
    }

    /**
     * Getter for <code>public.instances.host</code>.
     */
    @NotNull
    @Size(max = 255)
    public String getHost() {
        return (String) get(2);
    }

    /**
     * Setter for <code>public.instances.port</code>.
     */
    public InstancesRecord setPort(Integer value) {
        set(3, value);
        return this;
    }

    /**
     * Getter for <code>public.instances.port</code>.
     */
    public Integer getPort() {
        return (Integer) get(3);
    }

    /**
     * Setter for <code>public.instances.identity</code>.
     */
    public InstancesRecord setIdentity(String value) {
        set(4, value);
        return this;
    }

    /**
     * Getter for <code>public.instances.identity</code>.
     */
    public String getIdentity() {
        return (String) get(4);
    }

    /**
     * Setter for <code>public.instances.cid</code>.
     */
    public InstancesRecord setCid(Integer value) {
        set(5, value);
        return this;
    }

    /**
     * Getter for <code>public.instances.cid</code>.
     */
    public Integer getCid() {
        return (Integer) get(5);
    }

    /**
     * Setter for <code>public.instances.name</code>.
     */
    public InstancesRecord setName(String value) {
        set(6, value);
        return this;
    }

    /**
     * Getter for <code>public.instances.name</code>.
     */
    @NotNull
    @Size(max = 30)
    public String getName() {
        return (String) get(6);
    }

    /**
     * Setter for <code>public.instances.password</code>.
     */
    public InstancesRecord setPassword(String value) {
        set(7, value);
        return this;
    }

    /**
     * Getter for <code>public.instances.password</code>.
     */
    public String getPassword() {
        return (String) get(7);
    }

    // -------------------------------------------------------------------------
    // Primary key information
    // -------------------------------------------------------------------------

    /**
     * {@inheritDoc}
     */
    @Override
    public Record1<Integer> key() {
        return (Record1) super.key();
    }

    // -------------------------------------------------------------------------
    // Record8 type implementation
    // -------------------------------------------------------------------------

    /**
     * {@inheritDoc}
     */
    @Override
    public Row8<Integer, Boolean, String, Integer, String, Integer, String, String> fieldsRow() {
        return (Row8) super.fieldsRow();
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public Row8<Integer, Boolean, String, Integer, String, Integer, String, String> valuesRow() {
        return (Row8) super.valuesRow();
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public Field<Integer> field1() {
        return Instances.INSTANCES.ID;
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public Field<Boolean> field2() {
        return Instances.INSTANCES.AUTOSTART;
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public Field<String> field3() {
        return Instances.INSTANCES.HOST;
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public Field<Integer> field4() {
        return Instances.INSTANCES.PORT;
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public Field<String> field5() {
        return Instances.INSTANCES.IDENTITY;
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public Field<Integer> field6() {
        return Instances.INSTANCES.CID;
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public Field<String> field7() {
        return Instances.INSTANCES.NAME;
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public Field<String> field8() {
        return Instances.INSTANCES.PASSWORD;
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public Integer component1() {
        return getId();
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public Boolean component2() {
        return getAutostart();
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public String component3() {
        return getHost();
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public Integer component4() {
        return getPort();
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public String component5() {
        return getIdentity();
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public Integer component6() {
        return getCid();
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public String component7() {
        return getName();
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public String component8() {
        return getPassword();
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public Integer value1() {
        return getId();
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public Boolean value2() {
        return getAutostart();
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public String value3() {
        return getHost();
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public Integer value4() {
        return getPort();
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public String value5() {
        return getIdentity();
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public Integer value6() {
        return getCid();
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public String value7() {
        return getName();
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public String value8() {
        return getPassword();
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public InstancesRecord value1(Integer value) {
        setId(value);
        return this;
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public InstancesRecord value2(Boolean value) {
        setAutostart(value);
        return this;
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public InstancesRecord value3(String value) {
        setHost(value);
        return this;
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public InstancesRecord value4(Integer value) {
        setPort(value);
        return this;
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public InstancesRecord value5(String value) {
        setIdentity(value);
        return this;
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public InstancesRecord value6(Integer value) {
        setCid(value);
        return this;
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public InstancesRecord value7(String value) {
        setName(value);
        return this;
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public InstancesRecord value8(String value) {
        setPassword(value);
        return this;
    }

    /**
     * {@inheritDoc}
     */
    @Override
    public InstancesRecord values(Integer value1, Boolean value2, String value3, Integer value4, String value5, Integer value6, String value7, String value8) {
        value1(value1);
        value2(value2);
        value3(value3);
        value4(value4);
        value5(value5);
        value6(value6);
        value7(value7);
        value8(value8);
        return this;
    }

    // -------------------------------------------------------------------------
    // Constructors
    // -------------------------------------------------------------------------

    /**
     * Create a detached InstancesRecord
     */
    public InstancesRecord() {
        super(Instances.INSTANCES);
    }

    /**
     * Create a detached, initialised InstancesRecord
     */
    public InstancesRecord(Integer id, Boolean autostart, String host, Integer port, String identity, Integer cid, String name, String password) {
        super(Instances.INSTANCES);

        set(0, id);
        set(1, autostart);
        set(2, host);
        set(3, port);
        set(4, identity);
        set(5, cid);
        set(6, name);
        set(7, password);
    }
}