package tech.yamba.management.websocket;

import lombok.AllArgsConstructor;
import lombok.Getter;


@Getter
@AllArgsConstructor
public class YambaMessage {

	private final Method headline;
	private final String body;


	public YambaMessage(String message) {
		String[] parts = message.split("\n\n");
		headline = Method.valueOf(parts[0]);
		body = parts[1];
	}

	@AllArgsConstructor
	public enum Method {
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
}
