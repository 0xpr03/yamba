package tech.yamba.management.websocket;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;

import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.stereotype.Component;
import org.springframework.web.socket.CloseStatus;
import org.springframework.web.socket.TextMessage;
import org.springframework.web.socket.WebSocketSession;
import org.springframework.web.socket.config.annotation.EnableWebSocket;
import org.springframework.web.socket.config.annotation.WebSocketConfigurer;
import org.springframework.web.socket.config.annotation.WebSocketHandlerRegistry;
import org.springframework.web.socket.handler.TextWebSocketHandler;

import lombok.extern.slf4j.Slf4j;


@Configuration
@EnableWebSocket
@Slf4j
public class WebSocketConfig implements WebSocketConfigurer {

	@Override
	public void registerWebSocketHandlers(WebSocketHandlerRegistry registry) {
		registry.addHandler(new SubscriberTextWebSocketHandler(), "/socket").withSockJS();
	}


	@Component
	public static class SubscriberTextWebSocketHandler extends TextWebSocketHandler {

		Map<WebSocketSession, List<String>> sessions;


		@Bean
		public Map<WebSocketSession, List<String>> getWebsocketSessions() {
			return sessions;
		}


		@Override protected void handleTextMessage(WebSocketSession session, TextMessage message) throws Exception {
			super.handleTextMessage(session, message);
		}


		@Override public void afterConnectionEstablished(WebSocketSession session) {
			sessions.put(session, new ArrayList<>());
		}


		@Override public void afterConnectionClosed(WebSocketSession session, CloseStatus status) {
			sessions.remove(session);
			if (status != CloseStatus.NORMAL && status != CloseStatus.GOING_AWAY) {
				log.warn("Bad close status on Websocket session '{}' with Close status '{}'", session, status);
			}
		}

	}

}
