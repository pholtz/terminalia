#!/usr/bin/env python
# -*- coding: utf-8 -*-

#-------------------------------------------------------------------------------------#
# Summary:
# Terminal is a container class for a curses window as well as a character-based world
# and character animation using console graphics. Terminal loads a MAP file and allows
# the user to traverse the loaded world using WASD, showing a viewable portion of the 
# world through the terminal graphics.
#
# Name: terminal.py
# Author: Paul Holtz
# Date: 12/13/2015
#-------------------------------------------------------------------------------------#

#Builtin Imports
import os
import sys
import time
import curses
#Local Imports
from utilities import Stopwatch, Compass


TERMINAL_ROWS = 24
TERMINAL_COLS = 80

NORTH = 0
EAST = 1
SOUTH = 2
WEST = 3

PERIOD = 0.1


def main():
	try:
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
	curses.noecho()
	curses.cbreak()
	curses.curs_set(0)
	screen.keypad(1)
	screen.nodelay(1)
	#Color Pair creation for color terminals
	curses.init_pair(1, curses.COLOR_CYAN, curses.COLOR_BLACK)
	curses.init_pair(2, curses.COLOR_BLUE, curses.COLOR_BLACK)
	curses.init_pair(3, curses.COLOR_MAGENTA, curses.COLOR_BLACK)
	curses.init_pair(4, curses.COLOR_BLACK, curses.COLOR_WHITE)
	curses.init_pair(5, curses.COLOR_WHITE, curses.COLOR_YELLOW)
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





class Character:
	"""Generic class used to hold basic information about a game character."""
	def __init__(self, name, x, y):
		self.name = name
		self.x = x
		self.y = y
		self.dir = NORTH
		self.avatar = ord("@")

	def centered(self, direction=None):
		"""
		Test if the character is "centered" for the specified direction.
		What this roughly translates to is whether or not the character is 
		at the midpoint of the screen for a given direction.
		"""
		if direction is not None:


			if direction == NORTH or direction == SOUTH:
				#Test for horizontal centering
				if (TERMINAL_ROWS / 2) - 1 == self.y:
					return True
			elif direction == EAST or direction == WEST:
				#Test for vertical centering
				if (TERMINAL_COLS / 2) - 1 == self.x:
					return True

		else:
			#Default -- Tests for vertical & horizontal centering
			if (TERMINAL_ROWS / 2) - 1 == self.y and \
			   (TERMINAL_COLS / 2) - 1 == self.x:
				return True
		
		return False




class View:
	"""
	Houses the Local View, the terminal window of the world
	that the user sees.
	"""
	def __init__(self, x, y):
		#Initialize the View with "." characters
		self.text = self.initialize_text_buffer()

		self.x = x
		self.y = y


	def initialize_text_buffer(self):
		"""Create the text buffer used to store the current view."""
		text = []
		for row in range(TERMINAL_ROWS):
			text.append("")
			for column in range(TERMINAL_COLS):
				text[row] += " "
		return text




class Map:
	"""
	Container for the World View, the entire character matrix
	representing every location currently loaded into the game.
	"""
	def __init__(self, mapname):
		#Read the map from the MAP file
		self.name = mapname
		self.text = self.read_map(mapname)

		self.height = len(self.text)
		self.width = len(self.text[0])

		self.min_row = 0
		self.min_col = 0

		self.max_row = self.min_row + self.height
		self.max_col = self.min_col + self.width


	def read_map(self, mapname):
		"""Read the MAP file and create the map matrix."""
		mapbuffer = ""
		with open("./{0}.map".format(mapname), "r") as mapfile:
			mapbuffer = mapfile.read()

		mapmatrix = []
		for y, line in enumerate(mapbuffer.split("\n")):
			mapmatrix.append("")
			for x, character in enumerate(line):
				mapmatrix[y] += character

		return mapmatrix




