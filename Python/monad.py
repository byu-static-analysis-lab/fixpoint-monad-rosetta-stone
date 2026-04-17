from copy import deepcopy
from functools import reduce


class StateValue:
    def __init__(self, *conts):
        self.values = set()
        continuations = []
        for cont in conts:
            continuations.append(cont)
        self.continuations = continuations
        self.index = 0

    def add_waiter(self, cont):
        this = deepcopy(self)
        this.continuations.append(cont)
        return this
    
    def add_values(self, *values):
        this = deepcopy(self)
        for value in values:
            this.values.add(value)
        return this
    
    def contains(self, value):
        return value in self

    def __contains__(self, value):
        return value in self.values
    
    def is_empty(self):
        return len(self.values) == 0
    
    def __next__(self):
        if self.index >= len(self.continuations):
            self.index = 0
            raise StopIteration
        out = self.continuations[self.index]
        self.index += 1
        return out
    
    def __repr__(self):
        return repr(self.values)
    
    def __str__(self):
        return str(self.values)
    
class State:
    def __init__(self, user_state=None):
        self.map = {}
        self.user_state = user_state


    def __getitem__(self, key):
        return self.map.get(key)
    
    def insert(self, tag, value):
        this = deepcopy(self)
        this.map[tag] = value
        return this
    
    def contains_value(self, tag, expected):
        value = self.map.get(tag)
        if value is not None:
            return expected in value
        else:
            return False
        
    def is_empty_tag(self, tag):
        value = self.map.get(tag)
        match value:
            case None:
                return True
            case _:
                return value.is_empty()
            
    def get_user_state(self):
        return self.user_state
    
    def set_user_state(self, new_state):
        this = deepcopy(self)
        this.user_state = new_state
        return this
            
    def __str__(self):
        if self.user_state is None:
            out = ""
        else:
            out = f"user_state: {self.user_state}\n\n"
        for tag, value in self.map.items():
            out = out + f"{tag} ↦ {value}\n"
        return out

class Transformer:
    def __init__(self, func):
        self.func = func
    
    def __call__(self, state):
        new_state = deepcopy(state)
        return self.func(new_state)

class Continuation:
    def __init__(self, func):
        self.func = func

    def __call__(self, *args):
        return self.func(*args)

class ContinuationMonad:
    def __init__(self, func):
        self.func = func
    
    def __call__(self, cont: Continuation):
        cont = deepcopy(cont)
        return self.func(cont)
    
    @staticmethod
    def inject(*args):
        return ContinuationMonad(lambda continuation: continuation(args))
    
    def bind(self, f):
        return ContinuationMonad(lambda cont: Continuation(lambda *args: f(*args)))
    
    @staticmethod
    def each(*monads):
        return ContinuationMonad(lambda cont: Transformer(lambda s: reduce(lambda monad, state: monad(cont)(state), *monads, s)))
    
    @staticmethod
    def fail():
        return ContinuationMonad.each()

    def run(self):
        return self(Continuation(lambda _: Transformer(lambda s: s)))(State())


class Memo:
    def __init__(self, tag, func):
        self.tag = tag
        self.func = func

    def __call__(self, *args):
        tag = f"{self.tag}:{list(args)}"
        func = deepcopy(self.func)

        def monad_fn(cont):
            def transformer_fn(s: State):
                match s[tag]:
                    case None:
                        new_s = s.insert(tag, StateValue(cont))
                        monad = func(*args)
                        deliver_cont = Memo.deliver(tag)
                        transformer = monad(deliver_cont)
                        return transformer(new_s)
                    case st_value:
                        new_s_value = st_value.add_waiter(cont)
                        new_st = s.insert(tag, new_s_value)
                        for value in new_s_value.values:
                            transformer = cont(value)
                            new_st = transformer(new_st)
                        return new_st
            return Transformer(transformer_fn)
        return ContinuationMonad(monad_fn)
    
    @staticmethod
    def sigma():
        return Memo("σ", lambda _: ContinuationMonad.fail())
    
    @staticmethod
    def deliver(tag):
        def cont_fn(*vs):
            def transformer_fn(s: State):
                match s[tag]:
                    case None:
                        state_value = StateValue()
                    case v:
                        state_value = v
                
                for value in vs:
                    if value in state_value:
                        return s
                
                state_value = state_value.add_values(*vs)
                new_state = s.insert(tag, state_value)

                for cont in state_value:
                    transformer = cont(*vs)
                    new_state = transformer(new_state)
                return new_state
            return Transformer(transformer_fn)
        return Continuation(cont_fn)
    
    @staticmethod
    def bind_var(name, value):
        def monad_fn(cont):
            def transformer_f(s: State):
                # first deliver the value to the variable's address
                tag = f"σ:{name}"
                deliver_cont = Memo.deliver(tag)
                deliver_trans = deliver_cont(value)
                new_s = deliver_trans(s)

                # then call the continuation with an empty list
                return cont()(new_s)
            return Transformer(transformer_f)
        return ContinuationMonad(monad_fn)