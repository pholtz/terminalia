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
from utilities import Compass, Character, View, Map


TERMINAL_ROWS = 24
TERMINAL_COLS = 80

NORTH = 0
EAST = 1
SOUTH = 2
WEST = 3


class Terminal:
	"""
		This class maintains the curses window. It uses Character, Map, and View subclasses.
		Takes a curses.initscr() window as input and draws application to it.
	"""
	def __init__(self, screen):
		#Initialize the screen and user input components
		self.screen = screen
		self.use_key = True
		self.lastkey = -1

		self.nontraversable = ["#", "@", "=", "-", "_", "|", "/", "\\"]

		#Initialize Container classes for gameplay
		self.map = Map("home")
		self.view = View(10, 10)
		self.player = Character("John Cena", 39, 11)
		self.compass = Compass()

		#Initialize the log file
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


	def display(self):
		"""Dump the text buffer to the terminal using curses.addch()."""
		for y, row in enumerate(self.view.text):
			for x, column in enumerate(row):
				if self.view.text[y][x] in self.nontraversable:
					self.screen.addch(y, x, ord(self.view.text[y][x]), curses.color_pair(2) | curses.A_BOLD)
				else:
					self.screen.addch(y, x, ord(self.view.text[y][x]), curses.color_pair(1) | curses.A_DIM)

		#Color the player -- use the standout property to distinguish from the current background
		self.screen.addch(self.player.y, self.player.x, self.player.avatar, curses.color_pair(1) | curses.A_STANDOUT)

		#Move the cursor back to the origin to prevent curses.ERR from being out of bounds
		self.screen.move(0, 0)



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
		#Always set the new direction -- regardless of whether we actually move
		self.player.direction = direction
		#Get the potential new coordinates
		row, col = self.move_direction(self.player.y, self.player.x, direction)

		#Check if the move is valid -- i.e. player is allowed to move onto this square
		if self.is_valid_move(row, col):
			#Determine whether we should reposition the view or the character
			#When walking towards a map edge that is in sight, move the character
			#When walking away from a map edge
			if self.at_edge(direction):
				#Move Character
				self.player.y, self.player.x = row, col
				self.error("At Edge: Moved player 1 unit {0}".format(direction))
			elif not self.player.centered(direction):
				#Player is not centered in this direction -- move player
				self.player.y, self.player.x = row, col
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



	def process_input(self, key):
		"""Process the user's input."""
		#QUIT PROGRAM
		if key == ord("q"):
			sys.exit()

		#CHARACTER MOVEMENT
		if key == ord("w"):
			self.move(NORTH)

		if key == ord("a"):
			self.move(WEST)

		if key == ord("s"):
			self.move(SOUTH)

		if key == ord("d"):
			self.move(EAST)

		#INVENTORY
		if key == ord("i"):
			self.show_inventory()


		#TESTING / DEBUGGING
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


	def show_inventory(self):
		pass



	def clear_errors(self):
		if os.path.exists("./error.log"):
			os.remove("./error.log")

	def error(self, message):
		with open("error.log", "a") as errorfile:
			errorfile.write(message + "\n")

