#!/usr/bin/env python
# -*- coding: utf-8 -*-

#-------------------------------------------------------------------------------------#
# Summary:
# 
#
# Name: terminalia.py
# Author: Paul Holtz
# Date: 12/18/2015
#-------------------------------------------------------------------------------------#

#Builtin Imports
import os
import sys
import time
import curses
#Local Imports
from terminal import Terminal
from utilities import Stopwatch


PERIOD = 0.01


def main():
	try:
		os.environ["TERM"] = "xterm-256color"

		screen = init_curses()
		terminal = Terminal(screen)

		timer = Stopwatch()

		#Application Loop
		while True:
			#Determine if user input should be used this cycle
			if timer.get_elapsed() >= PERIOD:
				terminal.use_key = True
				timer.reset()
			#Generate and display the terminal frame
			terminal.application_frame()

	finally:
		exit_curses(screen)


def init_curses():
	#Curses Initializations
	screen = curses.initscr()
	curses.start_color()
	curses.use_default_colors()
	curses.noecho()
	curses.cbreak()
	curses.curs_set(0)
	screen.keypad(1)
	screen.nodelay(1)
	#Color Pair creation for color terminals
	#curses.init_pair(1, curses.COLOR_WHITE, curses.COLOR_BLACK)
	#curses.init_pair(2, curses.COLOR_CYAN, curses.COLOR_BLACK)
	#curses.init_pair(3, curses.COLOR_MAGENTA, curses.COLOR_BLACK)
	#curses.init_pair(4, curses.COLOR_BLACK, curses.COLOR_WHITE)
	#curses.init_pair(5, curses.COLOR_WHITE, curses.COLOR_YELLOW)

	#Terminalia Normal Element Background
	curses.init_pair(1, 240, 28)
	#Terminalia Nontraversable Background
	curses.init_pair(2, 7, 28)
	#curses.init_pair(2, 22, 255)
	#curses.init_pair(3, 200, 255)

	#Return a reference to the initialized screen object
	return screen

def exit_curses(screen):
	#Exit gracefully and unset screen changes
	screen.keypad(0)
	screen.nodelay(0)
	curses.nocbreak()
	curses.curs_set(2)
	curses.echo()
	curses.endwin()


#------------------------#
#  E N T R Y  P O I N T  #
#------------------------#
if __name__ == "__main__":
	main()