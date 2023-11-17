import json
import websocket
import threading
import time

def on_message(ws, message):
    # print("Raw Message:", message)
    msg_json = json.loads(message)
    event = msg_json["event"]
    print("Received:", event)
    # if event != "game_state_update":
    #     print("Received:", json.dumps(msg_json, indent=4))
    # else:
    #     print("Received game state update")

    if event == "new_game":
        game_id = msg_json["data"]["id"]
        start_game_msg = json.dumps({
            "event": "start_game",
            "data": {"id": game_id}
        })
        time.sleep(4)
        print("Sending:", start_game_msg)
        ws.send(start_game_msg)
    if event == "game_over":
        print("Game over, closing connection")
        ws.close()
    elif event == "move_request":
        data = msg_json["data"]
        moves = data["move_list"]
        move = moves[0]
        request_id = data["request_id"]
        time.sleep(1)
        print("Sending move:", move, "for request id:", request_id)
        ws.send(json.dumps({
            "event": "move_response",
            "data": {
                "request_id": request_id,
                "move_index": 0
            }
        }))

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

    def run(*args):
        import time
        while True:
            time.sleep(ping_interval)  # Wait for the specified interval
            ping_msg = json.dumps({
                "event": "ping",
                "data": {1: "test 1 2 3"}
            })
            print("Sending:", ping_msg)
            ws.send(ping_msg)

    # The daemon flag is set to True so that the thread will stop when the main thread exits
    threading.Thread(target=run, daemon=True).start()

    new_game_msg = json.dumps({
        "event": "new_game",
        "data": {
            "players": [
                {"name": "Player 1", "type": "random"},
                {"name": "Player 2", "type": "mcts"}
            ]
        }
    })
    print("Sending:", new_game_msg)
    ws.send(new_game_msg)

if __name__ == "__main__":
    ws_url = "ws://127.0.0.1:3001"
    
    # send ping every second
    ping_interval = 1
    ping_msg = json.dumps({
        "event": "ping",
        "data": {1: "test 1 2 3"}
    })

    ws_app = websocket.WebSocketApp(
        ws_url,
        on_message=on_message,
        on_error=on_error,
        on_close=on_close
    )
    ws_app.on_open = on_open
    ws_app.run_forever()
