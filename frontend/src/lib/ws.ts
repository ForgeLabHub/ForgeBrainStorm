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

	connect(): void {
		if (this.ws?.readyState === WebSocket.OPEN) {
			return;
		}

		const url = this.resolveUrl();
		if (!url) return;
		this.ws = new WebSocket(url);

		this.ws.onopen = () => {
			console.log('WebSocket connected');
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
			console.log('WebSocket disconnected');
			this.emit('disconnected', {});
			this.attemptReconnect();
		};

		this.ws.onerror = (error) => {
			console.error('WebSocket error:', error);
			this.emit('error', { error });
		};
	}

	send(type: string, data: any = {}): void {
		if (this.ws?.readyState !== WebSocket.OPEN) {
			console.warn('WebSocket not connected');
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
		if (handlers) {
			const index = handlers.indexOf(handler);
			if (index > -1) {
				handlers.splice(index, 1);
			}
		}
	}

	private emit(event: string, data: any): void {
		const handlers = this.handlers.get(event);
		if (handlers) {
			handlers.forEach((handler) => handler(data));
		}
	}

	private attemptReconnect(): void {
		if (this.reconnectAttempts >= this.maxReconnectAttempts) {
			console.error('Max reconnect attempts reached');
			return;
		}

		this.reconnectAttempts++;
		const delay = this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1);
		console.log(`Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts})`);

		setTimeout(() => {
			this.connect();
		}, delay);
	}

	disconnect(): void {
		this.ws?.close();
		this.ws = null;
	}
}

// Singleton instance
export const ws = new WSClient();
