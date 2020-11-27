#!/usr/bin/env python3

## Purpose:	Allows user to keep additional data in their CSV Input File to increase its usefulness and
##			enhance readability, yet be able to properly format it prior to importing into `cryptools`.
##			e.g.:
##				-Keep an additional first column for flagging/noting important transactions
##				-Keep additional columns for tracking a running balance
##				-Rows beneath transactions for life-to-date totals and other calculations and notes
##				-Ability to use number formatting with parenthesis for negative numbers and commas
##					-This script will change (1,000.00) to 1000.00
##
##			If a column doesn't have a header, this script will exclude it from the sanitized output.
##			Similarly, this script will exclude transaction rows missing data in either of the first
##			two fields of the row.

## Usage:

# 1.  Export/Save crypto activity as csv
# 2.  Move the csv file to your desired directory
# 3.  Rename file to <unedited>.csv (see variable below)
# 4.  Build/run this file in an editor or on command line (from same directory), creating the input file
# 5.  Import the input file into cryptools

import csv
import re
import os
import sys

unedited = "DigiTrnx.csv" 	# To be replaced with a launch arg, presumably	

stage1 = "stage1.csv"

##		First, writes all header rows.  Then attempts to write all transaction rows.
##      In the transaction rows, if it finds blank/empty transaction date or proceeds fields,
##		it discards the row.

##		This allows notes/sums/calculations/etc under the transaction rows to be discarded

with open(unedited) as fin, open(stage1, 'a') as fout:
	rdr = csv.reader(fin) 	
	wtr = csv.writer(fout)
	header = next(rdr)
	header2 = next(rdr)
	header3 = next(rdr)
	header4 = next(rdr)		
	wtr.writerow(header)
	wtr.writerow(header2)
	wtr.writerow(header3)
	wtr.writerow(header4)

	# First, double check there are no account number duplicates
	for i, val in enumerate(header):
		if val != "":
			if header.count(val) > 1:
				print("### There is a duplicate account number (" + val +").  Please fix and re-run. ###")
				sys.exit()

	for row in rdr:
		if row[0] == "" or row[1] == "":
			pass
		else:
			wtr.writerow(row)

stage2 = "stage2.csv"

##		Iterates over the fields in the first header row to search for empty/blank cells.
##      Keeps a list of every column index that does contain data, and disregards all the
##		indices for columns with a blank.

##		Using the indicies of valid columns, writes a new CSV file using only valid columns.

##		This is useful when the input file is also used to manually keep a running tally or
##		columns with additional notes, but which must be discarded to prepare a proper
##		CSV input file.

with open(stage1) as fin, open(stage2, 'a') as fout:
	rdr = csv.reader(fin)
	wtr = csv.writer(fout)
	header = next(rdr)
	header2 = next(rdr)
	header3 = next(rdr)
	header4 = next(rdr)

	colListKept = []

	for col in header:
		if col == "":
			pass
		else:
			colListKept.append(header.index(col))

	output = [v for (i,v) in enumerate(header) if i in colListKept]
	wtr.writerow(output)

	output = [v for (i,v) in enumerate(header2) if i in colListKept]
	wtr.writerow(output)

	output = [v for (i,v) in enumerate(header3) if i in colListKept]
	wtr.writerow(output)

	output = [v for (i,v) in enumerate(header4) if i in colListKept]
	wtr.writerow(output)

	for row in rdr:
		output = [v for (i,v) in enumerate(row) if i in colListKept]
		wtr.writerow(output)


stage3 = "InputFile-pycleaned.csv"

##		Performs final formatting changes to ensure values can be successfully parsed.
##	 	Numbers must have commas removed.  Negative numbers must have parentheses replaced
##		with a minus sign.  Could also be used to substitute the date separation character.

##	 	i.e., (1.01) -> -1.01  (1,000.00) -> -1000.00

with open(stage2) as fin, open(stage3, 'w') as fout:
	rdr = csv.reader(fin, quoting=csv.QUOTE_ALL) 	
	wtr = csv.writer(fout)
	
	header = next(rdr)
	header2 = next(rdr)
	header3 = next(rdr)
	header4 = next(rdr)
	wtr.writerow(header)
	wtr.writerow(header2)
	wtr.writerow(header3)
	wtr.writerow(header4)

	for row in rdr:
		listRow = []
		for field in row:
			fieldStr = str(field) # cast as string, just so there's no funny business
			try:
				# Handles negative numbers
				if fieldStr[0] == "(":
					fieldStr = fieldStr.replace('(','-').replace(')', '').replace(',', '')
					listRow.append(fieldStr)
					continue

			# Uncomment the below and modify as necessary if you want to change date formatting
				# elif re.search(r'\d\d-\d\d-\d\d',fieldStr):#	Find dates and change formatting
				# 	fieldStr = fieldStr.replace('-', '/')
				# 	listRow.append(fieldStr)
				# 	continue

				# Handle commas in remaining fields
				else: 
					try:
						# if you remove commas from a string and are able to convert to float...
						fieldStr_test = fieldStr.replace(',', '')
						fieldStr_float = float(fieldStr_test)
						# then it is definitely a positive number, so remove the comma.
						fieldStr = fieldStr.replace(',', '')
						listRow.append(fieldStr)
						continue
					except: # If the 'try' block fails, it's a memo, not a number, so leave any commas
						listRow.append(fieldStr)
						continue
			except: # If the `try` block fails, it's a blank/empty string
				listRow.append(fieldStr)
				continue
		wtr.writerow(listRow)

os.remove(stage1)
os.remove(stage2)

print("Input file ready")
