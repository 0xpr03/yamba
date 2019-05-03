package tech.yamba.management.websocket;

import java.io.IOException;
import java.util.HashMap;
import java.util.HashSet;
import java.util.Map;
import java.util.Set;
import java.util.function.Function;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.context.annotation.Bean;
import org.springframework.stereotype.Service;
import org.springframework.web.servlet.mvc.method.annotation.RequestMappingHandlerMapping;
import org.springframework.web.socket.CloseStatus;
import org.springframework.web.socket.TextMessage;
import org.springframework.web.socket.WebSocketSession;
import org.springframework.web.socket.handler.TextWebSocketHandler;

import lombok.extern.slf4j.Slf4j;
import tech.yamba.management.websocket.messages.ClientMessage;
import tech.yamba.management.websocket.messages.ServerMessage;
import tech.yamba.management.websocket.messages.YambaMessage;


@Service
@Slf4j
public class WebSocketService extends TextWebSocketHandler {

	private final Map<WebSocketSession, Set<String>> sessions = new HashMap<>();
	private final RequestMappingHandlerMapping mapper;


	@Autowired public WebSocketService(RequestMappingHandlerMapping mapper) {
		this.mapper = mapper;
	}


	public void notifySubscribers(Class<?> controllerClass, Function<String, Boolean> methodNameMatcher) {
		mapper.getHandlerMethods().forEach((key, value) -> {
			if (value.getMethod().getDeclaringClass().equals(controllerClass) && methodNameMatcher.apply(value.getMethod().getName())) {
				key.getPatternsCondition().getPatterns().forEach(this::notifySubscribers);
			}
		});
	}


	public void notifySubscribers(String endpoint) {
		sessions
				.entrySet()
				.stream()
				.filter(entry -> entry.getValue().contains(endpoint))
				.forEach(session -> {
					try {
						session.getKey().sendMessage(new YambaMessage<>(ServerMessage.UPDATE, endpoint).toTextMessage());
					} catch (IOException e) {
						e.printStackTrace();
					}
				});
	}

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
			YambaMessage<ClientMessage> clientMessage = new YambaMessage<>(ClientMessage.class, message.getPayload());

			switch (clientMessage.getMethod()) {
			case SUBSCRIBE:
				subscriptions.add(clientMessage.getBody());
				break;
			case DESUB:
				subscriptions.remove(clientMessage.getBody());
				break;
			}

			session.sendMessage(new YambaMessage<>(ServerMessage.OK).toTextMessage());
		} catch (Exception e) {
			session.sendMessage(new YambaMessage<>(ServerMessage.BAD, message.getPayload()).toTextMessage());
		}
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
