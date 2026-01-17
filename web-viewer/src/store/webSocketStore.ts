/**
 * WebSocket Store - Zustand State Management
 * Manages WebSocket connection for backend communication
 */

import { create } from 'zustand';

interface WebSocketState {
  // WebSocket instance
  ws: WebSocket | null;

  // Connection status
  connected: boolean;

  // Connection URL
  url: string | null;

  // Actions
  connect: (url: string) => void;
  disconnect: () => void;
  send: (data: any) => void;
  setWebSocket: (ws: WebSocket) => void;
}

export const useWebSocketStore = create<WebSocketState>((set, get) => ({
  ws: null,
  connected: false,
  url: null,

  connect: (url: string) => {
    // Don't reconnect if already connected to same URL
    const { ws, connected, url: currentUrl } = get();
    if (connected && currentUrl === url && ws && ws.readyState === WebSocket.OPEN) {
      console.log('[WebSocketStore] Already connected to:', url);
      return;
    }

    console.log('[WebSocketStore] Connecting to:', url);

    try {
      const socket = new WebSocket(url);

      socket.onopen = () => {
        console.log('[WebSocketStore] Connected');
        set({ ws: socket, connected: true, url });
      };

      socket.onclose = (event) => {
        console.log('[WebSocketStore] Disconnected', event.code, event.reason);
        set({ ws: null, connected: false });

        // Auto-reconnect after 3 seconds
        setTimeout(() => {
          console.log('[WebSocketStore] Reconnecting...');
          const { url: reconnectUrl } = get();
          if (reconnectUrl) {
            useWebSocketStore.getState().connect(reconnectUrl);
          }
        }, 3000);
      };

      socket.onerror = (error) => {
        console.error('[WebSocketStore] Error:', error);
        set({ connected: false });
      };

      set({ ws: socket });
    } catch (error) {
      console.error('[WebSocketStore] Connection error:', error);
      set({ connected: false });
    }
  },

  disconnect: () => {
    const { ws } = get();
    if (ws) {
      console.log('[WebSocketStore] Disconnecting...');
      ws.close();
      set({ ws: null, connected: false, url: null });
    }
  },

  send: (data: any) => {
    const { ws, connected } = get();
    if (ws && connected && ws.readyState === WebSocket.OPEN) {
      try {
        ws.send(JSON.stringify(data));
      } catch (error) {
        console.error('[WebSocketStore] Send error:', error);
      }
    } else {
      console.warn('[WebSocketStore] Cannot send, not connected');
    }
  },

  setWebSocket: (ws: WebSocket) => {
    set({ ws });
  },
}));
