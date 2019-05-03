package tech.yamba.management.websocket.messages;

import lombok.AllArgsConstructor;


@AllArgsConstructor
public enum ClientMessage {
	/**
	 * Subscribe to an enpoint
	 */
	SUBSCRIBE("SUBSCRIBE"),
	/**
	 * End endpoint subscription
	 */
	DESUB("DESUB");

	private final String method;
}