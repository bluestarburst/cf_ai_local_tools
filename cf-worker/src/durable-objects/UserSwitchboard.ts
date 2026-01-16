/**
 * UserSwitchboard - Simple WebSocket relay between desktop and web viewers
 *
 * Architecture: Dumb relay pattern
 * - Accepts WebSocket connections from both desktop and web-viewer clients
 * - Relays messages between them without processing
 * - One instance per user for isolation
 */

export class UserSwitchboard {
  private state: DurableObjectState;
  private desktop: WebSocket | null = null;
  private webViewers: Set<WebSocket> = new Set();

  constructor(state: DurableObjectState) {
    this.state = state;
  }

  async fetch(request: Request): Promise<Response> {
    const url = new URL(request.url);

    // WebSocket upgrade
    if (request.headers.get('Upgrade') === 'websocket') {
      const deviceType = url.searchParams.get('device') || 'desktop';
      return this.handleWebSocket(request, deviceType);
    }

    // Status check
    if (url.pathname === '/status') {
      return Response.json({
        desktop: this.desktop !== null,
        webViewers: this.webViewers.size,
      });
    }

    return new Response('Not Found', { status: 404 });
  }

  private handleWebSocket(request: Request, deviceType: string): Response {
    const pair = new WebSocketPair();
    const [client, server] = Object.values(pair);

    server.accept();

    if (deviceType === 'desktop') {
      // Only one desktop connection allowed
      if (this.desktop) {
        this.desktop.close(1000, 'New desktop connected');
      }
      this.desktop = server;
      console.log('Desktop connected');
    } else {
      // Multiple web viewers allowed
      this.webViewers.add(server);
      console.log(`Web viewer connected (total: ${this.webViewers.size})`);
    }

    server.addEventListener('message', (event: MessageEvent) => {
      try {
        // Relay: web-viewer → desktop
        if (deviceType === 'web-viewer' && this.desktop) {
          this.desktop.send(event.data);
        }

        // Relay: desktop → all web-viewers
        if (deviceType === 'desktop') {
          for (const viewer of this.webViewers) {
            try {
              viewer.send(event.data);
            } catch (err) {
              console.error('Failed to send to web viewer:', err);
            }
          }
        }
      } catch (err) {
        console.error('Error relaying message:', err);
      }
    });

    server.addEventListener('close', () => {
      if (deviceType === 'desktop') {
        console.log('Desktop disconnected');
        this.desktop = null;
      } else {
        console.log(`Web viewer disconnected (remaining: ${this.webViewers.size - 1})`);
        this.webViewers.delete(server);
      }
    });

    server.addEventListener('error', (event: Event) => {
      console.error(`WebSocket error (${deviceType}):`, event);
    });

    return new Response(null, {
      status: 101,
      webSocket: client,
    });
  }
}
