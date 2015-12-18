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

import time


TERMINAL_ROWS = 24
TERMINAL_COLS = 80

NORTH = 0
EAST = 1
SOUTH = 2
WEST = 3


class Character:
	"""Generic class used to hold basic information about a game character."""
	def __init__(self, name, x, y):
		self.name = name
		self.x = x
		self.y = y
		self.dir = NORTH
		self.avatar = ord("@")
		self.inventory = []

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
		with open("./maps/{0}.map".format(mapname), "r") as mapfile:
			mapbuffer = mapfile.read()

		mapmatrix = []
		for y, line in enumerate(mapbuffer.split("\n")):
			mapmatrix.append("")
			for x, character in enumerate(line):
				mapmatrix[y] += character

		return mapmatrix




class Stopwatch:
	def __init__(self):
		self.reset()

	def reset(self):
		self.old = time.clock()
		self.new = time.clock()

	def get_elapsed(self):
		self.new = time.clock()
		return self.new - self.old


class Compass:
	"""Helper class for dealing with cardinal directions."""
	def __init__(self):
		self.direction = self.enum("NORTH", "EAST", "SOUTH", "WEST")

	def enum(self, *sequential, **named):
	    enums = dict(zip(sequential, range(len(sequential))), **named)
	    return type('Enum', (), enums)

	def get_string(self, enumeration):
		"""Convert a numerical index into a dictional string."""
		if enumeration == self.direction.NORTH:
			return "North"
		elif enumeration == self.direction.EAST:
			return "East"
		elif enumeration == self.direction.SOUTH:
			return "South"
		elif enumeration == self.direction.WEST:
			return "West"
		else:
			return "Invalid Index"