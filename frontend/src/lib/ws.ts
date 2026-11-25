/**
 * WebSocket client with automatic reconnection and typed events.
 */

type MessageHandler = (data: any) => void;

export class WSClient {
	private ws: WebSocket | null = null;
	private url: string | null;
	private handlers: Map<string, MessageHandler[]> = new Map();
	private reconnectAttempts = 0;
	private maxReconnectAttempts = 10;
	private reconnectDelay = 1000;
	/** Set by `disconnect()` so leaving a page does not start a reconnect loop. */
	private closedByUser = false;

	constructor(url?: string) {
		this.url = url ?? null;
	}

	/** Resolve lazily so SSR never touches `window`. */
	private resolveUrl(): string {
		if (this.url) return this.url;
		const configured = import.meta.env.VITE_WS_URL;
		if (configured) return configured;
		if (typeof window === 'undefined') return '';
		const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
		return `${protocol}//${window.location.host}/ws`;
	}

	get isOpen(): boolean {
		return this.ws?.readyState === WebSocket.OPEN;
	}

	connect(): void {
		if (this.ws?.readyState === WebSocket.OPEN) {
			return;
		}

		const url = this.resolveUrl();
		if (!url) return;

		this.closedByUser = false;
		this.ws = new WebSocket(url);

		this.ws.onopen = () => {
			this.reconnectAttempts = 0;
			this.emit('connected', {});
		};

		this.ws.onmessage = (event) => {
			try {
				const message = JSON.parse(event.data);
				this.emit(message.type, message.data);
			} catch (error) {
				console.error('Failed to parse WS message:', error);
			}
		};

		this.ws.onclose = () => {
			this.emit('disconnected', {});
			if (!this.closedByUser) this.attemptReconnect();
		};

		this.ws.onerror = () => {
			// A failed socket always closes afterwards, so reconnection is handled there.
			this.emit('socket_error', {});
		};
	}

	send(type: string, data: any = {}): void {
		if (this.ws?.readyState !== WebSocket.OPEN) {
			console.warn('WebSocket is not connected, dropping message', type);
			return;
		}
		this.ws.send(JSON.stringify({ type, data }));
	}

	on(event: string, handler: MessageHandler): void {
		if (!this.handlers.has(event)) {
			this.handlers.set(event, []);
		}
		this.handlers.get(event)!.push(handler);
	}

	off(event: string, handler: MessageHandler): void {
		const handlers = this.handlers.get(event);
		if (!handlers) return;
		const index = handlers.indexOf(handler);
		if (index > -1) handlers.splice(index, 1);
	}

	private emit(event: string, data: any): void {
		this.handlers.get(event)?.forEach((handler) => handler(data));
	}

	private attemptReconnect(): void {
		if (this.reconnectAttempts >= this.maxReconnectAttempts) {
			this.emit('reconnect_failed', {});
			return;
		}

		this.reconnectAttempts++;
		const delay = this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1);
		setTimeout(() => this.connect(), delay);
	}

	disconnect(): void {
		this.closedByUser = true;
		this.ws?.close();
		this.ws = null;
	}
}

// Singleton instance
export const ws = new WSClient();
