from typing import Any, Dict


class Info:
    def __init__(self) -> None:
        pass

    @classmethod
    def from_json(cls, json_data: Dict[str, Any]) -> "Info":
        return cls()


class FwInfo:
    def __init__(self) -> None:
        pass

    @classmethod
    def from_json(cls, json_data: Dict[str, Any]) -> "FwInfo":
        return cls()
