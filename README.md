Linux2Rest

Get /,/v4l

Post /v4l
```
curl -H "Content-Type: application/json" --data '{
    "camera": "/dev/video0",
    "control": {
        "id": 9963803, // Sharpness
        "value": {
            "Integer": 255
        }
    }
}' --request POST http://0.0.0.0:8088/v4l

```