package tech.yamba.management.websocket.messages;

import lombok.AllArgsConstructor;


@AllArgsConstructor
public enum ServerMessage {
	/**
	 * Request was OK
	 */
	OK("OK"),
	/**
	 * Request was BAD
	 */
	BAD("BAD"),
	/**
	 * Update endpoint
	 */
	UPDATE("UPDATE");

	private final String method;
}
