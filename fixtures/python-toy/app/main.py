from app.orchestrate import orchestrate
import json


def main() -> None:
    result = orchestrate("hello")
    print(json.dumps({"result": result}))


if __name__ == "__main__":
    main()
