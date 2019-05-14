package tech.yamba.management.instances;

import lombok.extern.slf4j.Slf4j;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.http.HttpStatus;
import org.springframework.http.MediaType;
import org.springframework.http.ResponseEntity;
import org.springframework.stereotype.Controller;
import org.springframework.web.bind.annotation.*;
import tech.yamba.db.jooq.tables.pojos.Instance;
import tech.yamba.management.websocket.WebSocketService;

import java.util.HashMap;
import java.util.List;
import java.util.concurrent.locks.ReadWriteLock;
import java.util.concurrent.locks.ReentrantReadWriteLock;


@Controller
@RequestMapping(value = "/api/instance", produces = MediaType.APPLICATION_JSON_VALUE)
@Slf4j
public class InstanceController {

	private final InstanceService instanceService;
	private final DaemonService daemonService;
	private final WebSocketService webSocketService;
	private final HashMap<Integer,Instance> runningInstance;
	private final ReadWriteLock instancesLock;


	@Autowired
	public InstanceController(InstanceService instanceService, DaemonService daemonService, WebSocketService webSocketService) {
		this.instanceService = instanceService;
		this.daemonService = daemonService;
		this.webSocketService = webSocketService;
		runningInstance = new HashMap<>();
		instancesLock = new ReentrantReadWriteLock();
	}


	@PostMapping()
	public ResponseEntity<Instance> addInstance(@RequestBody Instance instance) {
		Instance result = instanceService.addInstance(instance);
		notifyGetInstance();
		return new ResponseEntity<>(result, HttpStatus.CREATED);
	}

	@PutMapping("/{id}/start")
	public ResponseEntity startInstance(@PathVariable int id) {
		if (!daemonService.isRunning(id)) {
			Instance instance = instanceService.getInstanceById(id);
			daemonService.startInstance(instance);
		}
		return new ResponseEntity(HttpStatus.OK);
	}

	@PutMapping("/{id}/stop")
	public ResponseEntity stopInstance(@PathVariable int id) {
		if (daemonService.isRunning(id)) {
			daemonService.stopInstance(id);
		}
		return new ResponseEntity(HttpStatus.OK);
	}


	@GetMapping()
	public ResponseEntity<List<Instance>> getInstances() {
		return new ResponseEntity<>(instanceService.getInstances(), HttpStatus.OK);
	}


	@GetMapping("/{id}")
	public ResponseEntity<Instance> getInstanceById(@PathVariable int id) {
		return new ResponseEntity<>(instanceService.getInstanceById(id), HttpStatus.OK);
	}


	@PutMapping("/{id}")
	public ResponseEntity<Instance> updateInstance(@PathVariable int id, @RequestBody Instance instance) {
		Instance result = instanceService.updateInstance(instance.setId(id));
		notifyGetInstance();
		return new ResponseEntity<>(result, HttpStatus.OK);
	}


	@DeleteMapping("/{id}")
	public ResponseEntity deleteInstance(@PathVariable short id) {
		instanceService.deleteInstance(id);
		notifyGetInstance();
		return new ResponseEntity(HttpStatus.NO_CONTENT);
	}


	private void notifyGetInstance() {
		webSocketService.notifySubscribers(InstanceController.class, methodName -> methodName.startsWith("getInstances"));
	}
}
