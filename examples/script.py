import asyncio
import binascii
import hashlib
import struct
import sys
from enum import Enum
from typing import List, Optional


def _to_hexstring(data: bytes):
    return binascii.b2a_hex(data).upper()


def _from_hexstring(data: str):
    return bytes.fromhex(data)


class ComError(Exception):
    def __init__(self, msg):
        super().__init__(msg)
        self._msg = msg

    @property
    def message(self):
        return self._msg


class ScriptExecutionFailed(Exception):
    pass


class Executor(object):
    """
    Abstract class to implement an executor to run
    through a script file.
    """
    def start(self):
        """Called at startup"""
        pass

    async def write(self, data: bytes):
        """
        Perform a write operation.

        :param data: Data to be written to the device
        :return: None
        """
        raise NotImplementedError

    async def query(self, data: bytes):
        """
        Perform a write/read operation and return the resulting data.

        :param data: Data to be written to the device
        :return: Return the data read from the device
        """
        raise NotImplementedError

    def update_progress(self, progress: float):
        """
        Called when a progress update command is encountered

        :param progress:
        :return:
        """
        pass

    def print_error(self, msg: str):
        print(msg, file=sys.stderr)

    def print_debug(self, msg: str):
        print(msg)

    def print_log(self, msg: str):
        print(msg)

    async def sleep(self, ms: int):
        await asyncio.sleep(ms / 1000.0)

    def finished(self, success: bool):
        """
        Called when the script finishes execution.

        :param success: True if script execution was successful
        :return: None
        """
        if not success:
            raise ScriptExecutionFailed()


class CommandType(Enum):
    HEADER = 0x01
    WRITE = 0x02
    QUERY = 0x03
    SET_TIMEOUT = 0x10
    LOG = 0x20
    SET_ERROR = 0x21
    PROGRESS = 0x22
    CHECKSUM = 0x30


class State(object):
    def __init__(self):
        self.timeout_ms = 0
        self.error_msg = ''
        self.should_abort = False

    def abort(self):
        self.should_abort = True


class Command(object):
    async def run(self, state: State, executor: Executor):
        pass

    def type(self) -> CommandType:
        raise NotImplementedError

    def dump(self) -> str:
        raise NotImplementedError


class Checksum(Command):
    def __init__(self, chksum: bytes):
        self._chksum = chksum

    @property
    def checksum(self) -> bytes:
        return self._chksum

    def type(self) -> CommandType:
        return CommandType.CHECKSUM

    def dump(self) -> str:
        return 'Checksum({})'.format(_to_hexstring(self._chksum))


class ChecksumVerificationFailed(Exception):
    pass


class InvalidScript(Exception):
    def __init__(self, msg):
        super().__init__(msg)
        self._msg = msg


class Script(object):
    """
    Represents a DDP script as a list commands.
    A script can be run by calling the `script.run(executor)` method.

    Parsing a script from a string should use the `Script.parse()` function.
    """
    def __init__(self, cmds: List[Command]):
        self._cmds = cmds

    @property
    def cmds(self):
        return self._cmds

    def _verify_content(self, content: str):
        chksum = self._get_checksum_command()
        if chksum is None:
            return False
        end = content.rfind(':')
        if end == -1:
            return False
        content = content[0:end - 1].strip()
        chars = [x for x in content if not x.isspace()]
        to_verify = ''.join(chars)
        m = hashlib.sha256()
        m.update(to_verify.encode('ascii'))
        result = m.digest()
        return result == chksum.checksum

    @classmethod
    def parse(cls, content: str, verify: bool = True):
        """
        Produce a DDP from its string representation.

        :param content: The data to be parsed
        :param verify: If True, a checksum command is expected at the end of the file and verified
        :return: A `Script` instance
        """
        cmds = []
        for line in content.split(':'):
            line = line.strip()
            if len(line) == 0:
                continue
            data = _from_hexstring(line)
            if len(data) == 0:
                raise InvalidScript('Not enough data')
            cmd = data[0]
            rest = data[1:]
            cmds.append(_make_command(cmd, rest))
        ret = Script(cmds)
        if verify:
            ret._verify_content(content)
        return ret

    def _get_checksum_command(self) -> Optional[Checksum]:
        if len(self._cmds) == 0:
            return None
        if self._cmds[-1].type() == CommandType.CHECKSUM:
            cmd = self._cmds[-1]
            assert isinstance(cmd, Checksum)
            return cmd
        return None

    async def run(self, executor: Executor):
        """
        Exectu
        :param executor:
        :return:
        """
        state = State()
        executor.start()
        for cmd in self._cmds:
            await cmd.run(state, executor)
            if state.should_abort:
                executor.finished(False)
        executor.finished(True)


