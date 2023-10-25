import json
import websocket

def on_message(ws, message):
    formatted_message = json.loads(message)
    print("Received:", json.dumps(formatted_message, indent=4))

def on_error(ws, error):
    print("Error:", error)

def on_close(ws, close_status_code, close_msg):
    print("WebSocket Closed")

def on_open(ws):
    ping_msg = json.dumps({
        "event": "ping",
        "data": {1: "test 1 2 3"}
    })
    ws.send(ping_msg)

    new_game_msg = json.dumps({
        "event": "new_game",
        "data": {
            "players": [
                {"name": "Player 1", "type": "human"},
                {"name": "Player 2", "type": "computer"}
            ]
        }
    })
    ws.send(new_game_msg)

if __name__ == "__main__":
    ws_url = "ws://127.0.0.1:3001"
    
    ws_app = websocket.WebSocketApp(
        ws_url,
        on_message=on_message,
        on_error=on_error,
        on_close=on_close
    )
    ws_app.on_open = on_open
    ws_app.run_forever()
