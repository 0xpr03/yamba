package tech.yamba.management.websocket.messages;

import org.springframework.web.socket.TextMessage;

import lombok.AllArgsConstructor;
import lombok.Data;
import lombok.RequiredArgsConstructor;


@Data
@AllArgsConstructor
@RequiredArgsConstructor
public class YambaMessage<M extends Enum<M>> {

	private final M method;
	private String body;


	public YambaMessage(Class<M> methodType, String message) {
		String[] parts = message.split("\n\n");
		if (parts.length == 0) {
			throw new IllegalArgumentException("Bad message");
		}
		method = Enum.valueOf(methodType, parts[0]);
		body = parts.length > 1 ? parts[1] : "";
	}


	public TextMessage toTextMessage() {
		return new TextMessage(getMethod().name() + "\n\n" + getBody());
	}
}