class Header(Command):
    def __init__(self, data: bytes):
        self._data = data

    @property
    def data(self):
        return self._data

    def type(self) -> CommandType:
        return CommandType.HEADER

    def dump(self) -> str:
        return 'Header'


class Write(Command):
    def __init__(self, data: bytes):
        self._data = data

    @property
    def data(self):
        return self._data

    def type(self) -> CommandType:
        return CommandType.WRITE

    def dump(self) -> str:
        return 'Write([{}])'.format(_to_hexstring(self._data))

    async def run(self, state: State, executor: Executor):
        try:
            await executor.write(self._data)
        except ComError as e:
            executor.print_debug('Write failed: {}'.format(e.message))
            executor.print_error(state.error_msg)
            state.abort()
            return
        if state.timeout_ms != 0:
            await executor.sleep(state.timeout_ms)


class Query(Command):
    def __init__(self, write: bytes, read: bytes):
        self._write = write
        self._read = read

    def type(self) -> CommandType:
        return CommandType.QUERY

    def dump(self) -> str:
        return 'Query([{0}, {1}])'.format(
            _to_hexstring(self._read), _to_hexstring(self._write))

    async def run(self, state: State, executor: Executor):
        try:
            read = await executor.query(self._write)
        except ComError as e:
            executor.print_debug('Query failed: {}'.format(e.message))
            executor.print_error(state.error_msg)
            state.abort()
            return
        if read != self._read:
            executor.print_debug('Query failed: Expected `{}` but got `{}`'.format(
                _to_hexstring(self._read), _to_hexstring(read)))
            executor.print_error(state.error_msg)
            state.abort()
            return
        if state.timeout_ms != 0:
            await executor.sleep(state.timeout_ms)

    @property
    def write(self):
        return self._write

    @property
    def read(self):
        return self._read


class SetTimeOut(Command):
    def __init__(self, timeout: int):
        self._timeout = timeout

    @property
    def timeout(self):
        return self._timeout

    def dump(self) -> str:
        return 'SetTimeOut({})'.format(self._timeout)

    def type(self) -> CommandType:
        return CommandType.SET_TIMEOUT

    async def run(self, state: State, executor: Executor):
        state.timeout_ms = self._timeout


class Log(Command):
    def __init__(self, msg):
        self._msg = msg

    def dump(self) -> str:
        return 'Log("{}")'.format(self._msg)

    def type(self) -> CommandType:
        return CommandType.LOG

    async def run(self, state: State, executor: Executor):
        executor.print_log(self._msg)


class SetError(Command):
    def __init__(self, msg):
        self._msg = msg

    def type(self) -> CommandType:
        return CommandType.SET_ERROR

    def dump(self) -> str:
        return 'SetError("{}")'.format(self._msg)

    async def run(self, state: State, executor: Executor):
        state.error_msg = self._msg


class Progress(Command):
    def __init__(self, progress: float):
        self._progress = progress

    def type(self) -> CommandType:
        return CommandType.PROGRESS

    def dump(self) -> str:
        return 'Prorgress({})'.format(self._progress)

    async def run(self, state: State, executor: Executor):
        executor.update_progress(self._progress)


def _make_command(cmd: int, data: bytes) -> Command:
    if cmd == CommandType.HEADER.value:
        return Header(data)
    elif cmd == CommandType.WRITE.value:
        return Write(data)
    elif cmd == CommandType.QUERY.value:
        if len(data) < 4:
            raise InvalidScript('Query: data length too short')
        write_len = struct.unpack('<H', data[0:2])[0]
        read_len = struct.unpack('<H', data[2:4])[0]
        if len(data) != 4 + write_len + read_len:
            raise InvalidScript('Query: invalid data length')
        write = data[4:4 + write_len]
        read = data[4 + write_len:]
        return Query(write, read)
    elif cmd == CommandType.SET_TIMEOUT.value:
        if len(data) != 4:
            raise InvalidScript("SetTimeOut: Invalid data length")
        timeout = struct.unpack('<L', data)[0]
        return SetTimeOut(timeout)
    elif cmd == CommandType.LOG.value:
        return Log(data.decode('utf-8'))
    elif cmd == CommandType.SET_ERROR.value:
        return SetError(data.decode('utf-8'))
    elif cmd == CommandType.PROGRESS.value:
        if len(data) != 1:
            raise InvalidScript("Progress: Invalid data length")
        return Progress(float(data[0]) / 255.0)
    elif cmd == CommandType.CHECKSUM.value:
        return Checksum(data)
    raise InvalidScript('Invalid Command')