class Terminal:
	"""
		This class runs the application loop and maintains the curses window.
		Takes a curses.initscr() window as input and draws application to it.
	"""
	def __init__(self, screen):

		self.screen = screen
		self.use_key = True
		self.lastkey = -1

		self.nontraversable = ["@", "-", "_", "|"]

		#Initialize Container classes for gameplay
		self.map = Map("barren")
		self.view = View(10, 10)
		self.player = Character("John Cena", 39, 11)
		self.compass = Compass()

		self.clear_errors()

		self.error("Map: {0}\nRows: {1}\nCols: {2}\n".format(self.map.name, self.map.max_row, self.map.max_col))





	def application_frame(self):

		#Get user input
		key = self.screen.getch()
		if key != -1:
			self.lastkey = key

		#Process Inputs
		if self.use_key:
			self.process_input(self.lastkey)
			self.use_key = False
			self.lastkey = -1

		#Update the text buffer
		self.update_text()

		#Push the text buffer to the screen and refresh
		self.display()
		self.screen.refresh()




	def update_text(self):
		"""Update the text buffer with the current map image and character."""
		#Get the desired min & max row indices
		top = self.view.y
		bottom = self.view.y + TERMINAL_ROWS
		#Get the desired min & max column indices
		left = self.view.x
		right = self.view.x + TERMINAL_COLS
		#Load the map background into the text buffer
		for y, row in enumerate(self.map.text[top:bottom]):
			#self.view.text[y] = self.map.text[y][left:right]
			self.view.text[y] = row[left:right]

		#Load the player avatar into the text buffer
		#line_list = list(self.view.text[self.player.y])
		#line_list[self.player.x] = self.player.avatar
		#self.view.text[self.player.y] = "".join(line_list)



	def is_valid_move(self, row, column):
		"""Check desired square for nontraversable characters."""
		for character in self.nontraversable:
			if self.view.text[row][column] == character:
				return False
		#Desired square does not contain a nontraversable characters
		return True


	def move(self, direction):
		"""
		Reposition either the character or the view, depending on current map location.
		Assume that the move is valid (i.e. terrain is traversable).
		"""
		#Determine whether we should reposition the view or the character
		#When walking towards a map edge that is in sight, move the character
		#When walking away from a map edge
		if self.at_edge(direction):
			#Move Character
			self.player.y, self.player.x = self.move_direction(self.player.y, self.player.x, direction)
			self.error("At Edge: Moved player 1 unit {0}".format(direction))
		elif not self.player.centered(direction):
			#Player is not centered in this direction -- move player
			self.player.y, self.player.x = self.move_direction(self.player.y, self.player.x, direction)
			self.error("Not Centered: Moved player 1 unit {0}".format(direction))
		else:
			#Move View
			self.view.y, self.view.x = self.move_direction(self.view.y, self.view.x, direction)
			self.error("Normal: Moved view 1 unit {0}".format(direction))



	def at_edge(self, direction=None):
		"""
		Summary:
			Test if viewport has reached any edge of the map.
			If a direction is specified, then test against that
			direction and nothing else.
		Params:
			direction - A cardinal direction NSWE to check (optional)
		Returns: 
			True if the specified directional edge has been reached. (direction specified)
			True when any edge has been reached. (no direction specified)
		"""
		if direction is not None:
			if direction == NORTH:
				if self.view.y <= self.map.min_row:
					self.error("{0} <= {1}".format(self.view.y, self.map.min_row))
					return True
				return False
			elif direction == EAST:
				if (self.view.x + TERMINAL_COLS) >= self.map.max_col:
					return True
				return False
			elif direction == SOUTH:
				if (self.view.y + TERMINAL_ROWS) >= self.map.max_row:
					return True
				return False
			elif direction == WEST:
				if self.view.x <= self.map.min_col:
					return True
				return False
			else:
				self.error("Invalid direction {0}".format(direction))
		else:
			if self.view.x <= self.map.min_row or \
			   self.view.x >= self.map.max_row or \
			   self.view.y <= self.map.min_col or \
			   self.view.y >= self.map.max_col:
				return True
			return False


	def move_direction(self, row, column, direction):
		if direction == NORTH:
			return row - 1, column
		elif direction == EAST:
			return row, column + 1
		elif direction == SOUTH:
			return row + 1, column
		elif direction == WEST:
			return row, column - 1
		else:
			return row, column



	def display(self):
		"""Dump the text buffer to the terminal using curses.addch()."""
		for y, row in enumerate(self.view.text):
			for x, column in enumerate(row):
				self.screen.addch(y, x, ord(self.view.text[y][x]))

		#Color the player
		self.screen.addch(self.player.y, self.player.x, self.player.avatar, curses.color_pair(1))

		#Move the cursor back to the origin to prevent curses.ERR from being out of bounds
		self.screen.move(0, 0)



	def process_input(self, key):
		"""Process the user's input."""
		#QUIT PROGRAM
		if key == ord("q"):
			sys.exit()

		#CHARACTER MOVEMENT
		if key == ord("w"):
			self.player.direction = NORTH
			if self.is_valid_move(self.player.y - 1, self.player.x):
				self.move(NORTH)

		if key == ord("a"):
			self.player.direction = WEST
			if self.is_valid_move(self.player.y, self.player.x - 1):
				self.move(WEST)

		if key == ord("s"):
			self.player.direction = SOUTH
			if self.is_valid_move(self.player.y + 1, self.player.x):
				self.move(SOUTH)

		if key == ord("d"):
			self.player.direction = EAST
			if self.is_valid_move(self.player.y, self.player.x + 1):
				self.move(EAST)

		#Test
		if key == ord("f"):
			curses.flash()

		if key == ord("1"):
			for row in self.view.text:
				self.error(row)

		if key == ord("2"):
			self.error("Player >>> row: {0}, col: {1}".format(self.player.y, self.player.x))
			self.error("View   >>> row: {0}, col: {1}".format(self.view.y, self.view.x))

		if key == ord("3"):
			self.error("START MAP TEXT")
			for row in self.map.text:
				self.error(row)
			self.error("END MAP TEXT")




	def clear_errors(self):
		if os.path.exists("./error.log"):
			os.remove("./error.log")

	def error(self, message):
		with open("error.log", "a") as errorfile:
			errorfile.write(message + "\n")



#------------------------#
#  E N T R Y  P O I N T  #
#------------------------#
if __name__ == "__main__":
	main()