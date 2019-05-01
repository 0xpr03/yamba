package tech.yamba.management.websocket;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.context.annotation.Configuration;
import org.springframework.web.socket.config.annotation.EnableWebSocket;
import org.springframework.web.socket.config.annotation.WebSocketConfigurer;
import org.springframework.web.socket.config.annotation.WebSocketHandlerRegistry;

import lombok.extern.slf4j.Slf4j;


@Configuration
@EnableWebSocket
@Slf4j
public class WebSocketConfig implements WebSocketConfigurer {

	private final WebSocketService websocketService;


	@Autowired
	public WebSocketConfig(WebSocketService websocketService) {
		this.websocketService = websocketService;
	}


	@Override
	public void registerWebSocketHandlers(WebSocketHandlerRegistry registry) {
		registry.addHandler(websocketService, "/api/socket").withSockJS();
	}

}
