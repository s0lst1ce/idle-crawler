import platform
import os

class Displayer:
    def __init__(self):
        self.os = platform.system()
        self.window_size = {"width": 0, "height": 0}

        self.refresh_window_size()

    def refresh_window_size(self):
        screen_max_width = 400
        screen_min_width = 80
        screen_max_height = 100
        screen_min_height = 30

        width, height = os.get_terminal_size(0)
        self.window_size["width"] = max(min(int(width), screen_max_width), screen_min_width)
        self.window_size["height"] = max(min(int(height), screen_max_height), screen_min_height)

    def clear_display(self):
        if self.os == "Windows":
            os.system("cls")
        else:
            os.system("clear")

    def display_sample_window(self):
        self.refresh_window_size()
        for y in range(0, self.window_size["height"] - 1):
            line = "+" if( y == 0 or y == self.window_size["height"] - 2) else "|"
            for x in range (2, self.window_size["width"]):
                line += "-" if( y == 0 or y == self.window_size["height"] - 2) else " "
            line += "+" if( y == 0 or y == self.window_size["height"] - 2) else "|"
            print(line)
