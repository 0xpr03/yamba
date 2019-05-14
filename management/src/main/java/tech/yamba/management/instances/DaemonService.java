package tech.yamba.management.instances;

import org.springframework.stereotype.Service;
import org.springframework.web.client.RestTemplate;
import tech.yamba.db.jooq.tables.pojos.Instance;
import tech.yamba.management.instances.daemon_models.GenericRequest;
import tech.yamba.management.instances.daemon_models.GenericResponse;
import tech.yamba.management.instances.daemon_models.InstanceLoadResponse;

import java.util.concurrent.ConcurrentHashMap;

@Service
public class DaemonService {

    private final ConcurrentHashMap<Integer, Instance> runningInstances;

    public DaemonService() {
        runningInstances = new ConcurrentHashMap<>();
    }

    public void startInstance(Instance instance) {
        final String uri = "http://localhost:8080/springrestexample/employees";

        RestTemplate restTemplate = new RestTemplate();
        InstanceLoadResponse result = restTemplate.postForObject( uri, instance, InstanceLoadResponse.class);

        System.out.println(result);
        runningInstances.put(instance.getId(),instance);
    }

    public boolean isRunning(int id) {
        return runningInstances.containsKey(id);
    }

    public void stopInstance(int id) {
        final String uri = "http://localhost:8080/springrestexample/employees";

        RestTemplate restTemplate = new RestTemplate();
        GenericRequest req = new GenericRequest(id);
        GenericResponse result = restTemplate.postForObject( uri, req, GenericResponse.class);
        System.out.println(result);
        runningInstances.remove(id);
    }
}
