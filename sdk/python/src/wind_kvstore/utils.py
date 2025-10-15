import json
import re
import warnings


def remind(msg: str)->None:
    if not msg:
        raise ValueError(
            f"The warning msg can not be empty."
        )
    warnings.warn(
        msg,
        UserWarning,
        stacklevel=3
    )


def format_exec_response(response_text: str)->json:
    cleaned_text = response_text.strip('"').replace('\\"', '"')
    parts = cleaned_text.split('" "')
    result = {}
    for i, part in enumerate(parts, 1):
        part = part.strip('"')
        if ': ' in part:
            cmd_part, msg_part = part.rsplit(': ', 1)
            cmd = cmd_part.rstrip(';')
            if i == 1:
                cmd = cmd.replace('{\"status\":\"\"\\n    ', '').strip()
            if ": Error" not in cmd:
                result[f"cmd{i}"] = {
                    "command": cmd.lstrip(),
                    "message": msg_part.strip("'")
                }
    return json.dumps(result, indent=2)


def format_exec_put_command(input_str: str)->str:
    pattern = r'(PUT\s*"[^"]+")\s*:\s*("[^"]+")'
    result = re.sub(pattern, r'\1:\2', input_str)
    return result