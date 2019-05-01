package tech.yamba.management.websocket;

import java.util.List;
import java.util.Map;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Service;
import org.springframework.web.socket.WebSocketSession;


@Service
public class WebsocketService {

	private final Map<WebSocketSession, List<String>> webSocketSessions;

	@Autowired
	public WebsocketService(Map<WebSocketSession, List<String>> webSocketSessions) {
		this.webSocketSessions = webSocketSessions;
	}

	public void pushRefetchFromUrl(String url) {

	}
}
