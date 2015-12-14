#!/usr/bin/env python
# -*- coding: utf-8 -*-

#-------------------------------------------------------------------------------------#
# Summary:
#
# Name: terminal.py
# Author: Paul Holtz
# Date: 12/13/2015
#-------------------------------------------------------------------------------------#

import os
import sys
import time
import curses


TERMINAL_ROWS = 24
TERMINAL_COLS = 80


def main():
	terminal = Terminal()
	terminal.application_loop()




class Character:
	"""Generic class used to hold basic information about a game character."""
	def __init__(self, name, x, y):
		self.name = name
		self.x = x
		self.y = y
		self.avatar = "O"




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
		for row in range(24):
			text.append("")
			for column in range(80):
				text[row] += "."
		return text




class Map:
	"""
	Container for the World View, the entire character matrix
	representing every location currently loaded into the game.
	"""
	def __init__(self, mapname):
		#Read the map from the MAP file
		self.text = self.read_map(mapname)

		self.width = len(self.text[0])
		self.height = len(self.text)


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
	"""This class runs the application loop and maintains the curses window."""
	def __init__(self):
		#Curses Initializations
		self.screen = curses.initscr()
		curses.noecho()
		curses.cbreak()
		curses.curs_set(0)
		self.screen.keypad(1)
		self.screen.nodelay(1)

		self.nontraversable = ["@"]


		#Initialize Container classes for gameplay
		self.map = Map("barren")
		self.view = View(0, 0)
		self.player = Character("John Cena", 39, 11)


		self.clear_errors()


	def application_loop(self):
		try:

			#Application Loop
			while True:

				#Get user input
				key = self.screen.getch()

				#Process Inputs
				self.process_input(key)

				#Update the text buffer
				self.update_text()

				#Update the screen
				self.display()
				self.screen.refresh()


		finally:
			#Exit gracefully
			self.screen.keypad(0)
			self.screen.nodelay(0)
			curses.nocbreak()
			curses.curs_set(2)
			curses.echo()
			curses.endwin()




	def update_text(self):
		"""Update the text buffer with the current map image and character."""
		#Load the map background into the text buffer
		for y, row in enumerate(self.map.text[self.view.y : (self.view.y + 24)]):
			self.view.text[y] = self.map.text[y][self.view.x : (self.view.x + 80)]

		#Load the player avatar into the text buffer
		line_list = list(self.view.text[self.player.y])
		line_list[self.player.x] = self.player.avatar
		self.view.text[self.player.y] = "".join(line_list)



	def display(self):
		"""Dump the text buffer to the terminal using curses.addch()."""
		for y, row in enumerate(self.view.text):
			for x, column in enumerate(row):
				self.screen.addch(y, x, ord(self.view.text[y][x]))
		#Move the cursor back to the origin to prevent curses.ERR from being out of bounds
		self.screen.move(0, 0)




	def process_input(self, key):
		"""Process the user's input."""
		#QUIT PROGRAM
		if key == ord("q"):
			sys.exit()

		#CHARACTER MOVEMENT
		if key == ord("w"):
			if self.is_valid_move(self.player.y - 1, self.player.x):
				self.player.y -= 1

		if key == ord("a"):
			if self.is_valid_move(self.player.y, self.player.x - 1):
				self.player.x -= 1

		if key == ord("s"):
			if self.is_valid_move(self.player.y + 1, self.player.x):
				self.player.y += 1

		if key == ord("d"):
			if self.is_valid_move(self.player.y, self.player.x + 1):
				self.player.x += 1

		#Test
		if key == ord("f"):
			curses.flash()


	def is_valid_move(self, row, column):
		"""Check desired square for nontraversable characters."""
		for character in self.nontraversable:
			if self.view.text[row][column] == character:
				return False
		#Desired square does not contain a nontraversable characters
		return True


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