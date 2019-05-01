package tech.yamba.management.websocket;

import java.util.HashMap;
import java.util.HashSet;
import java.util.Map;
import java.util.Set;

import org.springframework.context.annotation.Bean;
import org.springframework.stereotype.Service;
import org.springframework.web.socket.CloseStatus;
import org.springframework.web.socket.TextMessage;
import org.springframework.web.socket.WebSocketSession;
import org.springframework.web.socket.handler.TextWebSocketHandler;

import lombok.extern.slf4j.Slf4j;


@Service
@Slf4j
public class WebSocketService extends TextWebSocketHandler {

	private final Map<WebSocketSession, Set<String>> sessions = new HashMap<>();


	@Bean
	public Map<WebSocketSession, Set<String>> getWebsocketSessions() {
		return sessions;
	}


	@Override
	protected void handleTextMessage(WebSocketSession session, TextMessage message) throws Exception {
		Set<String> subscriptions = sessions.get(session);
		if (subscriptions == null) {
			throw new IllegalStateException("Session not in sessions map sent message:\n" + message);
		}
		try {
			YambaMessage messageObject = new YambaMessage(message.getPayload());

			switch (messageObject.getHeadline()) {
			case SUBSCRIBE:
				subscriptions.add(messageObject.getBody());
				break;
			case DESUB:
				subscriptions.remove(messageObject.getBody());
				break;
			}

			session.sendMessage(new TextMessage("OK"));
		} catch (Exception e) {
			session.sendMessage(new TextMessage("BAD\n\n" + message.getPayload()));
		}

		log.info(sessions.toString());
	}


	@Override
	public void afterConnectionEstablished(WebSocketSession session) {
		sessions.put(session, new HashSet<>());
	}


	@Override
	public void afterConnectionClosed(WebSocketSession session, CloseStatus status) {
		sessions.remove(session);
		if (!status.equalsCode(CloseStatus.NORMAL) && !status.equalsCode(CloseStatus.GOING_AWAY)) {
			log.warn("Bad close status on Websocket session '{}' with Close status '{}'", session, status);
		}
	}
}
