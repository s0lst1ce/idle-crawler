class Tile():
    """a Tile that holds all the data and events of this tile"""
    def __init__(self, pos: tuple, data: dict = Tile.new_tile):
        self.pos = pos
        self.data = data
        self.events = []
        self.players = {} # {"username": idx:int} the index is the index of the last event retrieved

        self.retrieved_order = [] #each player appends its username to the end and removes otther entries when getting event, the username at idx=0 is the oldest one

    async def register(self, event: dict) -> None:
        self.events.append(event)

    async def add_player(self, username: str) -> None:
        self.players[username] = len(self.events) - 1
        self.retrieved_order.append(username)

    async def get_player_event(self, username: str) -> list: #list of dict events OR 
        assert username in self.players, ValueError(f"Player {username} is not registered on tile {self.pos}!")

        idx = self.players[username]

        #update last read event
        self.players[username] = len(self.events) -1
        
        #allowing event history cleanup
        self.retrieved(username)
        
        return self.events[idx+1:]

    def retrieved(self, username: str): #WARN: not async because we can't allow a player to get their events while moving the offsets
        update = username == self.retrieved_order[0]

        self.retrieved_order.pop(self.retrieved_order.index(username))
        self.retrieved_order.append(username)

        if update:
            offset = self.players[username]
            for player in self.players:
                self.players[player] -= offset

            self.events = self.events[offset:]