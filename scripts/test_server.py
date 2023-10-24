import websocket
import json

def on_message(ws, message):
    data = json.loads(message)
    if data.get("event") == "pong":
        print("Received:", data["event"])

def main():
    ws_url = "ws://127.0.0.1:3001" # adjust if necessary
    ws = websocket.WebSocketApp(
        ws_url,
        on_message=on_message
    )

    # Define the "ping" message
    ping_msg = json.dumps({"event": "ping"})

    # Send the "ping" once the connection is open
    def on_open(ws_instance):
        ws_instance.send(ping_msg)

    ws.on_open = on_open
    ws.run_forever()

if __name__ == "__main__":
    main()
