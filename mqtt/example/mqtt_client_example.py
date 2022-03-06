import mqtt_client
import mqtt_message
import time

def mqtt_receive_callback(client, message: mqtt_message):
    print("Received message for topic {0}".format(message.topic))

def main() -> int:
    client = mqtt_client.mqtt_client()

    res = client.connect('localhost', 1883, 60)
    while res != 0:
        print("Could not connect -- retrying")
        time.sleep(5)
        res = client.connect('localhost', 1883, 60)

    print('Connected!')

    client.receive_callback_set(mqtt_receive_callback)
    client.subscribe('#', 2)
    client.publish('/test/topic', 14, 'Hello, World!', 0, False)

    while True:
        time.sleep(1)

if __name__ == "__main__":
    main()