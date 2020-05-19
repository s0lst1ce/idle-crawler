import os

from displayer import Displayer


class Client:
    def __init__(self):
        self.displayer = Displayer()

    def screenTest(self):
        test = ""
        while(test != "quit"):
            self.displayer.clear_display()
            self.displayer.display_sample_window()
            test = input("Action: ")

client = Client()
client.screenTest()
