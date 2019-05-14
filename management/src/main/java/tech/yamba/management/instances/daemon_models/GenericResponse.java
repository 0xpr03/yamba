package tech.yamba.management.instances.daemon_models;

import lombok.Data;

import java.util.Optional;

@Data
public class GenericResponse {
    Optional<String> msg;
}
