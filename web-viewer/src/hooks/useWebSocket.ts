import { useState, useEffect, useCallback, useRef } from 'react';

export function useWebSocket(url: string) {
  const [ws, setWs] = useState<WebSocket | null>(null);
  const [connected, setConnected] = useState(false);
  const reconnectTimeoutRef = useRef<number>();

  useEffect(() => {
    let socket: WebSocket | null = null;

    const connect = () => {
      try {
        console.log('[WebSocket] Connecting to:', url);
        socket = new WebSocket(url);

        socket.onopen = () => {
          console.log('[WebSocket] Connected');
          setConnected(true);
          setWs(socket);
        };

        socket.onclose = () => {
          console.log('[WebSocket] Disconnected');
          setConnected(false);
          setWs(null);

          // Auto-reconnect after 3 seconds
          reconnectTimeoutRef.current = setTimeout(() => {
            console.log('[WebSocket] Reconnecting...');
            connect();
          }, 3000);
        };

        socket.onerror = (error) => {
          console.error('[WebSocket] Error:', error);
        };
      } catch (error) {
        console.error('[WebSocket] Connection error:', error);
        setConnected(false);
      }
    };

    connect();

    return () => {
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
      if (socket) {
        socket.close();
      }
    };
  }, [url]);

  const send = useCallback(
    (data: any) => {
      if (ws && connected) {
        ws.send(JSON.stringify(data));
      } else {
        console.warn('[WebSocket] Cannot send, not connected');
      }
    },
    [ws, connected]
  );

  return { ws, connected, send };
}
