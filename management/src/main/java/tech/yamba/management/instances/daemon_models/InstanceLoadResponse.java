package tech.yamba.management.instances.daemon_models;

import lombok.Data;

import java.io.Serializable;

@Data
public class InstanceLoadResponse implements Serializable {
    private long startup_time;
}
