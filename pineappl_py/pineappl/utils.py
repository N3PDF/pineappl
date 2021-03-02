class PyWrapper:
    _raw = None

    @property
    def raw(self):
        return self._raw

    def __getattr__(self, name):
        if name[0] != "_":
            return self._raw.__getattribute__(name)
        else:
            raise AttributeError

    # def __getattribute__(self, name):
    # print(">>>", name)
    # return super().__getattribute__(name)