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