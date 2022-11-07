import abc
import time
import abc


class Callback:
    # Returns true if the callback wasn't stopped
    @abc.abstractmethod
    def call(self) -> bool:
        pass


class Stopper:
    @abc.abstractmethod
    def should_stop(self) -> bool:
        pass
