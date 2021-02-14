# Intro

The way most bibliographic software process .bib files is very annoying; they mess up with fields relentlessly. Despite that, they offer some very interesting capabilities, such as auto lookup. This is a way around that.

The idea of this script is to consolidate all the bibliographic data one has in .bib files spread around a folder, consolidate it in a unique file ready to be imported and treated in some of the available programs such as mendeley and zotero.

no assumptions, non destructive behaviour

# types
article
book
inproceedings
misc
collection
unpublished
incollection
online
report
thesis

# Remarks
- info like year and month is consolidated in date field, if clean is set

a article
b book
r reference

key: `year_first author last name_[number]`
path: `[+,!] a|b|r key [short]title`

\! -> reviewed and linked with reviewed entry
\# -> linked

date: 2009-01-31
filename to path:
  : -> --

# usage:
  bibrust or bibrust path/to/folder -> scan
