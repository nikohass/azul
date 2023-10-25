import json
import websocket

def on_message(ws, message):
    msg_json = json.loads(message)
    if not msg_json["event"] == "game_state_update":
        print("Received:", json.dumps(msg_json, indent=4))
    else:
        print("Received game state update...")
    
    if msg_json["event"] == "new_game":
        game_id = msg_json["data"]["id"]
        start_game_msg = json.dumps({
            "event": "start_game",
            "data": {"id": game_id}
        })
        print("Sending:", start_game_msg)
        ws.send(start_game_msg)

def on_error(ws, error):
    print("Error:", error)

def on_close(ws, close_status_code, close_msg):
    print("WebSocket Closed")

def on_open(ws):
    ping_msg = json.dumps({
        "event": "ping",
        "data": {1: "test 1 2 3"}
    })
    print("Sending:", ping_msg)
    ws.send(ping_msg)

    new_game_msg = json.dumps({
        "event": "new_game",
        "data": {
            "players": [
                {"name": "Player 1", "type": "computer"},
                {"name": "Player 2", "type": "computer"}
            ]
        }
    })
    print("Sending:", new_game_msg)
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
