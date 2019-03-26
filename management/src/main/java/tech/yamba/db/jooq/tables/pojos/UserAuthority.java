/*
 * This file is generated by jOOQ.
 */
package tech.yamba.db.jooq.tables.pojos;


import java.io.Serializable;

import javax.annotation.Generated;


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
public class UserAuthority implements Serializable {

    private static final long serialVersionUID = -38175516;

    private Short userId;
    private Short authorityId;

    public UserAuthority() {}

    public UserAuthority(UserAuthority value) {
        this.userId = value.userId;
        this.authorityId = value.authorityId;
    }

    public UserAuthority(
        Short userId,
        Short authorityId
    ) {
        this.userId = userId;
        this.authorityId = authorityId;
    }

    public Short getUserId() {
        return this.userId;
    }

    public UserAuthority setUserId(Short userId) {
        this.userId = userId;
        return this;
    }

    public Short getAuthorityId() {
        return this.authorityId;
    }

    public UserAuthority setAuthorityId(Short authorityId) {
        this.authorityId = authorityId;
        return this;
    }

    @Override
    public boolean equals(Object obj) {
        if (this == obj)
            return true;
        if (obj == null)
            return false;
        if (getClass() != obj.getClass())
            return false;
        final UserAuthority other = (UserAuthority) obj;
        if (userId == null) {
            if (other.userId != null)
                return false;
        }
        else if (!userId.equals(other.userId))
            return false;
        if (authorityId == null) {
            if (other.authorityId != null)
                return false;
        }
        else if (!authorityId.equals(other.authorityId))
            return false;
        return true;
    }

    @Override
    public int hashCode() {
        final int prime = 31;
        int result = 1;
        result = prime * result + ((this.userId == null) ? 0 : this.userId.hashCode());
        result = prime * result + ((this.authorityId == null) ? 0 : this.authorityId.hashCode());
        return result;
    }

    @Override
    public String toString() {
        StringBuilder sb = new StringBuilder("UserAuthority (");

        sb.append(userId);
        sb.append(", ").append(authorityId);

        sb.append(")");
        return sb.toString();
    }
}