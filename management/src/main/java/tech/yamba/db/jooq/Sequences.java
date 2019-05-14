/*
 * This file is generated by jOOQ.
 */
package tech.yamba.db.jooq;


import javax.annotation.Generated;

import org.jooq.Sequence;
import org.jooq.impl.SequenceImpl;


/**
 * Convenience access to all sequences in public
 */
@Generated(
    value = {
        "http://www.jooq.org",
        "jOOQ version:3.11.9"
    },
    comments = "This class is generated by jOOQ"
)
@SuppressWarnings({ "all", "unchecked", "rawtypes" })
public class Sequences {

    /**
     * The sequence <code>public.authorities_id_seq</code>
     */
    public static final Sequence<Short> AUTHORITIES_ID_SEQ = new SequenceImpl<Short>("authorities_id_seq", Public.PUBLIC, org.jooq.impl.SQLDataType.SMALLINT.nullable(false));

    /**
     * The sequence <code>public.groups_id_seq</code>
     */
    public static final Sequence<Short> GROUPS_ID_SEQ = new SequenceImpl<Short>("groups_id_seq", Public.PUBLIC, org.jooq.impl.SQLDataType.SMALLINT.nullable(false));

    /**
     * The sequence <code>public.instances_id_seq</code>
     */
    public static final Sequence<Integer> INSTANCES_ID_SEQ = new SequenceImpl<Integer>("instances_id_seq", Public.PUBLIC, org.jooq.impl.SQLDataType.INTEGER.nullable(false));

    /**
     * The sequence <code>public.users_id_seq</code>
     */
    public static final Sequence<Short> USERS_ID_SEQ = new SequenceImpl<Short>("users_id_seq", Public.PUBLIC, org.jooq.impl.SQLDataType.SMALLINT.nullable(false));
}
